
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

use crate::slash::Slash;
use crate::dash::Dash;
use crate::brick::Brick;

use crate::GROUND_POS;
use crate::Song;
use crate::objects::PLAYER_WIDTH;
use crate::objects::PLAYER_HEIGHT;
use crate::objects::BRICK_WIDTH;
use crate::objects::DASH_WIDTH;
use crate::objects::SLASH_WIDTH;

const SLASH_TIME: f32 = 0.04; // delay dash/slash by a tiny amount so they can be pressed at the same time
const SLASH_LINGER_TIME: f32 = 0.1; // how long the slash graphic lingers
const DASH_LINGER_TIME: f32 = 0.3; // how long the dash graphic lingers
const BOOST_LINGER_TIME: f32 = 0.1;
const BOOST_PRELINGER_TIME: f32 = 0.04;

pub struct Player {
	graphic: Graphic, // !!! all objects store Graphic
	state: PlayerState,
	
	bounds: ObjectBounds,
	
	target: Option<TargetInfo>,
	face_dir: Direction,
	hit_dir: Direction,
	
	slash: Option<Slash>,
	dash: Option<Dash>,
	hit_type: Option<BrickType>,
	
	lingering_graphics: Vec<LingeringGraphic>
}

enum PlayerState {
	Running,
	Walking,
	PreSlash(f32), // init time of slash/dash, to insert very short delay before becoming active
	PreDash(f32),
	PreSlashDash(f32)
}

struct TargetInfo {
	time: f32,
	pos: f32
}

impl Object for Player {
	
	fn bounds(&self) -> ObjectBounds {
		self.bounds
	}
}

impl Player {
	
	pub fn new(x: f32) -> Player {
		Player {
			graphic: Graphic { g: GraphicGroup::Walking, frame: 0, flags: 0, arg: 0 },
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
			
			slash: None,
			dash: None,
			hit_type: None,
			lingering_graphics: Vec::new() // graphics for objects no longer present but still showing, e.g. slashes/dashes that have executed
		}
	}
	
	// tick the player's state
	pub fn tick(&mut self, seconds_passed: f32, bricks_iter: vec_deque::Iter<Brick>, time_running: f32, song: &Song) {
		
		if let Some(slash) = &self.slash {
			self.slash = None;
		}
		if let Some(dash) = &self.dash {
			self.dash = None;
		}
		
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
	
	// returns the hitbox of the slash, if any
	pub fn slash_hitbox(&self) -> Option<HitBox> {
		match &self.slash {
			None => return None,
			Some(slash) => {
				return Some( HitBox {
					brick_type: slash.brick_type,
					bounds: slash.bounds
				});
			}
		}
	}
	
	// returns the hitbox of the dash, if any
	pub fn dash_hitbox(&self) -> Option<HitBox> {
		match &self.dash {
			None => return None,
			Some(dash) => {
				match dash.brick_type {
					None => return None,
					Some(bt) => {
						return Some( HitBox {
							brick_type: bt,
							bounds: dash.bounds
						});
					}
				}
			}
		}
	}
	
	fn boost(&mut self, time_running: f32) {
		let target = if let Some(t) = &self.target { t } else { return; };
		
		let pos_difference = target.pos - self.bounds.left_x;
		// if within range where boost is reasonable, then boost
		if pos_difference > 0.5 * PLAYER_WIDTH as f32 && pos_difference < 4.0 * PLAYER_WIDTH as f32 {
			self.lingering_graphics.push( LingeringGraphic {
				positioned_graphic: self.rendering_instruction(),
				start_t: time_running - BOOST_PRELINGER_TIME,
				end_t: time_running + BOOST_LINGER_TIME
			});
			self.bounds.left_x = target.pos;
			self.bounds.right_x = target.pos + PLAYER_WIDTH as f32;
		} else if pos_difference < -0.5 * PLAYER_WIDTH as f32 && pos_difference > -4.0 * PLAYER_WIDTH as f32 { // >:<
			self.lingering_graphics.push( LingeringGraphic {
				positioned_graphic: self.rendering_instruction(),
				start_t: time_running - BOOST_PRELINGER_TIME,
				end_t: time_running + BOOST_LINGER_TIME
			});
			self.bounds.left_x = target.pos;
			self.bounds.right_x = target.pos + PLAYER_WIDTH as f32;
		}
	}
	
	// creates a new slash if one is not present
	fn slash(&mut self, brick_type: BrickType, time_running: f32) {
		if let None = self.slash { 
			let slash = match self.hit_dir {
				Direction::Right => {
					Slash::new( self.bounds.right_x, self.bounds.top_y, brick_type, Direction::Right)
				},
				Direction::Left => {
					Slash::new( self.bounds.left_x - SLASH_WIDTH as f32, self.bounds.top_y, brick_type, Direction::Left)
				}
			};
			
			self.lingering_graphics.push( LingeringGraphic {
				positioned_graphic: slash.rendering_instruction(),
				start_t: time_running,
				end_t: time_running + SLASH_LINGER_TIME
			});
			self.slash = Some(slash);
		}
	}
	
	// creates a new dash if one is not present, updates player pos
	fn dash(&mut self, brick_type: Option<BrickType>, time_running: f32) {
		if let None = self.dash {
			let dash = match self.hit_dir {
				Direction::Right => {
					Dash::new( self.bounds.right_x, self.bounds.top_y, brick_type, self.hit_dir)
				},
				Direction::Left => {
					Dash::new( self.bounds.left_x - DASH_WIDTH as f32, self.bounds.top_y, brick_type, self.hit_dir)
				}
			};
			
			match self.hit_dir {
				Direction::Right => {
					self.bounds.left_x = dash.bounds.right_x;
					self.bounds.right_x = self.bounds.left_x + PLAYER_WIDTH as f32;
				},
				Direction::Left => {
					self.bounds.right_x = dash.bounds.left_x;
					self.bounds.left_x = self.bounds.right_x - PLAYER_WIDTH as f32;
				}
			}
			
			self.lingering_graphics.push( LingeringGraphic {
				positioned_graphic: dash.rendering_instruction(),
				start_t: time_running,
				end_t: time_running + DASH_LINGER_TIME
			});
			self.dash = Some(dash);
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
					self.target = Some( TargetInfo { time: bi.time, pos: left_target} )
				} else if self.bounds.left_x - right_target >= 0.0 {
					self.face_dir = Direction::Left;
					self.target = Some( TargetInfo { time: bi.time, pos: right_target} )
				} else if left_target - self.bounds.left_x > self.bounds.left_x - right_target {
					self.face_dir = Direction::Left;
					self.target = Some ( TargetInfo { time: bi.time, pos: left_target} )
				} else {
					self.face_dir = Direction::Right;
					self.target = Some ( TargetInfo { time: bi.time, pos: right_target} )
				}
				
				self.hit_dir = self.face_dir; // >:< 
			}
		}
	}
	
	// runs to the target, no dashing/boosting
	fn regular_move(&mut self, seconds_passed: f32, time_running: f32) {
		const RUN_SPEED: f32 = 480.0; // in pixels per second
		
		match &self.target {
			None => {
				
			},
			Some(ti) => {
				// >:< can shorten
				match self.face_dir {
					Direction::Left => {
						let end_pos = self.bounds.left_x - seconds_passed * RUN_SPEED;
						if end_pos < ti.pos {
							self.bounds.left_x = ti.pos;
						} else {
							self.bounds.left_x = end_pos;
						}
					},
					Direction::Right => {
						let end_pos = self.bounds.left_x + seconds_passed * RUN_SPEED;
						if end_pos > ti.pos {
							self.bounds.left_x = ti.pos;
						} else {
							self.bounds.left_x = end_pos;
						}
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
				if time_running - t > SLASH_TIME {
					let brick_type;
					if let Some(bt) = self.hit_type {
						brick_type = bt;
					} else { panic!(); }
					self.hit_type = None;
					
					self.boost(time_running);
					self.slash(brick_type, time_running);
					self.state = PlayerState::Walking;
				}
			},
			PlayerState::PreDash(t) => {
				if time_running - t > SLASH_TIME {
					self.boost(time_running);
					self.dash(None, time_running);
					self.state = PlayerState::Walking;
				}
			},
			PlayerState::PreSlashDash(t) => {
				if time_running - t > SLASH_TIME {
					let brick_type;
					if let Some(bt) = self.hit_type {
						brick_type = bt;
					} else { panic!(); }
					self.hit_type = None;
					
					self.boost(time_running);
					self.dash(Some(brick_type), time_running);
					self.slash(brick_type, time_running);
					
					self.state = PlayerState::Walking;
				}
			}
		}
	}
	
	// updates the player graphic and gets rid of old lingering graphics
	fn update_graphics(&mut self, time_running: f32) {
		let g;
		let frame;
		let flags;
		let arg = 0;
		match self.state {
			PlayerState::Running => {
				g = GraphicGroup::Running;
				frame = ((time_running / 0.01667) % 256.0) as u8;
			}
			_ => {
				g = GraphicGroup::Walking;
				frame = 0;
			}
		}
		flags = match self.face_dir {
			Direction::Right => 0,
			Direction::Left => GraphicFlags::HorizontalFlip as u8
		};
		
		self.graphic = Graphic { g, frame, flags, arg };
		
		// TODO would prefer if cloning the lingering graphics before removing them was unnecessary
		let new_set: Vec<LingeringGraphic> = self.lingering_graphics.iter().filter(|lg| lg.end_t > time_running).cloned().collect();
		self.lingering_graphics = new_set;
	}
	
	// rendering instruction for the player
	pub fn rendering_instruction(&self) -> PositionedGraphic {
		PositionedGraphic {
			g: self.graphic,
			x: self.bounds.left_x as i32,
			y: self.bounds.top_y as i32,
		}
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