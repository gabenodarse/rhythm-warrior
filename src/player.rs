
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
	
	slash: Option<Slash>,
	dash: Option<Dash>,
	hit_dir: Direction, // !!! necessary?
	face_dir: Direction
}

struct TargetInfo {
	time: f32,
	left_brick: f32, // x position of leftmost brick
	right_brick: f32 // x position of rightmost brick
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
			hit_dir: Direction::Right,
			face_dir: Direction::Right,
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
					self.regular_move(seconds_passed, bricks_iter, time_running);
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
			self.regular_move(seconds_passed, bricks_iter, time_running);
		}
	}
	
	fn regular_move(&mut self, seconds_passed: f32, mut bricks_iter: vec_deque::Iter<Brick>, time_running: f32) {
		// get nearest brick
		// run to it
		// make deterministic based on time of initial calculation
		// !!! do calculations before game starts
		
		const RUN_SPEED: f32 = 400.0; // in pixels per second
		let ti = self.target_info(bricks_iter, time_running);
		 
		if let Some(ti) = ti {
			let left_target = ti.left_brick - PLAYER_WIDTH as f32;
			let right_target = ti.right_brick + BRICK_WIDTH as f32;
			let left_x;
			let face_dir;
			let running: bool;
			let flags;
			
			// if left of target, right of target, in between targets
			if left_target - self.bounds.left_x >= 0.0 {
				let end_pos = self.bounds.left_x + seconds_passed * RUN_SPEED;
				if end_pos > left_target {
					left_x = left_target;
					running = false;
				} else {
					left_x = end_pos;
					running = true;
				}
				face_dir = Direction::Right;
				flags = 0;
			} else if self.bounds.left_x - right_target >= 0.0 {
				let end_pos = self.bounds.left_x - seconds_passed * RUN_SPEED;
				if end_pos < right_target {
					left_x = right_target;
					running = false;
				} else {
					left_x = end_pos;
					running = true;
				}
				face_dir = Direction::Left;
				flags = GraphicFlags::HorizontalFlip as u8;
			} else if left_target - self.bounds.left_x > self.bounds.left_x - right_target {
				let end_pos = self.bounds.left_x - seconds_passed * RUN_SPEED;
				if end_pos < left_target {
					left_x = left_target;
					running = false;
				} else {
					left_x = end_pos;
					running = true;
				}
				face_dir = Direction::Left;
				flags = GraphicFlags::HorizontalFlip as u8;
			} else {
				let end_pos = self.bounds.left_x + seconds_passed * RUN_SPEED;
				if end_pos > right_target {
					left_x = right_target;
					running = false;
				} else {
					left_x = end_pos;
					running = true;
				}
				face_dir = Direction::Right;
				flags = 0;
			}
			
			self.bounds.left_x = left_x;
			self.bounds.right_x = left_x + PLAYER_WIDTH as f32;
			self.face_dir = face_dir;
			self.hit_dir = face_dir;
			self.graphic = if running {
				Graphic { g: GraphicGroup::Running, frame: ((time_running / 0.01667) % 256.0) as u8, flags }
			} else {
				Graphic { g: GraphicGroup::Running, frame: 0, flags }
			}
		}
		
	}
	
	fn target_info(&mut self, mut bricks_iter: vec_deque::Iter<Brick>, time_running: f32) -> Option<TargetInfo> {
		
		const TIME_BUFFER: f32 = 0.025; // maximum time difference between bricks appearing at same time (difference should be 0.0)
		let mut target_info = None;
		
		for brick in bricks_iter {
			if brick.time < time_running {
				continue;
			} 
			
			match &mut target_info {
				None => {
					target_info = Some( TargetInfo {
						time: brick.time,
						left_brick: brick.bounds.left_x,
						right_brick: brick.bounds.left_x
					});
				},
				Some(ti) => {
					if ti.time + TIME_BUFFER < brick.time {
						break; // >:< always chases the highest brick after time running
					}
					
					if brick.bounds.left_x < ti.left_brick {
						ti.left_brick = brick.bounds.left_x;
					} else if brick.bounds.left_x > ti.right_brick {
						ti.right_brick = brick.bounds.left_x;
					}
				}
			}
		}
		
		return target_info;
	}
	
	pub fn rendering_instruction(&self) -> PositionedGraphic {
		PositionedGraphic {
			g: self.graphic,
			x: self.bounds.left_x as i32,
			y: self.bounds.top_y as i32,
		}
	}
}