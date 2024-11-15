
use std::collections::VecDeque;
use std::collections::vec_deque;
use std::collections::btree_set::BTreeSet;

use crate::log;
use crate::frame_number;
use crate::Input;

use crate::PositionedGraphic;
use crate::resources::GraphicGroup;
use crate::Graphic;
use crate::GraphicFlags;
use crate::LingeringGraphic;

use crate::objects::Object;
use crate::objects::Direction;
use crate::objects::ObjectBounds;
use crate::objects::BrickType;
use crate::objects::HitBox;

use crate::brick::Brick;

use crate::game::TargetInfo;

use crate::GROUND_POS;
use crate::LEFT_BOUNDARY;
use crate::RIGHT_BOUNDARY;
use crate::TIME_ZERO_BRICK_POS;
use crate::F32_ZERO;
use crate::GameData;
use crate::objects::PLAYER_WIDTH;
use crate::objects::PLAYER_HEIGHT;
use crate::objects::BRICK_WIDTH;
use crate::objects::MIN_DASH_WIDTH;
use crate::objects::SLASH_WIDTH;
use crate::objects::HOLD_HITBOX_WIDTH;
use crate::objects::HOLD_HITBOX_HEIGHT;

// delays dash/slash by a tiny amount so they can be pressed at the same time. starts animation during delay
pub const PRE_SLASH_TIME: f32 = 0.06; 
pub const PRE_HOLD_TIME: f32 = 0.16;
// so slash animation can finish
pub const POST_SLASH_TIME: f32 = 0.08;
const DASH_LINGER_TIME: f32 = 0.3; // how long the dash graphic lingers
const BOOST_LINGER_TIME: f32 = 0.3;
const BOOST_PRELINGER_TIME: f32 = 1.2;

pub const RUN_SPEED: f32 = 550.0; // in pixels per second
pub const WALK_SPEED: f32 = 250.0; // in pixels per second

pub const MAX_BOOST_DISTANCE: f32 = 4.0 * PLAYER_WIDTH as f32;
pub const BOOST_GRAPHIC_OFFSET: f32 = PLAYER_WIDTH as f32 / 10.0; // how close the boost graphics are to one another

pub struct Player {
	state: TaggedState,
	bounds: ObjectBounds,
	
	// !!! this way of getting the size of the enum is flimsy. Would prefer enum_variant_count!(Input)
	inputs_down: [bool; Input::Slash3 as usize + 1], 
	face_dir: Direction,
	hit_dir: Direction,
	
	hit_type: Option<BrickType>,

	target: Option<TargetInfo>,
	hold_positions: Vec<f32>,
	
	// flag indicating whether to go into a hold (false) or end the slash regularly without holding (true)
		// set to false when the slash input is taken, set to true when the key up end input is received
	stop_hold: bool, 
	
	lingering_graphics: Vec<LingeringGraphic>
}

enum PlayerState {
	Standing,
	Walking,
	Running,
	PreSlash,
	PreDash,
	PreSlashDash,
	Slash, 
	Dash,
	SlashDash,
	PostSlash,
	Hold, 
}

struct TaggedState {
	// init time of the state. so there can be a delay before becoming active/inactive and for animation frame calculation
	time: f32,
	state: PlayerState
}

impl Object for Player {
	fn bounds(&self) -> ObjectBounds {
		self.bounds
	}
}

impl Player {
	
	pub fn new(x: f32) -> Player {
		Player {
			state: TaggedState { time: 0.0, state: PlayerState::Standing },
			bounds: ObjectBounds {
				left_x: x,
				top_y: GROUND_POS as f32 - PLAYER_HEIGHT as f32,
				right_x: x + PLAYER_WIDTH as f32, 
				bottom_y: GROUND_POS as f32
			},
			
			inputs_down: [false; Input::Slash3 as usize + 1],
			face_dir: Direction::Right,
			hit_dir: Direction::Right,
			
			hit_type: None,

			target: None,
			hold_positions : Vec::new(),
			
			stop_hold: false,
			
			lingering_graphics: Vec::new() // graphics for objects no longer present but still showing, e.g. slashes/dashes that have executed
		}
	}
	
	pub fn check_action(&self, end_tick_time: f32) -> Option<f32> {
		let state_time = self.state.time;
		
		match self.state.state {
			PlayerState::PreSlash | PlayerState::PreSlashDash => {
				let action_time = state_time + PRE_SLASH_TIME;
				
				if action_time <= end_tick_time {
					return Some(action_time);
				} else {
					return None;
				}
			},
			_ => {
				return None;
			}
		}
	}
	
	// tick the player's state
	pub fn tick(&mut self, seconds_passed: f32, game_data: &GameData, target: Option<TargetInfo>) {
		self.target = target;

		self.update_state(seconds_passed, game_data);
		
		// TODO would prefer if cloning the lingering graphics before removing them was unnecessary
		let new_set: Vec<LingeringGraphic> = self.lingering_graphics.iter().filter(|lg| lg.end_t > game_data.time_running).cloned().collect();
		self.lingering_graphics = new_set;
	}
	
	// perform a slash or slash dash if the state is correct (in preslash or preslashdash), returns hitbox and updates state
	pub fn action_tick (&mut self, game_data: &GameData, target: Option<TargetInfo>) -> HitBox {
		self.target = target;
		
		let hitbox;
		let time_running = game_data.time_running;
		let t = self.state.time;

		match self.state.state {
			PlayerState::PreSlash => {
				// boost if in range of target
				if let Some(ti) = &self.target {
					let pos_difference = self.bounds.left_x - ti.dest_x;
					if pos_difference > -MAX_BOOST_DISTANCE && pos_difference < MAX_BOOST_DISTANCE && !ti.dash_to_target {
						self.boost(time_running);
					}
				}

				// get hitbox
				let brick_type = if let Some(bt) = self.hit_type {bt} else {panic!()};
				let hitbox_x = match self.hit_dir {
					Direction::Right => self.bounds.right_x,
					Direction::Left => self.bounds.left_x - SLASH_WIDTH as f32
				};
				let hitbox_bounds = ObjectBounds { 
					left_x: hitbox_x, 
					right_x: hitbox_x + SLASH_WIDTH as f32, 
					top_y: self.bounds.top_y,
					bottom_y: self.bounds.bottom_y
				};
				
				hitbox = HitBox { bounds: hitbox_bounds, brick_type };
					
				self.state = TaggedState { state: PlayerState::Slash, time: time_running };
			},
			PlayerState::PreSlashDash => {
				// boost if in range of target
				if let Some(ti) = &self.target {
					let pos_difference = self.bounds.left_x - ti.dest_x;
					if pos_difference > -MAX_BOOST_DISTANCE && pos_difference < MAX_BOOST_DISTANCE && !ti.dash_to_target {
						self.boost(time_running);
					}
				}
				
				let brick_type = if let Some(bt) = self.hit_type { bt } else { panic!() };
				
				let dash_left_x;
				let dash_right_x;
				if let Some(ti) = &self.target { 
					self.bounds.left_x = ti.post_hit_x;
					self.bounds.right_x = self.bounds.left_x + PLAYER_WIDTH as f32;
					match self.hit_dir {
						Direction::Right => {
							dash_left_x = ti.dest_x + PLAYER_WIDTH as f32;
							dash_right_x = ti.post_hit_x;
							self.face_dir = Direction::Right;
						},
						Direction::Left => {
							dash_left_x = ti.post_hit_x + PLAYER_WIDTH as f32;
							dash_right_x = ti.dest_x;
							self.face_dir = Direction::Left;
						}
					}
				} else { 
					match self.hit_dir {
						Direction::Right => {
							self.bounds.left_x = self.bounds.left_x + MIN_DASH_WIDTH as f32;
							self.bounds.right_x = self.bounds.left_x + PLAYER_WIDTH as f32;
							dash_left_x = self.bounds.left_x - MIN_DASH_WIDTH as f32;
							dash_right_x = self.bounds.left_x;
							self.face_dir = Direction::Right;
						},
						Direction::Left => {
							self.bounds.left_x = self.bounds.left_x - MIN_DASH_WIDTH as f32;
							self.bounds.right_x = self.bounds.left_x + PLAYER_WIDTH as f32;
							dash_left_x = self.bounds.right_x;
							dash_right_x = self.bounds.right_x + MIN_DASH_WIDTH as f32;
							self.face_dir = Direction::Left;
						}
					}
				}
				
				// get graphic groups based on brick type
				let dash_graphic_group = match brick_type {
					BrickType::Type1 => { GraphicGroup::Dash1 },
					BrickType::Type2 => { GraphicGroup::Dash2 },
					BrickType::Type3 => { GraphicGroup::Dash3 }
				};
				
				// push dash to lingering graphics
				let dash_graphic = Graphic { g: dash_graphic_group, frame: 0, flags: 0, arg: 0 };
				let mut dash_graphic_x = dash_left_x;
				let mut remaining_dash_distance = dash_right_x - dash_left_x;
				while remaining_dash_distance >= MIN_DASH_WIDTH as f32 {
					self.lingering_graphics.push( LingeringGraphic {
						positioned_graphic: PositionedGraphic::new(dash_graphic, dash_graphic_x, self.bounds.top_y),
						start_t: time_running,
						end_t: time_running + DASH_LINGER_TIME
					});
					dash_graphic_x += MIN_DASH_WIDTH as f32;
					remaining_dash_distance -= MIN_DASH_WIDTH as f32;
				}
				
				// get hitbox
				let left_x;
				let right_x;
				if let Some(ti) = &self.target { 
					match self.hit_dir {
						Direction::Right => {
							left_x = ti.dest_x;
							right_x = ti.post_hit_x + PLAYER_WIDTH as f32 + SLASH_WIDTH as f32;
						},
						Direction::Left => {
							left_x = ti.post_hit_x - SLASH_WIDTH as f32;
							right_x = ti.dest_x + PLAYER_WIDTH as f32;
						}
					}
				} else { 
					match self.hit_dir {
						Direction::Right => {
							left_x = self.bounds.left_x - MIN_DASH_WIDTH as f32;
							right_x = self.bounds.right_x + SLASH_WIDTH as f32;
						},
						Direction::Left => {
							left_x = self.bounds.left_x - SLASH_WIDTH as f32;
							right_x = self.bounds.right_x + MIN_DASH_WIDTH as f32;
						}
					}
				}
				
				let hitbox_bounds = ObjectBounds { 
					left_x, 
					right_x,
					top_y: self.bounds.top_y,
					bottom_y: self.bounds.bottom_y
				};
				
				hitbox = HitBox { bounds: hitbox_bounds, brick_type };
				
				self.state = TaggedState { state: PlayerState::SlashDash, time: time_running };
			},
			_ => panic!()
		}
	
		return hitbox;
	}
	
	pub fn update_hold_positions(&mut self, new_hold_positions: Vec<f32>) {
		self.hold_positions = new_hold_positions;
	}
	
	// accept an input, handle it only if it isn't already down
	pub fn input(&mut self, input: Input, time_running: f32) {
		if self.inputs_down[input as usize] == false {
			self.inputs_down[input as usize] = true;
			match input {
				Input::Dash => { self.input_dash(time_running); },
				Input::Slash1 => { self.input_slash(BrickType::Type1, time_running); },
				Input::Slash2 => { self.input_slash(BrickType::Type2, time_running); },
				Input::Slash3 => { self.input_slash(BrickType::Type3, time_running); }
			}
		}
	}
	
	pub fn end_input(&mut self, input: Input) {
		self.inputs_down[input as usize] = false;
		if let Some(hit_type) = self.hit_type {
			match (input, hit_type) {
				(Input::Slash1, BrickType::Type1) => {
					self.stop_hold = true;
					self.hold_positions = Vec::new();
				},
				(Input::Slash2, BrickType::Type2) => {
					self.stop_hold = true;
					self.hold_positions = Vec::new();
				},
				(Input::Slash3, BrickType::Type3) => {
					self.stop_hold = true;
					self.hold_positions = Vec::new();
				}
				_ => ()
			}
		}
	}

	// inputs a slash command, updating player state
	fn input_slash (&mut self, brick_type: BrickType, time_running: f32) {
		match self.state.state {
			PlayerState::PreSlash => (),
			PlayerState::PreDash => {
				self.stop_hold = false;
				self.state = TaggedState { time: self.state.time, state: PlayerState::PreSlashDash };
				self.hit_type = Some(brick_type);
			},
			_ => {
				self.stop_hold = false;
				self.state = TaggedState {time: time_running, state: PlayerState::PreSlash};
				self.hit_type = Some(brick_type);
			}
		}
	}
	
	// inputs a dash command, updating player state
	fn input_dash (&mut self, time_running: f32) {
		match self.state.state {
			PlayerState::PreDash => (),
			PlayerState::PreSlash => {
				self.state = TaggedState {time: self.state.time, state: PlayerState::PreSlashDash};
			}
			_ => {
				self.state = TaggedState {time: time_running, state: PlayerState::PreDash};
			}
		}
	}
	
	pub fn hold_hitboxes(&self) -> Vec<HitBox> {
		let mut hitboxes = Vec::new();
		if let PlayerState::Hold = self.state.state {
			let brick_type = if let Some(bt) = self.hit_type {bt} else {panic!()};
			
			if self.hold_positions.len() > 0 {
				for hp in &self.hold_positions {
					let bounds = ObjectBounds { 
						left_x: *hp, 
						right_x: *hp + HOLD_HITBOX_WIDTH as f32, 
						top_y: GROUND_POS,
						bottom_y: GROUND_POS + HOLD_HITBOX_HEIGHT as f32
					};
					
					hitboxes.push(HitBox { bounds, brick_type });
				}
			} else {
				let hitbox_x = match self.hit_dir {
					Direction::Left => {self.bounds.left_x - BRICK_WIDTH as f32 / 2.0 - HOLD_HITBOX_WIDTH as f32 / 2.0},
					Direction::Right => {self.bounds.right_x + BRICK_WIDTH as f32 / 2.0 - HOLD_HITBOX_WIDTH as f32 / 2.0}
				};
				
				let bounds = ObjectBounds { 
					left_x: hitbox_x, 
					right_x: hitbox_x + HOLD_HITBOX_WIDTH as f32, 
					top_y: GROUND_POS,
					bottom_y: GROUND_POS + HOLD_HITBOX_HEIGHT as f32
				};
				
				hitboxes.push(HitBox { bounds, brick_type });
			}
		}

		return hitboxes;
	}
	
	// boost from current position to next to target, if close enough
	fn boost(&mut self, time_running: f32) {
		let target = if let Some(t) = &self.target { t } else { return; };
		
		// if within range where boost is reasonable, then boost
		let pos_difference = target.dest_x - self.bounds.left_x;
		if pos_difference > 0.0 {
			let graphic = Graphic{ g: GraphicGroup::Running, frame: frame_number(time_running - self.state.time), flags: 0, arg: 0 };
			let mut rendering_instruction = PositionedGraphic::new(graphic, self.bounds.left_x, self.bounds.top_y);
			let mut remaining_pos_difference = pos_difference;
			
			while remaining_pos_difference > BOOST_GRAPHIC_OFFSET { 
				let mut positioned_graphic = rendering_instruction.clone();
				positioned_graphic.g.g = GraphicGroup::Running;
				self.lingering_graphics.push( LingeringGraphic {
					positioned_graphic: positioned_graphic,
					start_t: time_running - BOOST_PRELINGER_TIME,
					end_t: time_running + BOOST_LINGER_TIME
				});
				remaining_pos_difference -= BOOST_GRAPHIC_OFFSET;
				rendering_instruction.x += BOOST_GRAPHIC_OFFSET;
			}
			
			self.bounds.left_x = target.dest_x;
			self.bounds.right_x = target.dest_x + PLAYER_WIDTH as f32;
			self.hit_dir = target.hit_dir;
		} else if pos_difference < 0.0 {
			let graphic = Graphic{ g: GraphicGroup::Running, frame: frame_number(time_running - self.state.time), flags: GraphicFlags::HorizontalFlip as u8, arg: 0 };
			let mut rendering_instruction = PositionedGraphic::new(graphic, self.bounds.left_x, self.bounds.top_y);
			let mut remaining_pos_difference = -pos_difference;
			
			while remaining_pos_difference > BOOST_GRAPHIC_OFFSET { 
				let mut positioned_graphic = rendering_instruction.clone();
				positioned_graphic.g.g = GraphicGroup::Running;
				self.lingering_graphics.push( LingeringGraphic {
					positioned_graphic: positioned_graphic,
					start_t: time_running - BOOST_PRELINGER_TIME,
					end_t: time_running + BOOST_LINGER_TIME
				});
				remaining_pos_difference -= BOOST_GRAPHIC_OFFSET;
				rendering_instruction.x -= BOOST_GRAPHIC_OFFSET;
			}
			
			self.bounds.left_x = target.dest_x;
			self.bounds.right_x = target.dest_x + PLAYER_WIDTH as f32;
			self.hit_dir = target.hit_dir;
		}
	}
	
	// runs to the target, may boost
	fn move_player(&mut self, seconds_passed: f32, game_data: &GameData) -> PlayerState {
		let mut state;

		match self.target.clone() {
			None => {
				state = PlayerState::Standing;
			},
			Some(ti) => {
				// move based on target info
				let time_to_target = ti.hittable_time - game_data.time_running;
				let pos_difference = self.bounds.left_x - ti.dest_x;
				let distance_to_target;
				let target_dir;
				if pos_difference > 0.0 {
					distance_to_target = pos_difference;
					target_dir = Direction::Left;
				} else {
					distance_to_target = -pos_difference;
					target_dir = Direction::Right;
				}
				let move_speed;

				// either dash, boost, run, or walk
				if !ti.dash_to_target && ti.hittable_time <= game_data.time_running {
					if distance_to_target != 0.0 {
						self.boost(game_data.time_running);
					}

					move_speed = 0.0;
					state = PlayerState::Standing;
				}
				else if distance_to_target > time_to_target * WALK_SPEED {
					move_speed = RUN_SPEED;
					state = PlayerState::Running;
				}
				else {
					move_speed = WALK_SPEED;
					state = PlayerState::Walking;
				}

				// move left or right
				let mut end_x = self.bounds.left_x;
				match target_dir {
					Direction::Left => {
						end_x -= move_speed * seconds_passed;
						if end_x <= ti.dest_x {
							end_x = ti.dest_x;
							self.hit_dir = ti.hit_dir;
							self.face_dir = ti.hit_dir;
							state = PlayerState::Standing;
						}
						else {
							self.face_dir = Direction::Left;
						}
					},
					Direction::Right => {
						end_x += move_speed * seconds_passed;
						if end_x >= ti.dest_x {
							end_x = ti.dest_x;
							self.hit_dir = ti.hit_dir;
							self.face_dir = ti.hit_dir;
							state = PlayerState::Standing;
						}
						else {
							self.face_dir = Direction::Right;
						}
					}
				}

				self.bounds.left_x = end_x;
				self.bounds.right_x = self.bounds.left_x + PLAYER_WIDTH as f32;
			}
		}

		if self.bounds.left_x < LEFT_BOUNDARY { self.bounds.left_x = 0.0 };
		if self.bounds.left_x > RIGHT_BOUNDARY { self.bounds.left_x = RIGHT_BOUNDARY - PLAYER_WIDTH as f32 };
		self.bounds.right_x = self.bounds.left_x + PLAYER_WIDTH as f32;

		return state;
	}
	
	// updates the state and performs any other consequent updates to player position or lingering graphics
	fn update_state(&mut self, seconds_passed: f32, game_data: &GameData) {
		let time_running = game_data.time_running;
		let t = self.state.time;
		match self.state.state {
			PlayerState::Standing => {
				let new_state = self.move_player(seconds_passed, game_data);
				self.state = TaggedState { state: new_state, time: t };
				return;
			},
			PlayerState::Walking => {
				let new_state = self.move_player(seconds_passed, game_data);
				self.state = TaggedState { state: new_state, time: t };
				return;
			},
			PlayerState::Running => {
				let new_state = self.move_player(seconds_passed, game_data);
				self.state = TaggedState { state: new_state, time: t };
				return;
			},
			PlayerState::PreSlash => {
				if time_running - t > PRE_SLASH_TIME {
					panic!(); // pre slashes turning into slashes should be handled in action_tick
				}
				return;
			},
			PlayerState::PreDash => {
				if time_running - t <= PRE_SLASH_TIME {
					return;
				}
				
				// boost if in range of target
				if let Some(ti) = &self.target {
					let pos_difference = self.bounds.left_x - ti.dest_x;
					if pos_difference > -MAX_BOOST_DISTANCE && pos_difference < MAX_BOOST_DISTANCE && !ti.dash_to_target {
						self.boost(time_running);
					}
				}
				
				let dash_graphic_group = GraphicGroup::Dash0;

				let dash_left_x;
				let dash_right_x;
				match self.hit_dir {
					Direction::Right => {
						self.bounds.left_x = self.bounds.left_x + MIN_DASH_WIDTH as f32;
						self.bounds.right_x = self.bounds.left_x + PLAYER_WIDTH as f32;
						dash_left_x = self.bounds.left_x - MIN_DASH_WIDTH as f32;
						dash_right_x = self.bounds.left_x;
						self.face_dir = Direction::Right;
					},
					Direction::Left => {
						self.bounds.left_x = self.bounds.left_x - MIN_DASH_WIDTH as f32;
						self.bounds.right_x = self.bounds.left_x + PLAYER_WIDTH as f32;
						dash_left_x = self.bounds.right_x;
						dash_right_x = self.bounds.right_x + MIN_DASH_WIDTH as f32;
						self.face_dir = Direction::Left;
					}
				}
				
				// push dash to lingering graphics
				let dash_graphic = Graphic { g: dash_graphic_group, frame: 0, flags: 0, arg: 0 };
				let mut dash_graphic_x = dash_left_x;
				let mut remaining_dash_distance = dash_right_x - dash_left_x;
				while remaining_dash_distance >= MIN_DASH_WIDTH as f32 {
					self.lingering_graphics.push( LingeringGraphic {
						positioned_graphic: PositionedGraphic::new(dash_graphic, dash_graphic_x, self.bounds.top_y),
						start_t: time_running,
						end_t: time_running + DASH_LINGER_TIME
					});
					dash_graphic_x += MIN_DASH_WIDTH as f32;
					remaining_dash_distance -= MIN_DASH_WIDTH as f32;
				}
				
				self.state = TaggedState { state: PlayerState::Dash, time: time_running };
				return;
			},
			PlayerState::PreSlashDash => {
				if time_running - t > PRE_SLASH_TIME {
					panic!(); // preSlashDash turning into slashDash should be handled in action_tick
				}

				return;
			},
			PlayerState::Slash => {
				
				if self.hold_positions.len() > 0 && !self.stop_hold {
					self.state = TaggedState {state:PlayerState::Hold, time: time_running};
					return;
				}
				
				self.face_dir = self.hit_dir;
				self.state = TaggedState { state: PlayerState::PostSlash, time: time_running };
				return;
			},
			PlayerState::Dash => {	
				self.state = TaggedState { state: PlayerState::Standing, time: time_running };
				return;
			},
			PlayerState::SlashDash => {
				
				if self.hold_positions.len() > 0 && !self.stop_hold {
					self.state = TaggedState {state:PlayerState::Hold, time: time_running};
					return;
				}
				
				self.face_dir = self.hit_dir;
				self.state = TaggedState { state: PlayerState::PostSlash, time: time_running };
				return;
			},
			PlayerState::PostSlash => {
				if self.hold_positions.len() > 0 && !self.stop_hold {
					self.state = TaggedState {state:PlayerState::Hold, time: time_running};
					return;
				}
				
				let time_difference = time_running - t;
				if time_difference > POST_SLASH_TIME {
					if !self.stop_hold  && time_difference > PRE_HOLD_TIME {
						self.state = TaggedState {state:PlayerState::Hold, time: time_running};
						return;
					} else if self.stop_hold {
						self.state = TaggedState { state: PlayerState::Standing, time: time_running };
						self.hit_type = None;						
					}
					
				}

				self.face_dir = self.hit_dir;
				return;
			},
			PlayerState::Hold => {
				if let Some(ht) = self.hit_type {
					if !self.stop_hold {
						return;
					}
				} 

				self.face_dir = self.hit_dir;
				self.state = TaggedState { state: PlayerState::Standing, time: time_running };
				return;
			}
		}
	}
	
	// rendering instruction for the player
	pub fn rendering_instructions(&self, time_running: f32) -> Vec<PositionedGraphic> {
		let flags = match self.face_dir {
			Direction::Right => 0,
			Direction::Left => GraphicFlags::HorizontalFlip as u8
		};
		let arg = 0;
		let mut positioned_graphics = Vec::with_capacity(64);
		
		let t = self.state.time;
		match self.state.state {
			PlayerState::Standing | PlayerState::PreDash | PlayerState::Dash => {
				let graphic_group = GraphicGroup::Standing;
				let graphic = Graphic { g: graphic_group, frame: 0, flags, arg };
				positioned_graphics.push(PositionedGraphic::new(graphic, self.bounds.left_x, self.bounds.top_y));
			},
			PlayerState::Walking => {
				let graphic_group = GraphicGroup::Walking;
				let frame = frame_number(time_running - t);
				let graphic = Graphic { g: graphic_group, frame, flags, arg };
				positioned_graphics.push(PositionedGraphic::new(graphic, self.bounds.left_x, self.bounds.top_y));
			}
			PlayerState::Running => {
				let graphic_group = GraphicGroup::Running;
				let frame = frame_number(time_running - t);
				let graphic = Graphic { g: graphic_group, frame, flags, arg };
				positioned_graphics.push(PositionedGraphic::new(graphic, self.bounds.left_x, self.bounds.top_y));
			},
			PlayerState::PreSlash | PlayerState::PreSlashDash => {
				let brick_type = if let Some(bt) = self.hit_type { bt } else { panic!() };
				let graphic_group = match brick_type {
					BrickType::Type1 => GraphicGroup::Slashing1,
					BrickType::Type2 => GraphicGroup::Slashing2,
					BrickType::Type3 => GraphicGroup::Slashing3
				};
				let frame = frame_number(time_running - t);
				
				let graphic = Graphic { g: graphic_group, frame, flags, arg };
				positioned_graphics.push(PositionedGraphic::new(graphic, self.bounds.left_x, self.bounds.top_y));
			},
			PlayerState::Slash | PlayerState::SlashDash => {
				let brick_type = if let Some(bt) = self.hit_type { bt } else { panic!() };
				let graphic_group = match brick_type {
					BrickType::Type1 => GraphicGroup::Slashing1,
					BrickType::Type2 => GraphicGroup::Slashing2,
					BrickType::Type3 => GraphicGroup::Slashing3
				};
				let frame = frame_number(time_running - t + PRE_SLASH_TIME);
				
				let graphic = Graphic { g: graphic_group, frame, flags, arg };
				positioned_graphics.push(PositionedGraphic::new(graphic, self.bounds.left_x, self.bounds.top_y));
			},
			PlayerState::PostSlash => {
				let brick_type = if let Some(bt) = self.hit_type { bt } else { panic!() };
				let graphic_group = match brick_type {
					BrickType::Type1 => GraphicGroup::Slashing1,
					BrickType::Type2 => GraphicGroup::Slashing2,
					BrickType::Type3 => GraphicGroup::Slashing3
				};
				
				if time_running - t < POST_SLASH_TIME {
					let frame = frame_number(time_running - t + PRE_SLASH_TIME);
					let graphic = Graphic { g: graphic_group, frame, flags, arg };
					positioned_graphics.push(PositionedGraphic::new(graphic, self.bounds.left_x, self.bounds.top_y));
				} else {
					let frame = frame_number(POST_SLASH_TIME + PRE_SLASH_TIME);
					let graphic = Graphic { g: graphic_group, frame, flags, arg };
					positioned_graphics.push(PositionedGraphic::new(graphic, self.bounds.left_x, self.bounds.top_y));
				}
			},
			PlayerState::Hold => {
				let brick_type = if let Some(bt) = self.hit_type { bt } else { panic!() };
				
				// push player graphic
				let graphic_group = match brick_type {
					BrickType::Type1 => GraphicGroup::Holding1,
					BrickType::Type2 => GraphicGroup::Holding2,
					BrickType::Type3 => GraphicGroup::Holding3
				};
				
				let graphic = Graphic { g: graphic_group, frame: 0, flags, arg };
				positioned_graphics.push(PositionedGraphic::new(graphic, self.bounds.left_x, self.bounds.top_y));
				
				// push hold hitbox graphics
				let hitbox_graphic_group = match brick_type {
					BrickType::Type1 => GraphicGroup::Hold1,
					BrickType::Type2 => GraphicGroup::Hold2,
					BrickType::Type3 => GraphicGroup::Hold3
				};
				
				if self.hold_positions.len() > 0 {
					for hp in &self.hold_positions {
						let graphic = Graphic {g: hitbox_graphic_group, frame: 0, flags: 0, arg: 0};
						positioned_graphics.push(PositionedGraphic::new(graphic, *hp, GROUND_POS));
					}
				} else {
					let hitbox_graphic_x = match self.hit_dir {
						Direction::Left => {self.bounds.left_x - BRICK_WIDTH as f32 / 2.0 - HOLD_HITBOX_WIDTH as f32 / 2.0},
						Direction::Right => {self.bounds.right_x + BRICK_WIDTH as f32 / 2.0 - HOLD_HITBOX_WIDTH as f32 / 2.0}
					};
					let hitbox_graphic = Graphic {g: hitbox_graphic_group, frame: 0, flags: 0, arg: 0};
					positioned_graphics.push(PositionedGraphic::new(hitbox_graphic, hitbox_graphic_x, self.bounds.bottom_y));
				}
			}
		}
		
		return positioned_graphics;
	}
	
	// rendering instruction for any lingering graphics
	pub fn lg_rendering_instructions(&self, time_running: f32) -> Vec<PositionedGraphic> {
		let mut positioned_graphics = Vec::new();
		for lg in &self.lingering_graphics {
			
			let mut pg = lg.positioned_graphic.clone();
			let proportion_time_passed = (time_running - lg.start_t) / (lg.end_t - lg.start_t);
			
			pg.g.flags |= GraphicFlags::Opacity as u8;
			pg.g.arg = 255 - (proportion_time_passed * 255.0) as u8;
			
			if lg.end_t < time_running {
				pg.g.arg = 0;
			}
			
			positioned_graphics.push(pg);
		}
		return positioned_graphics;
	}
}