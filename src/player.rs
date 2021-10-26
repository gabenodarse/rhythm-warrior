
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

use crate::GROUND_POS;
use crate::GAME_WIDTH;
use crate::F32_ZERO;
use crate::GameData;
use crate::objects::PLAYER_WIDTH;
use crate::objects::PLAYER_HEIGHT;
use crate::objects::BRICK_WIDTH;
use crate::objects::MIN_DASH_WIDTH;
use crate::objects::SLASH_WIDTH;
use crate::objects::HOLD_HITBOX_WIDTH;
use crate::objects::HOLD_HITBOX_HEIGHT;

// delays dash/slash by a tiny amount so they can be pressed at the same time. starts snimation during delay
const PRE_SLASH_TIME: f32 = 0.06; 
const PRE_HOLD_TIME: f32 = 0.06;
// so slash animation can finish
const POST_SLASH_TIME: f32 = 0.06;
const DASH_LINGER_TIME: f32 = 0.3; // how long the dash graphic lingers
const BOOST_LINGER_TIME: f32 = 0.3;
const BOOST_PRELINGER_TIME: f32 = 1.2;
const MAX_BOOST_DISTANCE: f32 = 4.0 * PLAYER_WIDTH as f32;
const BOOST_GRAPHIC_OFFSET: f32 = PLAYER_WIDTH as f32 / 2.0; // how close the boost graphics are to one another

pub struct Player {
	state: TaggedState,
	bounds: ObjectBounds,
	
	inputs_down: [bool; Input::Slash3 as usize + 1], // >:< this way of getting the size of the enum is flimsy
	target: Option<TargetInfo>,
	face_dir: Direction,
	hit_dir: Direction,
	
	hit_type: Option<BrickType>,
	
	lingering_graphics: Vec<LingeringGraphic>
}

enum PlayerState {
	Running,
	Walking,
	PreSlash,
	PreDash,
	PreSlashDash,
	Slash, 
	Dash,
	SlashDash,
	PostSlash,
	PreHold(bool), // bool stating whether it's a single or multiple hold
	Hold(bool), 
}

struct TaggedState {
	// init time. so there can be a delay before becoming active/inactive and for animation frame calculation
	time: f32,
	state: PlayerState
}

struct TargetInfo {
	time: f32,
	pos: f32, // where the player left_x should be
	hit_dir: Direction,
	dash_distance: f32,
	target_centers: Vec<f32>,
}

impl Object for Player {
	fn bounds(&self) -> ObjectBounds {
		self.bounds
	}
}

impl Player {
	
	pub fn new(x: f32) -> Player {
		Player {
			state: TaggedState { time: 0.0, state: PlayerState::Walking },
			bounds: ObjectBounds {
				left_x: x,
				top_y: GROUND_POS as f32 - PLAYER_HEIGHT as f32,
				right_x: x + PLAYER_WIDTH as f32, 
				bottom_y: GROUND_POS as f32
			},
			
			inputs_down: [false; Input::Slash3 as usize + 1],
			face_dir: Direction::Right,
			hit_dir: Direction::Right,
			target: None,
			
			hit_type: None,
			
			lingering_graphics: Vec::new() // graphics for objects no longer present but still showing, e.g. slashes/dashes that have executed
		}
	}
	
	// tick the player's state
	pub fn tick(&mut self, seconds_passed: f32, bricks_iter: vec_deque::Iter<Brick>, game_data: &GameData) {
		self.update_state(game_data.time_running);
		// update target info if not holding
		match self.state.state {
			PlayerState::PreHold(_) | PlayerState::Hold(_) => (),
			_ => {
				self.update_target_info(bricks_iter, game_data.time_running, game_data.brick_speed);
				self.move_player(seconds_passed, game_data.time_running);
			}
		}
		
		// TODO would prefer if cloning the lingering graphics before removing them was unnecessary
		let new_set: Vec<LingeringGraphic> = self.lingering_graphics.iter().filter(|lg| lg.end_t > game_data.time_running).cloned().collect();
		self.lingering_graphics = new_set;
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
	}
	
	// inputs a slash command, updating player state
	pub fn input_slash (&mut self, brick_type: BrickType, time_running: f32) {
		match self.state.state {
			PlayerState::PreSlash => (),
			PlayerState::PreDash => {
				self.state = TaggedState { time: self.state.time, state: PlayerState::PreSlashDash };
				self.hit_type = Some(brick_type);
			},
			_ => {
				self.state = TaggedState {time: time_running, state: PlayerState::PreSlash};
				self.hit_type = Some(brick_type);
			}
		}
	}
	
	// inputs a dash command, updating player state
	pub fn input_dash (&mut self, time_running: f32) {
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
	
	pub fn hitbox (&self) -> Option<HitBox> {
		match self.state.state {
			PlayerState::Walking | PlayerState::Running | PlayerState::PreSlash | PlayerState::PreSlashDash
			| PlayerState::PreDash | PlayerState::Dash | PlayerState::PostSlash | PlayerState::PreHold(_) => {
				return None;
			},
			PlayerState::Slash => {
				let brick_type = if let Some(bt) = self.hit_type {bt} else {panic!()};
				let hitbox_x = match self.hit_dir {
					Direction::Right => self.bounds.right_x,
					Direction::Left => self.bounds.left_x - SLASH_WIDTH as f32
				};
				
				let bounds = ObjectBounds { 
					left_x: hitbox_x, 
					right_x: hitbox_x + SLASH_WIDTH as f32, 
					top_y: self.bounds.top_y,
					bottom_y: self.bounds.bottom_y
				};
				return Some(HitBox { bounds, brick_type });
			},
			PlayerState::SlashDash => {
				let brick_type = if let Some(bt) = self.hit_type {bt} else {panic!()};
				let dash_distance = if let Some(ti) = &self.target { ti.dash_distance } else { MIN_DASH_WIDTH as f32 };
				let hitbox_x = match self.hit_dir {
					Direction::Right => self.bounds.left_x - dash_distance,
					Direction::Left => self.bounds.left_x - SLASH_WIDTH as f32
				};
				
				let bounds = ObjectBounds { 
					left_x: hitbox_x, 
					right_x: hitbox_x + PLAYER_WIDTH as f32 + SLASH_WIDTH as f32 + dash_distance as f32,
					top_y: self.bounds.top_y,
					bottom_y: self.bounds.bottom_y
				};
				return Some(HitBox { bounds, brick_type });
			}
			PlayerState::Hold(multi) => {
				let brick_type = if let Some(bt) = self.hit_type {bt} else {panic!()};
				let hitbox_x = match self.hit_dir {
					Direction::Left => {self.bounds.left_x - BRICK_WIDTH as f32 / 2.0 - HOLD_HITBOX_WIDTH as f32 / 2.0},
					Direction::Right => {self.bounds.right_x + BRICK_WIDTH as f32 / 2.0 - HOLD_HITBOX_WIDTH as f32 / 2.0}
				};
				let bounds = ObjectBounds { 
					left_x: hitbox_x, 
					right_x: hitbox_x + HOLD_HITBOX_WIDTH as f32, 
					top_y: self.bounds.bottom_y,
					bottom_y: self.bounds.bottom_y + HOLD_HITBOX_HEIGHT as f32
				};
				return Some(HitBox { bounds, brick_type });
			},
		};
	}
	
	// boost from current position to next to target, if close enough
	fn boost(&mut self, time_running: f32) {
		let target = if let Some(t) = &self.target { t } else { return; };
		
		// if within range where boost is reasonable, then boost
		let pos_difference = target.pos - self.bounds.left_x;
		if pos_difference < MAX_BOOST_DISTANCE && pos_difference > 0.0 {
			let graphic = Graphic{ g: GraphicGroup::Running, frame: frame_number(time_running - self.state.time), flags: 0, arg: 0 };
			let mut rendering_instruction = PositionedGraphic{ g: graphic, x: self.bounds.left_x, y: self.bounds.top_y };
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
			
			self.bounds.left_x = target.pos;
			self.bounds.right_x = target.pos + PLAYER_WIDTH as f32;
			self.hit_dir = target.hit_dir;
		} else if pos_difference > -MAX_BOOST_DISTANCE && pos_difference < 0.0 {
			let graphic = Graphic{ g: GraphicGroup::Running, frame: frame_number(time_running - self.state.time), flags: GraphicFlags::HorizontalFlip as u8, arg: 0 };
			let mut rendering_instruction = PositionedGraphic{ g: graphic, x: self.bounds.left_x, y: self.bounds.top_y };
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
			
			self.bounds.left_x = target.pos;
			self.bounds.right_x = target.pos + PLAYER_WIDTH as f32;
			self.hit_dir = target.hit_dir;
		}
	}
	
	// updates target, face_dir, and hit_dir
	fn update_target_info(&mut self, bricks_iter: vec_deque::Iter<Brick>, time_running: f32, brick_speed: f32) {
		
		const SAME_GROUP_TIME_BUFFER: f32 = 0.025; // maximum time difference between bricks in the same group 
			// (bricks in same group should have time difference of 0. can do validation checking of groups when loading song)
		// maximum time player will wait before chasing next bricks
			// !!! should be variable based on how fast bricks are travelling
		let next_bricks_time_buffer:  f32 = PLAYER_HEIGHT as f32 / brick_speed;
		
		let mut bricks_info = None;
		let mut target_centers = Vec::new();
		
		struct UpcomingBricks {
			time: f32,
			left_brick: f32,
			right_brick: f32
		}
		
		for brick in bricks_iter {
			// iterate over the bricks that are already passed
			if brick.time + next_bricks_time_buffer < time_running {
				continue;
			} 
			
			// get the group of UpcomingBricks
			match &mut bricks_info {
				None => {
					target_centers.push(brick.bounds.left_x + BRICK_WIDTH as f32 / 2.0);
					bricks_info = Some( UpcomingBricks {
						time: brick.time,
						left_brick: brick.bounds.left_x,
						right_brick: brick.bounds.left_x
					});
				},
				Some(bi) => {
					if bi.time + SAME_GROUP_TIME_BUFFER < brick.time {
						break; 
					}
					
					target_centers.push(brick.bounds.left_x + BRICK_WIDTH as f32 / 2.0);
					if brick.bounds.left_x < bi.left_brick {
						bi.left_brick = brick.bounds.left_x;
					} else if brick.bounds.left_x > bi.right_brick {
						bi.right_brick = brick.bounds.left_x;
					}
				}
			}
		}
		
		match bricks_info {
			None => {
				self.target = None;
			}
			Some(bi) => {
				let left_target = bi.left_brick - PLAYER_WIDTH as f32;
				let right_target = bi.right_brick + BRICK_WIDTH as f32;
				let target_difference = right_target - left_target;
				let dash_distance = target_difference - BRICK_WIDTH as f32 - PLAYER_WIDTH as f32; // before the last brick, not after it
				let dash_distance = if dash_distance > MIN_DASH_WIDTH as f32 { dash_distance } else { MIN_DASH_WIDTH as f32 };
				
				let pos;
				let hit_dir;
				// if left of target, right of target, in between targets
				if left_target - self.bounds.left_x >= 0.0 {
					self.face_dir = Direction::Right;
						pos = left_target;
						hit_dir = Direction::Right;
				} else if self.bounds.left_x - right_target >= 0.0 {
					self.face_dir = Direction::Left;
						pos = right_target;
						hit_dir = Direction::Left;
				} else if left_target - self.bounds.left_x > self.bounds.left_x - right_target {
					self.face_dir = Direction::Left;
						pos = left_target;
						hit_dir = Direction::Right;
				} else {
					self.face_dir = Direction::Right;
						pos = right_target;
						hit_dir = Direction::Left;
				}
				
				self.target = Some( TargetInfo { time: bi.time, pos, hit_dir, dash_distance, target_centers } );
				self.hit_dir = self.face_dir;
			}
		}
	}
	
	// runs to the target, may boost
	fn move_player(&mut self, seconds_passed: f32, time_running: f32) {
		const RUN_SPEED: f32 = 480.0; // in pixels per second
		
		match &self.target {
			None => {
				if self.bounds.left_x < 0.0 { self.bounds.left_x = 0.0 };
				if self.bounds.left_x > GAME_WIDTH as f32 { self.bounds.left_x = GAME_WIDTH as f32 - PLAYER_WIDTH as f32 };
				self.bounds.right_x = self.bounds.left_x + PLAYER_WIDTH as f32;
			},
			Some(ti) => {
				match self.face_dir {
					Direction::Left => {
						let end_pos = self.bounds.left_x - seconds_passed * RUN_SPEED;
						self.bounds.left_x = if end_pos < ti.pos { ti.pos } else { end_pos };
					},
					Direction::Right => {
						let end_pos = self.bounds.left_x + seconds_passed * RUN_SPEED;
						self.bounds.left_x = if end_pos > ti.pos { ti.pos } else { end_pos };
					}
				}
				
				self.bounds.right_x = self.bounds.left_x + PLAYER_WIDTH as f32;
			}
		}
	}
	
	// updates the state and performs any other consequent updates
	fn update_state(&mut self, time_running: f32) {
		let t = self.state.time;
		match self.state.state {
			PlayerState::Running => {
				// run or walk depending on if the target is reached
				match &self.target {
					None => self.state = TaggedState { state: PlayerState::Walking, time: time_running },
					Some(ti) => {
						if ti.pos - self.bounds.left_x < F32_ZERO && ti.pos - self.bounds.left_x > -F32_ZERO {
							self.state = TaggedState { state: PlayerState::Walking, time: time_running };
						} 
					}
				}
				return;
			},
			PlayerState::Walking => {
				match &self.target {
					// run or walk depending if the target is reached
					None => (),
					Some(ti) => {
						if ti.pos - self.bounds.left_x > F32_ZERO || ti.pos - self.bounds.left_x < -F32_ZERO {
							self.state = TaggedState { state: PlayerState::Running, time: time_running };
						}
					}
				}
				return;
			},
			PlayerState::PreSlash => {
				if time_running - t > PRE_SLASH_TIME {
					self.boost(time_running);
					self.state = TaggedState { state: PlayerState::Slash, time: time_running };
					return;
				}
				return;
			},
			PlayerState::PreDash => {
				if time_running - t > PRE_SLASH_TIME {
					self.boost(time_running);
					
					let mut dash_graphic_x;
					let dash_graphic_group = GraphicGroup::Dash0;
					let dash_distance = if let Some(ti) = &self.target { ti.dash_distance } else { MIN_DASH_WIDTH as f32 };
					
					// fill variables based on hit dir and push dash
					match self.hit_dir {
						Direction::Right => {
							dash_graphic_x = self.bounds.right_x;
							self.bounds.left_x = self.bounds.right_x + dash_distance as f32;
							self.bounds.right_x = self.bounds.left_x + PLAYER_WIDTH as f32;
						},
						Direction::Left => {
							self.bounds.right_x = self.bounds.left_x - dash_distance as f32;
							self.bounds.left_x = self.bounds.right_x - PLAYER_WIDTH as f32;
							dash_graphic_x = self.bounds.right_x;
						}
					}
					
					// push dash to lingering graphics
					let dash_graphic = Graphic { g: dash_graphic_group, frame: 0, flags: 0, arg: 0 };
					let mut remaining_dash_distance = dash_distance;
					while remaining_dash_distance >= MIN_DASH_WIDTH as f32 {
						self.lingering_graphics.push( LingeringGraphic {
							positioned_graphic: PositionedGraphic { g: dash_graphic, x: dash_graphic_x, y: self.bounds.top_y },
							start_t: time_running,
							end_t: time_running + DASH_LINGER_TIME
						});
						dash_graphic_x += MIN_DASH_WIDTH as f32;
						remaining_dash_distance -= MIN_DASH_WIDTH as f32;
					}
					
					self.state = TaggedState { state: PlayerState::Dash, time: time_running };
					return;
				}
			},
			PlayerState::PreSlashDash => {
				if time_running - t > PRE_SLASH_TIME {
					self.boost(time_running);
					
					let brick_type = if let Some(bt) = self.hit_type { bt } else { panic!() };
					
					let mut dash_graphic_x;
					let dash_distance = if let Some(ti) = &self.target { ti.dash_distance } else { MIN_DASH_WIDTH as f32 };
					
					// fill variables based on hit dir
					match self.hit_dir {
						Direction::Right => {
							dash_graphic_x = self.bounds.right_x;
							self.bounds.left_x = self.bounds.right_x + dash_distance as f32;
							self.bounds.right_x = self.bounds.left_x + PLAYER_WIDTH as f32;
						},
						Direction::Left => {
							self.bounds.right_x = self.bounds.left_x - dash_distance as f32;
							self.bounds.left_x = self.bounds.right_x - PLAYER_WIDTH as f32;
							dash_graphic_x = self.bounds.right_x as f32;
						}
					}
					
					// get graphic groups based on brick type
					let dash_graphic_group = match brick_type {
						BrickType::Type1 => { GraphicGroup::Dash1 },
						BrickType::Type2 => { GraphicGroup::Dash2 },
						BrickType::Type3 => { GraphicGroup::Dash3 }
					};
					
					let dash_graphic = Graphic { g: dash_graphic_group, frame: 0, flags: 0, arg: 0 };
					
					// push dash to lingering graphics
					let mut remaining_dash_distance = dash_distance;
					while remaining_dash_distance >= MIN_DASH_WIDTH as f32 {
						self.lingering_graphics.push( LingeringGraphic {
							positioned_graphic: PositionedGraphic { g: dash_graphic, x: dash_graphic_x, y: self.bounds.top_y },
							start_t: time_running,
							end_t: time_running + DASH_LINGER_TIME
						});
						dash_graphic_x += MIN_DASH_WIDTH as f32;
						remaining_dash_distance -= MIN_DASH_WIDTH as f32;
					}
					
					self.state = TaggedState { state: PlayerState::SlashDash, time: time_running };
				}
				return;
			},
			PlayerState::Slash => {
				if let Some(ht) = self.hit_type {
					if self.inputs_down[BrickType::to_input(ht) as usize] {
						self.state = TaggedState {state:PlayerState::PreHold(false), time: time_running};
						return;
					}
				}
				self.state = TaggedState { state: PlayerState::PostSlash, time: time_running };
				return;
			},
			PlayerState::Dash => {				
				self.state = TaggedState { state: PlayerState::Walking, time: time_running };
				return;
			},
			PlayerState::SlashDash => {
				if let Some(ht) = self.hit_type {
					if self.inputs_down[BrickType::to_input(ht) as usize] {
						self.state = TaggedState {state:PlayerState::PreHold(true), time: time_running};
						return;
					}
				}
				self.state = TaggedState { state: PlayerState::PostSlash, time: time_running };
				return;
			},
			PlayerState::PostSlash => {
				if time_running - t > POST_SLASH_TIME {
					self.state = TaggedState { state: PlayerState::Walking, time: time_running };
					self.hit_type = None;
				}
				return;
			},
			PlayerState::PreHold(multi) => {
				if let Some(ht) = self.hit_type {
					if self.inputs_down[BrickType::to_input(ht) as usize] {
						if time_running - t > PRE_HOLD_TIME {
							self.state = TaggedState { state: PlayerState::Hold(multi), time: time_running };
						}
						return;
					}
				} 
				self.state = TaggedState { state: PlayerState::Walking, time: time_running };
				return;
			},
			PlayerState::Hold(_) => {
				if let Some(ht) = self.hit_type {
					if self.inputs_down[BrickType::to_input(ht) as usize] {
						return;
					}
				} 
				self.state = TaggedState { state: PlayerState::Walking, time: time_running };
				return;
			}
		}
	}
	
	// rendering instruction for the player
	pub fn rendering_instructions(&self, time_running: f32) -> Vec<PositionedGraphic> {
		let graphic_group;
		let frame;
		let flags;
		let arg = 0;
		let mut x = self.bounds.left_x;
		let mut positioned_graphics = Vec::with_capacity(32);
		
		let t = self.state.time;
		match self.state.state {
			PlayerState::Running => {
				graphic_group = GraphicGroup::Running;
				frame = frame_number(time_running - t);
			},
			PlayerState::PreSlash | PlayerState::PreSlashDash => {
				let brick_type = if let Some(bt) = self.hit_type { bt } else { panic!() };
				graphic_group = match brick_type {
					BrickType::Type1 => GraphicGroup::Slashing1,
					BrickType::Type2 => GraphicGroup::Slashing2,
					BrickType::Type3 => GraphicGroup::Slashing3
				};
				frame = frame_number(time_running - t);
				match self.face_dir {
					Direction::Right => (),
					Direction::Left => x = self.bounds.left_x - SLASH_WIDTH as f32,
				};
			},
			PlayerState::Slash | PlayerState::SlashDash | PlayerState::PostSlash => {
				let brick_type = if let Some(bt) = self.hit_type { bt } else { panic!() };
				graphic_group = match brick_type {
					BrickType::Type1 => GraphicGroup::Slashing1,
					BrickType::Type2 => GraphicGroup::Slashing2,
					BrickType::Type3 => GraphicGroup::Slashing3
				};
				frame = frame_number(time_running - t + PRE_SLASH_TIME);
				match self.face_dir {
					Direction::Right => (),
					Direction::Left => x = self.bounds.left_x - SLASH_WIDTH as f32,
				};
			},
			PlayerState::Hold(multi) => {
				graphic_group = GraphicGroup::Walking;
				frame = 0;
				
				let brick_type = if let Some(bt) = self.hit_type {bt} else {panic!()};
				let hitbox_graphic_group = match brick_type {
					BrickType::Type1 => GraphicGroup::Hold1,
					BrickType::Type2 => GraphicGroup::Hold2,
					BrickType::Type3 => GraphicGroup::Hold3
				};
				let hitbox_graphic_x = match self.hit_dir {
					Direction::Left => {self.bounds.left_x - BRICK_WIDTH as f32 / 2.0 - HOLD_HITBOX_WIDTH as f32 / 2.0},
					Direction::Right => {self.bounds.right_x + BRICK_WIDTH as f32 / 2.0 - HOLD_HITBOX_WIDTH as f32 / 2.0}
				};
				let hitbox_graphic = Graphic {g: hitbox_graphic_group, frame: 0, flags: 0, arg: 0};
				positioned_graphics.push(PositionedGraphic {g: hitbox_graphic, x: hitbox_graphic_x, y: self.bounds.bottom_y});
			}
			_ => {
				graphic_group = GraphicGroup::Walking;
				frame = 0;
			}
		}
		
		flags = match self.face_dir {
			Direction::Right => 0,
			Direction::Left => GraphicFlags::HorizontalFlip as u8
		};
		
		let graphic = Graphic { g: graphic_group, frame, flags, arg };
		
		positioned_graphics.push( PositionedGraphic { g: graphic, x, y: self.bounds.top_y } );
		
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