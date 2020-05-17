
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

// storing all bounds rather than pos+size because more comparisons between objects are possible than updates of positions
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
	dash_time: f32,
	
	bounds: ObjectBounds,
	dx: f32, // in pixels per second
	dy: f32, // in pixels per second
	
	slash: Option<Slash>,
	dash: Option<Dash>,
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

pub enum TempObjectState {
	New(f32), // Stores how long into the tick the object was created, since inputs happen asynchronously with ticks
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
	direction: Direction,
	bounds: ObjectBounds,
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
			dash_time: 0.2, // !!! base on tempo
			bounds: ObjectBounds {
				left_x: x,
				top_y: y,
				right_x: x + PLAYER_WIDTH as f32, 
				bottom_y: y + PLAYER_HEIGHT as f32,
			},
			dx: 0.0,
			dy: 0.0,
			
			jumping: false,
			slash: None,
			dash: None,
			moving_left: false,
			moving_right: false,
			face_dir: Direction::Right,
		}
	}
	
	pub fn slash (&mut self, t_since_tick: f32) {
		self.slash = Some(
			match self.face_dir {
				Direction::Right => {
					Slash::new( self.bounds.right_x, self.bounds.top_y, t_since_tick, Direction::Right)
				}
				Direction::Left => {
					Slash::new( self.bounds.left_x, self.bounds.top_y, t_since_tick, Direction::Left)
				}
			}
		);
	}
	pub fn dash (&mut self, t_since_tick: f32) {
		if let None = self.dash {
			if let Direction::Right = self.face_dir {
				self.dash = Some(Dash::new( self.bounds.right_x, self.bounds.top_y, t_since_tick, self.face_dir)); }
			else {
				self.dash = Some(Dash::new( self.bounds.left_x, self.bounds.top_y, t_since_tick, self.face_dir)); }}
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
	pub fn tick(&mut self, seconds_passed: f32) {
		if let Some(dash) = &mut self.dash { 
			match dash.state() {
				TempObjectState::Lingering(t) if *t < 0.0 => self.dash = None,
				_ => {
					dash.tick(self.dash_time, seconds_passed);
					if let Direction::Right = dash.direction() {
						self.bounds.left_x = dash.bounds.right_x;
						self.bounds.right_x = self.bounds.left_x + PLAYER_WIDTH as f32;
					}
					else {
						self.bounds.right_x = dash.bounds.left_x;
						self.bounds.left_x = self.bounds.right_x - PLAYER_WIDTH as f32;
					}
				}
			}}
		else {
			self.regular_move(seconds_passed); }
		
		if let Some(slash) = &mut self.slash { 
			match slash.state() {
				TempObjectState::Lingering(t) if *t < 0.0 => self.slash = None,
				_ => slash.tick(seconds_passed)
			}}
			
		
		
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
	pub fn new(x: f32, y: f32, t_since_tick: f32, dir: Direction) -> Slash {
		if let Direction::Right = dir {
			Slash {
				state: TempObjectState::New(t_since_tick),
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
				state: TempObjectState::New(t_since_tick),
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
			TempObjectState::New(t) => self.state = TempObjectState::Lingering(0.1 - seconds_passed + *t),
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
	pub fn new(x: f32, y: f32, t_since_tick: f32, dir: Direction) -> Dash {
		if let Direction::Right = dir {
			Dash {
				state: TempObjectState::New(t_since_tick),
				graphic: Graphic::None,
				direction: Direction::Right,
				bounds: ObjectBounds {
					left_x: x,
					top_y: y,
					right_x: x,
					bottom_y: y + DASH_HEIGHT as f32,
				}
			}
		} 
		else {
			Dash {
				state: TempObjectState::New(t_since_tick),
				graphic: Graphic::None,
				direction: Direction::Left,
				bounds: ObjectBounds {
					left_x: x,
					top_y: y,
					right_x: x,
					bottom_y: y + DASH_HEIGHT as f32,
				}
			}
		}
	}
	
	pub fn direction (&self) -> Direction {
		self.direction	
	}
	
	pub fn state (&self) -> &TempObjectState {
		&self.state
	}
	
	pub fn tick(&mut self, dash_time: f32, seconds_passed: f32) {
		match &mut self.state {
		// a new dash lasts dash time - the time since the dash input
		TempObjectState::New(t) => self.state = TempObjectState::Active(dash_time - seconds_passed + *t),
		TempObjectState::Active(t) => {
			// update bounds and graphic
			if *t > 0.0 {
				let next_t = *t - seconds_passed;
				let dash_distance = 
					if next_t > 0.0 { DASH_WIDTH as f32 * (dash_time - next_t) / dash_time } else { DASH_WIDTH as f32};
				
				if let Direction::Right = self.direction {
					if next_t > 0.8 * dash_time {
						self.graphic = Graphic::DashR0 }
					else if next_t > 0.6 * dash_time {
						self.graphic = Graphic::DashR1 }
					else if next_t > 0.4 * dash_time {
						self.graphic = Graphic::DashR2 }
					else if next_t > 0.2 * dash_time {
						self.graphic = Graphic::DashR3 }
					else {
						self.graphic = Graphic::Dash }
					self.bounds.right_x = self.bounds.left_x + dash_distance }
				else {
					if next_t > 0.8 * dash_time {
						self.graphic = Graphic::DashL0 }
					else if next_t > 0.6 * dash_time {
						self.graphic = Graphic::DashL1 }
					else if next_t > 0.4 * dash_time {
						self.graphic = Graphic::DashL2 }
					else if next_t > 0.2 * dash_time {
						self.graphic = Graphic::DashL3 }
					else {
						self.graphic = Graphic::Dash }
					self.bounds.left_x = self.bounds.right_x - dash_distance }
					
				self.state = TempObjectState::Active(next_t); }
			else{
				self.state = TempObjectState::Lingering(*t - seconds_passed + 0.05);}
		},
		TempObjectState::Lingering(t) => self.state = TempObjectState::Lingering(*t - seconds_passed)
		}
	}
}
