
// TODO
// Log objects going beyond boundaries
use std::cmp::Ordering;
use wasm_bindgen::prelude::*;

use crate::Graphic;
use crate::PositionedGraphic;
use crate::GROUND_POS;
use crate::LEFT_BOUNDARY;
use crate::RIGHT_BOUNDARY;
use crate::TOP_BOUNDARY;

const F32_ZERO: f32 = 0.000001; // approximately zero for f32. any num between -F32_ZERO and +F32_ZERO is essentially 0
const GRAVITY: f32 = 500.0; // acceleration in pixels per second^2


pub trait Object {
	fn get_left_x (&self) -> i32;
	fn get_right_x (&self) -> i32;
	fn get_top_y (&self) -> i32;
	fn get_bottom_y (&self) -> i32;
	fn get_rendering_instruction(&self) -> PositionedGraphic;
}

enum PlayerAction{
	None,
	Slash,
	Dash
}

#[derive(Clone, Copy)]
pub enum Direction {
	Left,
	Right,
}

pub struct Player {
	
	graphic: Graphic,
	
	jump_velocity: f32,
	
	// using right_x and bottom_y rather than sizes because more comparisons between objects are possible than updates of positions
	left_x: f32,
	top_y: f32,
	right_x: f32, 
	bottom_y: f32,
	dx: f32, // in pixels per second
	dy: f32, // in pixels per second
	
	action: PlayerAction,
	jumping: bool,
	moving_left: bool,
	moving_right: bool,
	face_dir: Direction,
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BrickType {
	Default,
}

// !!! store brick type
#[derive(Clone, Copy)]
pub struct Brick {
	graphic: Graphic,
	left_x: f32,
	top_y: f32,
	right_x: f32,
	bottom_y: f32
}

pub struct Slash {
	graphic: Graphic,
	left_x: f32,
	top_y: f32,
	right_x: f32,
	bottom_y: f32,
	last_time: f32, // how long the slash graphic lasts !!! replace with an animation
}


impl Object for Player {
	
	fn get_left_x(&self) -> i32 {
		self.left_x as i32
	}
	fn get_right_x(&self) -> i32 {
		self.right_x as i32
	}
	fn get_top_y(&self) -> i32 {
		self.top_y as i32
	}
	fn get_bottom_y(&self) -> i32 {
		self.bottom_y as i32
	}
	fn get_rendering_instruction(&self) -> PositionedGraphic {
		PositionedGraphic {
			g: self.graphic,
			x: self.left_x as i32,
			y: self.top_y as i32,
		}
	}
}
impl Player {
	
	pub fn new() -> Player {
		// >:< logic is backwards... graphic size should come from player
		let size: PositionedGraphic = crate::get_graphic_size(Graphic::Player); 
		const X: f32 = 850.0; // >:< take starting pos as parameters
		const Y: f32 = 0.0;
		const JUMP_VELOCITY: f32 = -250.0;
		
		Player {
			graphic: Graphic::Player,
			jump_velocity: JUMP_VELOCITY,
			left_x: X,
			top_y: Y,
			right_x: X + size.x as f32, 
			bottom_y: Y + size.y as f32,
			dx: 0.0,
			dy: 0.0,
			
			action: PlayerAction::None,
			jumping: false,
			moving_left: false,
			moving_right: false,
			face_dir: Direction::Right,
		}
	}
	
	
	pub fn jump (&mut self) {
		self.jumping = true;
	}
	pub fn move_left (&mut self) {
		self.moving_left = true;
		self.face_dir = Direction::Left;
	}
	pub fn move_right (&mut self) {
		self.moving_right = true;
		self.face_dir = Direction::Right;
	}
	pub fn stop_left (&mut self) {
		self.moving_left = false;
		if self.moving_right {
			self.face_dir = Direction::Right;
		}
	}
	pub fn stop_right (&mut self) {
		self.moving_right = false;
		if self.moving_left {
			self.face_dir = Direction::Left;
		}
	}
	
	pub fn face_direction(&self) -> Direction {
		self.face_dir
	}
	
	// tick the players state, taking into account input commands
	pub fn tick(&mut self, seconds_passed: f32) {
		match self.action {
			PlayerAction::None | PlayerAction::Slash => {
				self.regular_move(seconds_passed);
			}
			PlayerAction::Dash => {
				
			}
		}
		
		
	}
	
	fn regular_move(&mut self, seconds_passed: f32) {
		// !!! bounce off ends for sweet bunny hopping mechanics
		//>:< get rid of magic numbers (PID without a PHD)
		let on_ground: bool;
		{
			let distance_from_ground = self.bottom_y - GROUND_POS;
		
			if distance_from_ground > F32_ZERO { 
				// !!! Error, player below ground, log
				self.top_y -= distance_from_ground;
				self.bottom_y -= distance_from_ground;
				on_ground = true;
			} else if distance_from_ground > -F32_ZERO {
				on_ground = true;
			} else {
				on_ground = false;
			}
		}
		
		// handle jump
		if on_ground && self.dy > -F32_ZERO && self.jumping {
			self.dy = self.jump_velocity;
		}
		// TODO if it will hit the floor on next tick (or under a threshold), don't set jumping to false
		self.jumping = false;
		
		// handle lateral movement
		if self.moving_right ^ self.moving_left {
			if self.moving_right {
				self.dx = ((self.dx + 1200.0) / 4.0) * seconds_passed + self.dx * (1.0 - seconds_passed); 
			} else {
				self.dx = ((self.dx - 1200.0) / 4.0) * seconds_passed + self.dx * (1.0 - seconds_passed); 
			}
		} else {
			self.dx = (self.dx / 10.0) * seconds_passed + self.dx * (1.0 - seconds_passed);
		}
		
		
		// handle vertical movement and gravity
		self.dy += GRAVITY * seconds_passed;
		
		// calculate resulting position, while checking to not go past any boundaries
		self.left_x += self.dx * seconds_passed;
		self.right_x += self.dx * seconds_passed;
		if self.left_x < LEFT_BOUNDARY {
			self.right_x -= self.left_x - LEFT_BOUNDARY;
			self.left_x -= self.left_x - LEFT_BOUNDARY;
			self.dx = 0.0;
		} else if self.right_x > RIGHT_BOUNDARY {
			self.left_x -= self.right_x - RIGHT_BOUNDARY;
			self.right_x -= self.right_x - RIGHT_BOUNDARY;
			self.dx = 0.0;
		}
		
		
		self.bottom_y += self.dy * seconds_passed;
		self.top_y += self.dy * seconds_passed;
		if self.bottom_y > GROUND_POS {
			self.top_y -= self.bottom_y - GROUND_POS;
			self.bottom_y -= self.bottom_y - GROUND_POS;
		}
	}
}

impl PartialEq for Brick {
	fn eq(&self, other: &Brick) -> bool {
		self.left_x == other.left_x && self.top_y == other.top_y
		&& self.right_x == other.right_x && self.bottom_y == other.bottom_y
	}
}

impl Eq for Brick {}

impl PartialOrd for Brick {
	fn partial_cmp(&self, other: &Brick) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Brick {
	fn cmp(&self, other: &Brick) -> Ordering {
		if self.top_y < other.top_y      { Ordering::Less }
		else if self.top_y > other.top_y { Ordering::Greater }
		else                             { Ordering::Equal }
	}
}

impl Object for Brick {
	fn get_left_x(&self) -> i32 {
		self.left_x as i32
	}
	fn get_right_x(&self) -> i32 {
		self.right_x as i32
	}
	fn get_top_y(&self) -> i32 {
		self.top_y as i32
	}
	fn get_bottom_y(&self) -> i32 {
		self.bottom_y as i32
	}
	fn get_rendering_instruction(&self) -> PositionedGraphic {
		PositionedGraphic {
			g: self.graphic,
			x: self.left_x as i32,
			y: self.top_y as i32,
		}
	}
}

impl Brick {
	pub fn new (pg: PositionedGraphic) -> Brick { // >:< take position parameters
		let size: PositionedGraphic = crate::get_graphic_size(Graphic::Brick);
		Brick {
			graphic: pg.g,
			left_x: pg.x as f32,
			top_y: pg.y as f32,
			right_x: (pg.x + size.x) as f32,
			bottom_y: (pg.y + size.y) as f32,
		}
	}
	
	pub fn tick(&mut self, brick_speed: f32, seconds_passed: f32) {
		self.top_y -= brick_speed * seconds_passed;
		self.bottom_y -= brick_speed * seconds_passed;
	}
}


impl Object for Slash {
	fn get_left_x(&self) -> i32 {
		self.left_x as i32
	}
	fn get_right_x(&self) -> i32 {
		self.right_x as i32
	}
	fn get_top_y(&self) -> i32 {
		self.top_y as i32
	}
	fn get_bottom_y(&self) -> i32 {
		self.bottom_y as i32
	}
	fn get_rendering_instruction(&self) -> PositionedGraphic {
		PositionedGraphic {
			g: self.graphic,
			x: self.left_x as i32,
			y: self.top_y as i32,
		}
	}
}

impl Slash {
	pub fn new(pg: PositionedGraphic, left_to_right: bool) -> Slash { // >:< take position parameters
		let size: PositionedGraphic = crate::get_graphic_size(pg.g);
		if left_to_right {
			Slash {
				graphic: pg.g,
				left_x: pg.x as f32,
				top_y: pg.y as f32,
				right_x: (pg.x + size.x) as f32,
				bottom_y: (pg.y + size.y) as f32,
				last_time: 0.1,
			}
		} else {
			Slash {
				graphic: pg.g,
				left_x: (pg.x - size.x) as f32,
				top_y: pg.y as f32,
				right_x: pg.x as f32,
				bottom_y: (pg.y + size.y) as f32,
				last_time: 0.1,
			}
		}
	}
	
	pub fn get_last_time (&self) -> f32 {
		self.last_time
	}
	
	pub fn tick(&mut self, seconds_passed: f32) {
		self.last_time -= seconds_passed;
	}
}

