
use std::collections::VecDeque;
use std::collections::vec_deque;
use std::collections::btree_set::BTreeSet;

use crate::log;

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

use crate::FRAME_TIME;
use crate::GROUND_POS;
use crate::GAME_WIDTH;
use crate::Song;
use crate::objects::PLAYER_WIDTH;
use crate::objects::PLAYER_HEIGHT;
use crate::objects::BRICK_WIDTH;
use crate::objects::DASH_WIDTH;
use crate::objects::SLASH_WIDTH;

const PRE_SLASH_TIME: f32 = 0.06; // delay dash/slash by a tiny amount so they can be pressed at the same time
const SLASH_TIME: f32 = 0.1;
const DASH_LINGER_TIME: f32 = 0.3; // how long the dash graphic lingers
const BOOST_LINGER_TIME: f32 = 0.2;
const BOOST_PRELINGER_TIME: f32 = 0.04;

pub struct Player {
	graphic_group: GraphicGroup,
	graphic: PositionedGraphic,
	state: PlayerState,
	
	bounds: ObjectBounds,
	
	target: Option<TargetInfo>,
	face_dir: Direction,
	hit_dir: Direction,
	
	hit_type: Option<BrickType>,
	hitbox: Option<HitBox>,
	
	lingering_graphics: Vec<LingeringGraphic>
}

enum PlayerState {
	Running,
	Walking,
	PreSlash(f32), // init time of slash/dash, to insert short delay before becoming active
	PreDash(f32),
	PreSlashDash(f32),
	Slash(f32), // init time so there can be a delay before becoming inactive
	Dash(f32),
	SlashDash(f32),
	PostSlash(f32),
}

struct TargetInfo {
	time: f32,
	pos: f32,
	hit_dir: Direction
}

impl Object for Player {
	
	fn bounds(&self) -> ObjectBounds {
		self.bounds
	}
}

impl Player {
	
	pub fn new(x: f32) -> Player {
		Player {
			graphic_group: GraphicGroup::Walking,
			graphic: PositionedGraphic {
				g: Graphic { g: GraphicGroup::Walking, frame: 0, flags: 0, arg: 0 },
				x,
				y: GROUND_POS as f32 - PLAYER_HEIGHT as f32
			},
			state: PlayerState::Walking,
			
			bounds: ObjectBounds {
				left_x: x,
				top_y: GROUND_POS as f32 - PLAYER_HEIGHT as f32,
				right_x: x + PLAYER_WIDTH as f32, 
				bottom_y: GROUND_POS as f32
			},
			
			face_dir: Direction::Right,
			hit_dir: Direction::Right,
			target: None,
			
			hit_type: None,
			hitbox: None,
			
			lingering_graphics: Vec::new() // graphics for objects no longer present but still showing, e.g. slashes/dashes that have executed
		}
	}
	
	// tick the player's state
	pub fn tick(&mut self, seconds_passed: f32, bricks_iter: vec_deque::Iter<Brick>, time_running: f32, song: &Song) {
		self.update_target_info(bricks_iter, time_running, song.brick_speed);
		self.regular_move(seconds_passed, time_running);
		self.update_state(time_running);
		self.update_graphics(time_running);
	}
	
	// inputs a slash command, updating player state
	pub fn input_slash (&mut self, brick_type: BrickType, time_running: f32) {
		match self.state {
			PlayerState::PreSlash(_) => (),
			PlayerState::PreDash(t) => {
				self.state = PlayerState::PreSlashDash(t);
				self.hit_type = Some(brick_type);
			},
			_ => {
				self.state = PlayerState::PreSlash(time_running);
				self.hit_type = Some(brick_type);
			}
		}
	}
	
	// inputs a dash command, updating player state
	pub fn input_dash (&mut self, time_running: f32) {
		match self.state {
			PlayerState::PreDash(_) => (),
			PlayerState::PreSlash(t) => {
				self.state = PlayerState::PreSlashDash(t);
			}
			_ => {
				self.state = PlayerState::PreDash(time_running);
			}
		}
	}
	
	pub fn hitbox (&self) -> &Option<HitBox> {
		return &self.hitbox;
	}
	
	// boost from current position to next to target, if close enough
	fn boost(&mut self, time_running: f32) {
		let target = if let Some(t) = &self.target { t } else { return; };
		
		let pos_difference = target.pos - self.bounds.left_x;
		// if within range where boost is reasonable, then boost
		if pos_difference < 4.0 * PLAYER_WIDTH as f32 && pos_difference > 0.0 {
			self.lingering_graphics.push( LingeringGraphic {
				positioned_graphic: self.rendering_instruction(),
				start_t: time_running - BOOST_PRELINGER_TIME,
				end_t: time_running + BOOST_LINGER_TIME
			});
			self.bounds.left_x = target.pos;
			self.bounds.right_x = target.pos + PLAYER_WIDTH as f32;
			self.hit_dir = target.hit_dir;
		} else if pos_difference > -4.0 * PLAYER_WIDTH as f32 && pos_difference < 0.0 {
			self.lingering_graphics.push( LingeringGraphic {
				positioned_graphic: self.rendering_instruction(),
				start_t: time_running - BOOST_PRELINGER_TIME,
				end_t: time_running + BOOST_LINGER_TIME
			});
			self.bounds.left_x = target.pos;
			self.bounds.right_x = target.pos + PLAYER_WIDTH as f32;
			self.hit_dir = target.hit_dir;
		}
	}
	
	// updates target, face_dir, and hit_dir
	fn update_target_info(&mut self, bricks_iter: vec_deque::Iter<Brick>, time_running: f32, brick_speed: f32) {
		
		const SAME_GROUP_TIME_BUFFER: f32 = 0.025; // maximum time difference between bricks in the same group
		// maximum time player will wait before chasing next bricks
			// !!! should be variable based on how fast bricks are travelling
		let next_bricks_time_buffer:  f32 = PLAYER_HEIGHT as f32 / brick_speed;
		
		let mut bricks_info = None;
		
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
				
				// if left of target, right of target, in between targets
				if left_target - self.bounds.left_x >= 0.0 {
					self.face_dir = Direction::Right;
					self.target = Some( TargetInfo { time: bi.time, pos: left_target, hit_dir: Direction::Right} )
				} else if self.bounds.left_x - right_target >= 0.0 {
					self.face_dir = Direction::Left;
					self.target = Some( TargetInfo { time: bi.time, pos: right_target, hit_dir: Direction::Left} )
				} else if left_target - self.bounds.left_x > self.bounds.left_x - right_target {
					self.face_dir = Direction::Left;
					self.target = Some ( TargetInfo { time: bi.time, pos: left_target, hit_dir: Direction::Right} )
				} else {
					self.face_dir = Direction::Right;
					self.target = Some ( TargetInfo { time: bi.time, pos: right_target, hit_dir: Direction::Left} )
				}
				
				self.hit_dir = self.face_dir;
			}
		}
	}
	
	// runs to the target, no dashing/boosting
	fn regular_move(&mut self, seconds_passed: f32, time_running: f32) {
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
		match self.state {
			PlayerState::Running => {
				match &self.target {
					None => self.state = PlayerState::Walking,
					Some(ti) => {
						if ti.pos == self.bounds.left_x { // >:< comparison against F32_ZERO is more robust
							self.state = PlayerState::Walking;
						} else {
							self.state = PlayerState::Running;
						}
					}
				}
			},
			PlayerState::Walking => {
				match &self.target {
					None => self.state = PlayerState::Walking,
					Some(ti) => {
						if ti.pos == self.bounds.left_x { // >:< comparison against F32_ZERO is more robust
							self.state = PlayerState::Walking;
						} else {
							self.state = PlayerState::Running;
						}
					}
				}
			},
			PlayerState::PreSlash(t) => {
				if time_running - t > PRE_SLASH_TIME {
					self.boost(time_running);
					
					let brick_type = if let Some(bt) = self.hit_type { bt } else { panic!() };
					let frame = 0;
					let flags;
					let arg = 0;
					let hitbox_x;
					let mut hitbox = HitBox { bounds: self.bounds, brick_type: brick_type };
					
					match self.hit_dir {
						Direction::Right => { 
							flags = 0;
							hitbox_x = self.bounds.left_x;
						},
						Direction::Left => { 
							flags = GraphicFlags::HorizontalFlip as u8;
							hitbox_x = self.bounds.left_x - SLASH_WIDTH as f32;
						}
					};
					
					hitbox.bounds.left_x = hitbox_x;
					hitbox.bounds.right_x = hitbox_x + PLAYER_WIDTH as f32 + SLASH_WIDTH as f32;
					self.hitbox = Some(hitbox);
					self.state = PlayerState::Slash(time_running);
				}
			},
			PlayerState::PreDash(t) => {
				if time_running - t > PRE_SLASH_TIME {
					self.boost(time_running);
					
					let dash_x;
					let dash_graphic_group = GraphicGroup::Dash0;
					
					match self.hit_dir {
						Direction::Right => {
							dash_x = self.bounds.right_x;
							self.bounds.left_x = self.bounds.right_x + DASH_WIDTH as f32;
							self.bounds.right_x = self.bounds.left_x + PLAYER_WIDTH as f32;
						},
						Direction::Left => {
							self.bounds.right_x = self.bounds.left_x - DASH_WIDTH as f32;
							self.bounds.left_x = self.bounds.right_x - PLAYER_WIDTH as f32;
							dash_x = self.bounds.right_x;
						}
					}
					
					let dash_graphic = Graphic { g: dash_graphic_group, frame: 0, flags: 0, arg: 0 };
					self.lingering_graphics.push( LingeringGraphic {
						positioned_graphic: PositionedGraphic { g: dash_graphic, x: dash_x, y: self.bounds.top_y },
						start_t: time_running,
						end_t: time_running + DASH_LINGER_TIME
					});
					
					self.hitbox = None;
					self.state = PlayerState::Dash(time_running);
				}
			},
			PlayerState::PreSlashDash(t) => {
				if time_running - t > PRE_SLASH_TIME {
					self.boost(time_running);
					
					let brick_type = if let Some(bt) = self.hit_type { bt } else { panic!() };
					
					let frame = 0;
					let arg = 0;
					let dash_graphic_x;
					let hitbox_x;
					let mut hitbox = HitBox { bounds: self.bounds, brick_type: brick_type };
					
					// fill variables based on hit dir
					match self.hit_dir {
						Direction::Right => {
							dash_graphic_x = self.bounds.right_x;
							self.bounds.left_x = self.bounds.right_x + DASH_WIDTH as f32;
							self.bounds.right_x = self.bounds.left_x + PLAYER_WIDTH as f32;
							hitbox_x = dash_graphic_x;
						},
						Direction::Left => {
							self.bounds.right_x = self.bounds.left_x - DASH_WIDTH as f32;
							self.bounds.left_x = self.bounds.right_x - PLAYER_WIDTH as f32;
							dash_graphic_x = self.bounds.right_x as f32;
							hitbox_x = self.bounds.left_x - SLASH_WIDTH as f32;
						}
					}
					
					// get graphic groups based on brick type
					let dash_graphic_group = match brick_type {
						BrickType::Type1 => { GraphicGroup::Dash1 },
						BrickType::Type2 => { GraphicGroup::Dash2 },
						BrickType::Type3 => { GraphicGroup::Dash3 }
					};
					
					let dash_graphic = Graphic { g: dash_graphic_group, frame, flags: 0, arg };
					
					// push dash and slash to lingering graphics
					self.lingering_graphics.push( LingeringGraphic {
						positioned_graphic: PositionedGraphic { g: dash_graphic, x: dash_graphic_x, y: self.bounds.top_y },
						start_t: time_running,
						end_t: time_running + DASH_LINGER_TIME
					});
					
					// update hitbox and state
					hitbox.bounds.left_x = hitbox_x;
					hitbox.bounds.right_x = hitbox_x + PLAYER_WIDTH as f32 + SLASH_WIDTH as f32 + DASH_WIDTH as f32;
					self.hitbox = Some(hitbox);
					self.state = PlayerState::SlashDash(time_running);
				}
			},
			PlayerState::Slash(t) => {
				self.state = PlayerState::PostSlash(t);
				self.hitbox = None;
			},
			PlayerState::Dash(t) => {
				self.state = PlayerState::Walking;
				self.hitbox = None;
			},
			PlayerState::SlashDash(t) => {
				self.state = PlayerState::PostSlash(t);
				self.hitbox = None;
			},
			PlayerState::PostSlash(t) => {
				if time_running - t > SLASH_TIME {
					self.state = PlayerState::Walking;
					self.hit_type = None;
				}
			}
		}
	}
	
	// updates the player graphic and gets rid of old lingering graphics
	fn update_graphics(&mut self, time_running: f32) {
		let graphic_group;
		let frame;
		let flags;
		let arg = 0;
		let mut x = self.bounds.left_x;
		
		match self.state {
			PlayerState::Running => {
				graphic_group = GraphicGroup::Running;
				frame = ((time_running / FRAME_TIME) % 256.0) as u8;
			},
			PlayerState::Slash(t) | PlayerState::SlashDash(t) | PlayerState::PostSlash(t) => {
				let brick_type = if let Some(bt) = self.hit_type { bt } else { panic!() };
				graphic_group = match brick_type {
					BrickType::Type1 => GraphicGroup::Slashing1,
					BrickType::Type2 => GraphicGroup::Slashing2,
					BrickType::Type3 => GraphicGroup::Slashing3
				};
				frame = (((time_running - t) / FRAME_TIME) % 256.0) as u8;
				match self.face_dir {
					Direction::Right => (),
					Direction::Left => x = self.bounds.left_x - SLASH_WIDTH as f32,
				};
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
		
		self.graphic = PositionedGraphic { g: graphic, x, y: self.bounds.top_y };
		
		// TODO would prefer if cloning the lingering graphics before removing them was unnecessary
		let new_set: Vec<LingeringGraphic> = self.lingering_graphics.iter().filter(|lg| lg.end_t > time_running).cloned().collect();
		self.lingering_graphics = new_set;
	}
	
	// rendering instruction for the player
	pub fn rendering_instruction(&self) -> PositionedGraphic {
		self.graphic.clone()
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