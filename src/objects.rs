
use crate::Graphic;
use crate::PositionedGraphic;


// !!! maybe don't use global/static state?
static mut BRICK_SPEED: f32 = 250.0; // the speed at which bricks move up the screen
pub const GROUND_POS: f32 = 240.0;
const LEFT_BOUNDARY: f32 = 0.0;
const RIGHT_BOUNDARY: f32 = crate::GAME_WIDTH as f32;
// !!! Create a top boundary? At least for logging possible anomalies?
const F32_ZERO: f32 = 0.0000001; // approximately zero for float numbers. any x between -F32_ZERO and +F32_ZERO is essentially 0
const JUMP_SPEED: f32 = -200.0;


pub trait Object {
	fn get_left_x(&self) -> i32;
	fn get_top_y(&self) -> i32;
	fn get_rendering_instruction(&self) -> PositionedGraphic;
	fn tick(&mut self, seconds_passed: f32);
}


// !!! if there are many different commands, separate a vector of commands to execute and a map of which commands have been added
	// to the execution list (requires HashMap OR conversion of command keys to an incremented enum of commands)
	// or create functions that are added to a vector of functions to execute if the corresponding key was pressed
pub struct Player {
	graphic: Graphic,
	
	// using right_x and bottom_y rather than sizes because more comparisons between objects are possible than updates of positions
	left_x: f32,
	top_y: f32,
	right_x: f32, 
	bottom_y: f32,
	dx: f32, // in pixels per second
	dy: f32, // in pixels per second
	
	jumping: bool,
	moving_up: bool,
	moving_down: bool,
	moving_left: bool,
	moving_right: bool
}

#[derive(Clone, Copy)]
pub struct Brick {
	graphic: Graphic,
	left_x: f32,
	top_y: f32,
	right_x: f32,
	bottom_y: f32
}


impl Object for Player {
	
	fn get_left_x(&self) -> i32 {
		self.left_x as i32
	}
	fn get_top_y(&self) -> i32 {
		self.top_y as i32
	}
	fn get_rendering_instruction(&self) -> PositionedGraphic {
		PositionedGraphic {
			g: self.graphic,
			x: self.left_x as i32,
			y: self.top_y as i32,
		}
	}
	
	
	// !!! account for object collisions
	// tick the players state, taking into account input commands
	fn tick(&mut self, seconds_passed: f32) {
		let distance_from_ground;
		distance_from_ground = self.bottom_y - GROUND_POS;
		
		if distance_from_ground > F32_ZERO { 
			// !!! Error, player below ground
		}
		
		// handle jump
		if distance_from_ground > -F32_ZERO && self.dy > -F32_ZERO && self.jumping {
			// !!! smoother jump??? Meaning, not maximum speed instantly
			self.dy = JUMP_SPEED;
		}
		// !!! if it will hit the floor on next tick, don't set jumping to false
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
		if self.moving_down ^ self.moving_up {
			if self.moving_up {
				self.dy += 100.0 * seconds_passed;
			} else {
				self.dy += 300.0 * seconds_passed;
			}
		} else {
			self.dy += 200.0 * seconds_passed;
		}
	
		
		
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
impl Player {
	
	pub fn new() -> Player {
		let size: PositionedGraphic = crate::get_graphic_size(Graphic::Player); // >:<
		const X: f32 = 850.0; // !!! take starting pos as parameters
		const Y: f32 = 0.0;
		
		Player {
			graphic: Graphic::Player,
			left_x: X,
			top_y: Y,
			right_x: X + size.x as f32, 
			bottom_y: Y + size.y as f32,
			dx: 0.0,
			dy: 0.0,
			
			jumping: false,
			moving_up: false,
			moving_down: false,
			moving_left: false,
			moving_right: false
		}
	}
	
	
	pub fn jump (&mut self) {
		self.jumping = true;
	}
	pub fn move_up (&mut self) {
		self.moving_up = true;
	}
	pub fn move_down (&mut self) {
		self.moving_down = true;
	}
	pub fn move_left (&mut self) {
		self.moving_left = true;
	}
	pub fn move_right (&mut self) {
		self.moving_right = true;
	}
	pub fn stop_up (&mut self) {
		self.moving_up = false;
	}
	pub fn stop_down (&mut self) {
		self.moving_down = false;
	}
	pub fn stop_left (&mut self) {
		self.moving_left = false;
	}
	pub fn stop_right (&mut self) {
		self.moving_right = false;
	}
	
}


impl Object for Brick {
	fn get_left_x(&self) -> i32 {
		self.left_x as i32
	}
	fn get_top_y(&self) -> i32 {
		self.top_y as i32
	}
	fn get_rendering_instruction(&self) -> PositionedGraphic {
		PositionedGraphic {
			g: self.graphic,
			x: self.left_x as i32,
			y: self.top_y as i32,
		}
	}
	
	
	fn tick(&mut self, seconds_passed: f32) {
		unsafe { self.top_y -= BRICK_SPEED * seconds_passed; }
		unsafe { self.bottom_y -= BRICK_SPEED * seconds_passed; }
	}
}
impl Brick {
	pub fn new() -> Brick { // >:< take position parameters
		let size: PositionedGraphic = crate::get_graphic_size(Graphic::Brick); // >:<
		Brick {
			graphic: Graphic::Brick,
			left_x: 0.0,
			top_y: crate::GAME_HEIGHT as f32,
			right_x: size.x as f32,
			bottom_y: crate::GAME_HEIGHT as f32 + size.y as f32,
		}
	}
}


