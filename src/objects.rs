
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

pub const PLAYER_WIDTH: u32 = 50;
pub const PLAYER_HEIGHT: u32 = 100;
pub const BRICK_WIDTH: u32 = 60;
pub const BRICK_HEIGHT: u32 = 120;
pub const SLASH_WIDTH: u32 = 65;
pub const SLASH_HEIGHT: u32 = 100;
pub const DASH_WIDTH: u32 = 140;
pub const DASH_HEIGHT: u32 = 100;

pub trait Object {
	fn bounds (&self) -> ObjectBounds; // TODO copying the full object bounds may be extra work in some instances
	fn rendering_instruction(&self) -> PositionedGraphic;
}

#[derive(Clone, Copy)]
pub struct ObjectBounds {
	pub left_x: f32,
	pub right_x: f32,
	pub top_y: f32,
	pub bottom_y: f32
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
	bounds: ObjectBounds,
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

// !!! store brick type, match graphic to brick type
#[derive(Clone, Copy)]
pub struct Brick {
	graphic: Graphic,
	bounds: ObjectBounds,
}

// !!! single State enum for ephemeral graphics?
pub enum TempObjectState {
	New,
	Active(f32),
	Lingering(f32)
}

pub struct Slash {
	state: TempObjectState, // how long the slash graphic lasts !!! replace with an animation
	graphic: Graphic,
	bounds: ObjectBounds,
}

pub struct Dash {
	state: TempObjectState, 
	graphic: Graphic,
	bounds: ObjectBounds,
}

//>:< update player action
enum PlayerAction {
	None,
	Slash,
	Dash
}

pub fn intersect(obj1: &ObjectBounds, obj2: &ObjectBounds) -> bool {
	if obj1.top_y > obj2.bottom_y
	|| obj1.right_x < obj2.left_x
	|| obj1.left_x > obj2.right_x
	|| obj1.bottom_y < obj2.top_y {
		return false;
	}
	return true;
}

impl Object for Player {
	
	fn bounds(&self) -> ObjectBounds {
		self.bounds
	}
	fn rendering_instruction(&self) -> PositionedGraphic {
		PositionedGraphic {
			g: self.graphic,
			x: self.bounds.left_x as i32,
			y: self.bounds.top_y as i32,
		}
	}
}
impl Player {
	
	pub fn new(x: f32, y: f32) -> Player {
		const JUMP_VELOCITY: f32 = -250.0;
		
		Player {
			graphic: Graphic::Player,
			jump_velocity: JUMP_VELOCITY,
			bounds: ObjectBounds {
				left_x: x,
				top_y: y,
				right_x: x + PLAYER_WIDTH as f32, 
				bottom_y: y + PLAYER_HEIGHT as f32,
			},
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
			let distance_from_ground = self.bounds.bottom_y - GROUND_POS;
		
			if distance_from_ground > F32_ZERO { 
				// !!! Error, player below ground, log
				self.bounds.top_y -= distance_from_ground;
				self.bounds.bottom_y -= distance_from_ground;
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
		self.bounds.left_x += self.dx * seconds_passed;
		self.bounds.right_x += self.dx * seconds_passed;
		if self.bounds.left_x < LEFT_BOUNDARY {
			self.bounds.right_x -= self.bounds.left_x - LEFT_BOUNDARY;
			self.bounds.left_x -= self.bounds.left_x - LEFT_BOUNDARY;
			self.dx = 0.0;
		} else if self.bounds.right_x > RIGHT_BOUNDARY {
			self.bounds.left_x -= self.bounds.right_x - RIGHT_BOUNDARY;
			self.bounds.right_x -= self.bounds.right_x - RIGHT_BOUNDARY;
			self.dx = 0.0;
		}
		
		
		self.bounds.bottom_y += self.dy * seconds_passed;
		self.bounds.top_y += self.dy * seconds_passed;
		if self.bounds.bottom_y > GROUND_POS {
			self.bounds.top_y -= self.bounds.bottom_y - GROUND_POS;
			self.bounds.bottom_y -= self.bounds.bottom_y - GROUND_POS;
		}
	}
}

impl PartialEq for Brick {
	fn eq(&self, other: &Brick) -> bool {
		self.bounds.left_x == other.bounds.left_x && self.bounds.top_y == other.bounds.top_y
		&& self.bounds.right_x == other.bounds.right_x && self.bounds.bottom_y == other.bounds.bottom_y
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
		if self.bounds.top_y < other.bounds.top_y      { Ordering::Less }
		else if self.bounds.top_y > other.bounds.top_y { Ordering::Greater }
		else                             { Ordering::Equal }
	}
}

impl Object for Brick {
	fn bounds(&self) -> ObjectBounds {
		self.bounds
	}
	fn rendering_instruction(&self) -> PositionedGraphic {
		PositionedGraphic {
			g: self.graphic,
			x: self.bounds.left_x as i32,
			y: self.bounds.top_y as i32,
		}
	}
}

impl Brick {
	pub fn new (x: f32, y: f32) -> Brick {
		Brick {
			graphic: Graphic::Brick, 
			bounds: ObjectBounds {
				left_x: x,
				top_y: y,
				right_x: x + BRICK_WIDTH as f32,
				bottom_y: y + BRICK_HEIGHT as f32,
			}
		}
	}
	
	pub fn tick(&mut self, brick_speed: f32, seconds_passed: f32) {
		self.bounds.top_y -= brick_speed * seconds_passed;
		self.bounds.bottom_y -= brick_speed * seconds_passed;
	}
}


impl Object for Slash {
	fn bounds(&self) -> ObjectBounds {
		self.bounds
	}
	fn rendering_instruction(&self) -> PositionedGraphic {
		PositionedGraphic {
			g: self.graphic,
			x: self.bounds.left_x as i32,
			y: self.bounds.top_y as i32,
		}
	}
}

impl Slash {
	pub fn new(x: f32, y: f32, left_to_right: bool) -> Slash {
		if left_to_right {
			Slash {
				state: TempObjectState::New,
				graphic: Graphic::SlashRight,
				bounds: ObjectBounds {
					left_x: x,
					top_y: y,
					right_x: x + SLASH_WIDTH as f32,
					bottom_y: y + SLASH_HEIGHT as f32,
				}
			}
		} else {
			Slash {
				state: TempObjectState::New,
				graphic: Graphic::SlashLeft,
				bounds: ObjectBounds {
					left_x: x - SLASH_WIDTH as f32,
					top_y: y,
					right_x: x,
					bottom_y: y + SLASH_HEIGHT as f32,
				}
			}
		}
	}
	
	pub fn state (&self) -> &TempObjectState {
		&self.state
	}
	
	pub fn tick(&mut self, seconds_passed: f32) {
		match &mut self.state {
			TempObjectState::New => self.state = TempObjectState::Lingering(0.1 - seconds_passed),
			TempObjectState::Lingering(t) => self.state = TempObjectState::Lingering(*t - seconds_passed),
			_ => { panic!() }
		}
	}
}

impl Object for Dash {
	fn bounds(&self) -> ObjectBounds {
		self.bounds
	}
	fn rendering_instruction(&self) -> PositionedGraphic {
		PositionedGraphic {
			g: self.graphic,
			x: self.bounds.left_x as i32,
			y: self.bounds.top_y as i32,
		}
	}
}

impl Dash {
	pub fn new(x: f32, y: f32, left_to_right: bool) -> Dash {
		if left_to_right {
			Dash {
				state: TempObjectState::New,
				graphic: Graphic::Dash,
				bounds: ObjectBounds {
					left_x: x,
					top_y: y,
					right_x: x + DASH_WIDTH as f32,
					bottom_y: y + DASH_HEIGHT as f32,
				}
			}
		} else {
			Dash {
				state: TempObjectState::New,
				graphic: Graphic::Dash,
				bounds: ObjectBounds {
					left_x: x - DASH_WIDTH as f32,
					top_y: y,
					right_x: x,
					bottom_y: y + DASH_HEIGHT as f32,
				}
			}
		}
	}
	
	pub fn state (&self) -> &TempObjectState {
		&self.state
	}
	
	pub fn tick(&mut self, seconds_passed: f32) {
		match &mut self.state {
			TempObjectState::New => self.state = TempObjectState::Active(0.1 - seconds_passed),
			TempObjectState::Active(t) => {
				self.state = 
					if *t < 0.0 { TempObjectState::Lingering(*t - seconds_passed + 0.05) }
					else { TempObjectState::Active(*t - seconds_passed) };
			},
			TempObjectState::Lingering(t) => self.state = TempObjectState::Lingering(*t - seconds_passed)
		}
	}
}
