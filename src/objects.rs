
// TODO
// Log objects going beyond boundaries


use crate::Graphic;
use crate::PositionedGraphic;
use crate::GROUND_POS;
use crate::LEFT_BOUNDARY;
use crate::RIGHT_BOUNDARY;
use crate::TOP_BOUNDARY;

const F32_ZERO: f32 = 0.000001; // approximately zero for f32. any num between -F32_ZERO and +F32_ZERO is essentially 0


pub trait Object {
	fn get_left_x (&self) -> i32;
	fn get_right_x (&self) -> i32;
	fn get_top_y (&self) -> i32;
	fn get_bottom_y (&self) -> i32;
	fn get_rendering_instruction(&self) -> PositionedGraphic;
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
	
	jumping: bool,
	moving_left: bool,
	moving_right: bool,
	is_facing_right: bool,
}

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
	lifetime: f32, // how long the slash graphic lasts !!! replace with an animation
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
		let size: PositionedGraphic = crate::get_graphic_size(Graphic::Player);
		const X: f32 = 850.0; // >:< take starting pos as parameters
		const Y: f32 = 0.0;
		
		Player {
			graphic: Graphic::Player,
			jump_velocity: -200.0,
			left_x: X,
			top_y: Y,
			right_x: X + size.x as f32, 
			bottom_y: Y + size.y as f32,
			dx: 0.0,
			dy: 0.0,
			
			jumping: false,
			moving_left: false,
			moving_right: false,
			is_facing_right: true,
		}
	}
	
	
	pub fn jump (&mut self) {
		self.jumping = true;
	}
	pub fn move_left (&mut self) {
		self.moving_left = true;
		self.is_facing_right = false;
	}
	pub fn move_right (&mut self) {
		self.moving_right = true;
		self.is_facing_right = true;
	}
	pub fn stop_left (&mut self) {
		self.moving_left = false;
		if self.moving_right {
			self.is_facing_right = true;
		}
	}
	pub fn stop_right (&mut self) {
		self.moving_right = false;
		if self.moving_left {
			self.is_facing_right = false;
		}
	}
	
	pub fn is_facing_right(&self) -> bool {
		if self.is_facing_right {
			true
		} else {
			false
		}
	}
	
	// tick the players state, taking into account input commands
	pub fn tick(&mut self, seconds_passed: f32) {
		
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
		self.dy += 200.0 * seconds_passed;
		
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
				lifetime: 0.1,
			}
		} else {
			Slash {
				graphic: pg.g,
				left_x: (pg.x - size.x) as f32,
				top_y: pg.y as f32,
				right_x: pg.x as f32,
				bottom_y: (pg.y + size.y) as f32,
				lifetime: 0.1,
			}
		}
	}
	
	pub fn get_lifetime (&self) -> f32 {
		self.lifetime
	}
	
	pub fn tick(&mut self, seconds_passed: f32) {
		self.lifetime -= seconds_passed;
	}
}

