// TODO
// create the data structure to hold objects in order of layer
// how to manage pixel location in the event of resizing? 
	// - multiply base pixel values based on 1920x1080 by constant factor to achieve window size
// Should SVGs be translated to a jpg/png of an appropriate size?
// Create a function to load a new image for a given identifier

// Create a test exporting the expected names of image files for each variant of Graphic and seeing if the actual image name matches

// mark every #[wasm_bindgen] with just javascript or offline also
// handle losing focus on window / possible browser events that disrupt the game
// best way to detect object collision??

// do wasm functions always happen synchronously ??? (if I have an event handler for key presses, will it only trigger after the
	// wasm function ends??
// decide if objects should be global or exported structures



use wasm_bindgen::prelude::*;
use js_sys::Array;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


const GAME_WIDTH: u32 = 1920;
const GAME_HEIGHT: u32 = 1080;


mod game {
	// !!! split into modules, and bring only used crates into scope
	use crate::*;
	use crate::objects::Object; // needed to use member's methods that are implemented as a part of trait Object
	//use crate::objects;
	
	
	#[wasm_bindgen]
	pub struct Game {
		// !!! create a copy of the reference to player and bricks in a data structure for ordering objects
			// the objects either point to subsequently positioned objects or not (Option type)
		player: objects::Player,
		
	}
	#[wasm_bindgen]
	impl Game {
		pub fn new () -> Game {
			Game {
				player: objects::Player::new()
			}
		}
		
		
		pub fn input_command (&mut self, key: InputKey) {
			match key {
				InputKey::Space => {
					self.player.jump();
				}
				InputKey::LeftArrow => {
					self.player.move_left();
				}
				InputKey::UpArrow => {
					self.player.move_up();
				}
				InputKey::RightArrow => {
					self.player.move_right();
				}
				InputKey::DownArrow => {
					self.player.move_down();
				}
			}
		}
		pub fn stop_command (&mut self, key: InputKey) {
			match key {
				InputKey::Space => {
					return; // !!! can make jumping by holding space bar possible, pros/cons ??
				}
				InputKey::LeftArrow => {
					self.player.stop_left();
				}
				InputKey::UpArrow => {
					self.player.stop_up();
				}
				InputKey::RightArrow => {
					self.player.stop_right();
				}
				InputKey::DownArrow => {
					self.player.stop_down();
				}
			}
		}
		
		
		pub fn tick(&mut self, seconds_passed: f32) {
			self.player.tick(seconds_passed);
		}
		
		
		pub fn get_instructions(&self) -> Array {
			let foo = vec!(
				PositionedGraphic {
					g: Graphic::Background,
					x: 0,
					y: objects::GROUND_POS as i32
				},
				PositionedGraphic {
					g: self.player.get_graphic(),
					x: self.player.get_left_x(),
					y: self.player.get_top_y()
				},
				PositionedGraphic { // >:< create a brick object
					g: Graphic::Brick ,
					x: 0,
					y: 200,
				}
			);
			foo.into_iter().map(JsValue::from).collect()
		}
	}
}


mod objects {
	// !!! split into modules, and bring only used crates into scope
	use crate::Graphic;
	use crate::PositionedGraphic;


	// !!! maybe don't use global/static state?
	static mut BRICK_SPEED: f32 = 0.0; // the speed at which bricks move up the screen
	pub const GROUND_POS: f32 = 240.0;
	const LEFT_BOUNDARY: f32 = 0.0;
	const RIGHT_BOUNDARY: f32 = crate::GAME_WIDTH as f32;
	// !!! Create a top boundary? At least for logging possible anomalies?
	const F32_ZERO: f32 = 0.0000001; // approximately zero for float numbers. any x between -F32_ZERO and +F32_ZERO is essentially 0
	const JUMP_SPEED: f32 = -200.0;
	
	
	pub trait Object {
		fn get_graphic(&self) -> Graphic;
		fn get_left_x(&self) -> i32;
		fn get_top_y(&self) -> i32;
		fn tick(&mut self, seconds_passed: f32);
	}
	
	
	// !!! if there are many different commands, separate a vector of commands to execute and a map of which commands have been added
		// to the execution list (requires HashMap OR conversion of command keys to an incremented enum of commands)
		// or create functions that are added to a vector of functions to execute if the corresponding key was pressed
	pub struct Player {
		graphic: Graphic,
		left_x: f32,
		top_y: f32,
		// using right_x and bottom_y rather than sizes because more comparisons between objects are possible than updates of positions
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
	impl Object for Player {
		
		fn get_graphic(&self) -> Graphic {
			self.graphic
		}
		fn get_left_x(&self) -> i32 {
			self.left_x as i32
		}
		fn get_top_y(&self) -> i32 {
			self.top_y as i32
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
			let size: PositionedGraphic = crate::get_graphic_size(Graphic::Player);
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
	
	
	pub struct Brick {
		graphic: Graphic,
		left_x: f32,
		top_y: f32,
		right_x: f32,
		bottom_y: f32
	}
}


struct AnimationFrame<'a> {
	frame: Graphic,
	next_frame: Option<&'a AnimationFrame<'a>>
	// !!! track # of frames remaining in animation ???
}


// >:<
#[wasm_bindgen]
extern {
    fn alert(s: &str);
}


// !!! offline also
#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Graphic {
	Background = 0,
	Player = 1,
	Brick = 2,
}
#[wasm_bindgen]
pub struct PositionedGraphic {
	pub g: Graphic,
	pub x: i32,
	pub y: i32,
}


// TODO split object dimensions and graphic dimensions
#[wasm_bindgen]
pub fn get_graphic_size(g: Graphic) -> PositionedGraphic {
	return match g {
		Graphic::Background => { PositionedGraphic {
			g,
			x: GAME_WIDTH as i32,
			y: GAME_HEIGHT as i32,
		}},
		Graphic::Player => { PositionedGraphic {
			g,
			x: 45,
			y: 90,
		}},
		Graphic::Brick => { PositionedGraphic {
			g,
			x: 100,
			y: 200,
		}},
	};
}


// !!! offline also
#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum InputKey {
	Space = 32,
	LeftArrow = 37,
	UpArrow = 38,
	RightArrow = 39,
	DownArrow = 40
}



//>:< template for returning pointers to wasm memory
// #[wasm_bindgen]
// pub fn get_array_ptr() -> *const u8 {
	// let dummy_array: [u8; 5] = [0; 5]; //works with vectors as well
	// return dummy_array.as_ptr();
// }


