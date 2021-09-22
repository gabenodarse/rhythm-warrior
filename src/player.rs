
use std::collections::VecDeque;
use std::collections::vec_deque;

use crate::PositionedGraphic;
use crate::resources::GraphicGroup;
use crate::Graphic;
use crate::GraphicFlags;

use crate::objects::Object;
use crate::objects::Direction;
use crate::objects::ObjectBounds;
use crate::objects::BrickType;
use crate::objects::TempObjectState;

use crate::slash::Slash;
use crate::dash::Dash;
use crate::brick::Brick;

use crate::GROUND_POS;
use crate::objects::PLAYER_WIDTH;
use crate::objects::PLAYER_HEIGHT;
use crate::objects::BRICK_WIDTH;

pub struct Player {
	graphic: Graphic, // !!! all objects store Graphic
	movement_frame: u8,
	movement_frame_t: f32,
	
	bounds: ObjectBounds,
	dx: f32, // in pixels per second
	
	target: Option<TargetInfo>,
	slash: Option<Slash>,
	dash: Option<Dash>,
	face_dir: Direction,
	hit_dir: Direction
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
			graphic: Graphic { g: GraphicGroup::Walking, frame: 0, flags: 0 },
			bounds: ObjectBounds {
				left_x: x,
				top_y: GROUND_POS as f32 - PLAYER_HEIGHT as f32,
				right_x: x + PLAYER_WIDTH as f32, 
				bottom_y: GROUND_POS as f32
			},
			
			dx: 0.0,
			slash: None,
			dash: None,
			face_dir: Direction::Right,
			hit_dir: Direction::Right,
			target: None,
			movement_frame: 0,
			movement_frame_t: 0.0
		}
	}
	
	pub fn slash (&mut self, brick_type: BrickType, t_since_tick: f32) {
		self.slash = Some(
			match self.hit_dir {
				Direction::Right => {
					Slash::new( self.bounds.right_x, self.bounds.top_y, brick_type, t_since_tick, Direction::Right)
				}
				Direction::Left => {
					Slash::new( self.bounds.left_x, self.bounds.top_y, brick_type, t_since_tick, Direction::Left)
				}
			}
		);
	}
	pub fn dash (&mut self, t_since_tick: f32) {
		if let None = self.dash {
			if let Direction::Right = self.hit_dir {
				self.dash = Some(Dash::new( self.bounds.right_x, self.bounds.top_y, t_since_tick, self.hit_dir)); }
			else {
				self.dash = Some(Dash::new( self.bounds.left_x, self.bounds.top_y, t_since_tick, self.hit_dir)); }}
	}
	
	pub fn face_direction(&self) -> &Direction {
		&self.face_dir
	}
	
	pub fn slashing(&self) -> &Option<Slash> {
		&self.slash
	}
	
	pub fn dashing(&self) -> &Option<Dash> {
		&self.dash
	}
	
	// tick the players state, taking into account input commands
	pub fn tick(&mut self, seconds_passed: f32, bricks_iter: vec_deque::Iter<Brick>, time_running: f32) {
		
		// TODO move a new slash unconditionally to player's position, so its position is set when it actually appears?
		// Tick slash. If it's new check if the dash is also new, in which case they link
		
		// link the dash and slash if they are both new
		let dash = &mut self.dash;
		let slash = &mut self.slash;
		match (slash, dash) {
			(Some(slash), Some(dash)) => {
				match (slash.state(), dash.state()) {
					(TempObjectState::New(slash_t), TempObjectState::New(dash_t)) => {
						// in case a movement input was immediately after 1 or both inputs
						let direction = self.hit_dir;
						
						// position slash to the end of the dash
						slash.link_dash_slash(dash.bounds.left_x, direction, *dash_t);
						
						dash.direction = direction;
						dash.brick_type = Some(slash.brick_type);
					},
					_ => ()
				}
			},
			_ => ()
		}
		
		// tick slash
		if let Some(slash) = &mut self.slash {
			match slash.state() {
				TempObjectState::Lingering(t) if *t < 0.0 => self.slash = None,
				_ => slash.tick(seconds_passed)
			}
		}
		
		// tick dash and/or move
		if let Some(dash) = &mut self.dash {
			match dash.state() {
				TempObjectState::Lingering(t) => {
					if *t < 0.0 {
						self.dash = None;
					}
					else {
						dash.tick(seconds_passed);
					}
					
					self.get_target_info(bricks_iter, time_running);
					self.regular_move(seconds_passed, time_running);
				},
				_ => {
					dash.tick(seconds_passed);
					if let Direction::Right = dash.direction() {
						self.bounds.left_x = dash.bounds.right_x;
						self.bounds.right_x = self.bounds.left_x + PLAYER_WIDTH as f32;
					}
					else {
						self.bounds.right_x = dash.bounds.left_x;
						self.bounds.left_x = self.bounds.right_x - PLAYER_WIDTH as f32;
					}
					self.dx = 0.0;
				}
			}
		}
		else {
			self.get_target_info(bricks_iter, time_running);
			self.regular_move(seconds_passed, time_running);
		}
	}
	
	fn regular_move(&mut self, seconds_passed: f32, time_running: f32) {
		// get nearest brick
		// run to it
		// if it's too far, boost
		// !!! do calculations before game starts
		
		const RUN_SPEED: f32 = 480.0; // in pixels per second
		
		match &self.target {
			None => {
				
			},
			Some(ti) => {
				
				match self.face_dir {
					Direction::Left => {
						let end_pos = self.bounds.left_x - seconds_passed * RUN_SPEED;
						if end_pos < ti.pos {
							self.bounds.left_x = ti.pos;
							self.graphic = Graphic { g: GraphicGroup::Walking, frame: 0, 
								flags: GraphicFlags::HorizontalFlip as u8 }
						} else {
							self.bounds.left_x = end_pos;
							self.graphic = Graphic { g: GraphicGroup::Running, frame: ((time_running / 0.01667) % 256.0) as u8, 
								flags: GraphicFlags::HorizontalFlip as u8 }
						}
					},
					Direction::Right => {
						let end_pos = self.bounds.left_x + seconds_passed * RUN_SPEED;
						if end_pos > ti.pos {
							self.bounds.left_x = ti.pos;
							self.graphic = Graphic { g: GraphicGroup::Walking, frame: 0, flags: 0 }
						} else {
							self.bounds.left_x = end_pos;
							self.graphic = Graphic { g: GraphicGroup::Running, frame: ((time_running / 0.01667) % 256.0) as u8, flags: 0 }
						}
					}
				}
				
				self.bounds.right_x = self.bounds.left_x + PLAYER_WIDTH as f32;
			}
		}
	}
	
	fn get_target_info(&mut self, mut bricks_iter: vec_deque::Iter<Brick>, time_running: f32) {
		
		const TIME_BUFFER: f32 = 0.025; // maximum time difference between bricks appearing at same time (difference should be 0.0)
		let mut bricks_info = None;
		
		struct UpcomingBricks {
			time: f32,
			left_brick: f32,
			right_brick: f32
		}
		
		for brick in bricks_iter {
			if brick.time < time_running {
				continue;
			} 
			
			match &mut bricks_info {
				None => {
					bricks_info = Some( UpcomingBricks {
						time: brick.time,
						left_brick: brick.bounds.left_x,
						right_brick: brick.bounds.left_x
					});
				},
				Some(bi) => {
					if bi.time + TIME_BUFFER < brick.time {
						break; // >:< always chases the highest brick after time running
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
	
	pub fn rendering_instruction(&self) -> PositionedGraphic {
		PositionedGraphic {
			g: self.graphic,
			x: self.bounds.left_x as i32,
			y: self.bounds.top_y as i32,
		}
	}
}