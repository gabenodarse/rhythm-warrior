// TODO
// export renderAll to js
	// renderAll from wasm calls javascript function renderObject OR
	// returns single, long rendering instruction (array or string)
// create the data structure to hold objects in order of layer
// how to manage pixel location in the event of resizing??>:<
// Should SVGs be translated to a jpg/png of an appropriate size?
// Function to load a new image for a given identifier>:<

// How to make sure the instruction for rendering is in sync with the game logic??>:<
// Create a test exporting the expected names of image files for each variant of Graphic and seeing if the actual image name matches

// mark every #[wasm_bindgen] with just javascript or offline also
// look up most efficient data representation for passing/receiving numbers to/from javascript (u8, i32, ???)
// handle losing focus on window / possible browser events that disrupt the game
// best way to detect object collision??


use wasm_bindgen::prelude::*;
use js_sys::Array;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;






// >:<
// do wasm functions always happen synchronously ??? (if I have an event handler for key presses, will it only trigger after the
	// was function ends??
// decide if objects should be global or exported structures

mod game {
	// >:< bring only used crates into scope
	use crate::*; 
	//use crate::objects;
	
	
	// >:< Game and its methods
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
					return;
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
		
		
		pub fn tick(&mut self, time_passed: f32) {
			self.player.tick(time_passed / 1000.0); // >:< state divide by 1000.0
		}
		
		
		pub fn get_instructions(&self) -> Array {
			let foo = vec!(
				self.player.get_player_pos(), 
				PositionedGraphic { // >:<
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
	use crate::Graphic;


	// !!! maybe don't use global/static state?
	static mut BRICK_SPEED: f32 = 0.0; // the speed at which bricks move up the screen
	const GROUND_POS: f32 = 160.0; // the y distance from the top of the window where the ground is
	const F32_ZERO: f32 = 0.000001; // approximately zero for float numbers. any x between -F32_ZERO and +F32_ZERO is essentially 0
	const JUMP_SPEED: f32 = -5.0;
	
	
	// >:< use an object oriented approach ???
	pub enum Object {
		Player(Player),
		Brick(Brick),
	}
	
	
	// >:<
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
		dx: f32,
		dy: f32,
		
		jumping: bool,
		moving_up: bool,
		moving_down: bool,
		moving_left: bool,
		moving_right: bool
	}
	impl Player {
		// >:< initialize as desired
		pub fn new() -> Player {
			let size: crate::PositionedGraphic = crate::get_graphic_size(crate::Graphic::Player); // !!! make const
			const X: f32 = 300.0;
			const Y: f32 = 0.0;
			
			Player {
				graphic: crate::Graphic::Player,
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
		
		
		// >:< might not want this function
		pub fn reset_commands (&mut self) {
			self.jumping = false;			
			self.moving_left = false;		
			self.moving_up = false;		
			self.moving_right = false;		
			self.moving_down = false;
		}
		
		
		// >:< units for dx, dy, time_passed
		pub fn tick(&mut self, time_passed: f32) {
			// >:< tick, and account for object collisions and input commands
			let distance_from_ground;
			unsafe { distance_from_ground = self.bottom_y - GROUND_POS; }
			
			if distance_from_ground < -F32_ZERO { 
				// !!! Error, player below ground
			}
			
			// handle jump
			if distance_from_ground < F32_ZERO && self.dy > -F32_ZERO && self.jumping {
				// !!! smoother jump??? Meaning, not maximum speed instantly
				// !!! hit floor first???
				self.dy = JUMP_SPEED;
				self.jumping = false;
			}
			
			// handle lateral movement
			if self.moving_right ^ self.moving_left {
				if self.moving_right {
					self.dx = (self.dx + 10.0) / 2.0; 
				} else {
					self.dx = (self.dx - 10.0) / 2.0; 
				}
			} else {
				self.dx = self.dx / 10.0;
			}
			
			// >:< handle vertical movement
			self.dy += 2.0; // gravity
			
			
			// calculate resulting position
			// >:< Don't go over left or right boundary
			self.left_x += self.dx * time_passed;
			self.right_x += self.dx * time_passed;
			if GROUND_POS - self.bottom_y > self.dy * time_passed {
				self.bottom_y += self.dy * time_passed;
				self.top_y += self.dy * time_passed;
			} else {
				self.top_y += GROUND_POS - self.bottom_y;
				self.bottom_y = GROUND_POS
			}
			
			
			crate::alert(&format!("x: {}, y: {}, dx: {}, dy: {}", self.left_x, self.top_y, self.dx, self.dy)); // >:<
		}
		
		
		pub fn get_player_pos(&self) -> crate::PositionedGraphic {
			crate::PositionedGraphic {
				g: crate::Graphic::Player,
				x: self.left_x as i32,
				y: self.top_y as i32,
			}
		}
	}
	
	
	pub struct Brick {
		graphic: crate::Graphic,
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
	Player = 0,
	Brick = 1,
}
#[wasm_bindgen]
pub struct PositionedGraphic {
	pub g: Graphic,
	pub x: i32,
	pub y: i32,
}


// >:< split object dimensions and graphic dimensions
#[wasm_bindgen]
pub fn get_graphic_size(g: Graphic) -> PositionedGraphic {
	return match g {
		Graphic::Player => { PositionedGraphic {
			g,
			x: 60,
			y: 60,
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


// !!! offline also
#[wasm_bindgen]
pub fn init_game(foo: i32) {
	alert(&format!("{}",foo)); // >:<
}


// >:<
#[wasm_bindgen]
pub fn get_instructions() -> Array {
	let foo = vec!(
		PositionedGraphic {
			g: Graphic::Player ,
			x: 300,
			y: 0,
		}, PositionedGraphic {
			g: Graphic::Brick ,
			x: 0,
			y: 200,
		}
	);
	foo.into_iter().map(JsValue::from).collect()
}





//>:<
// #[wasm_bindgen]
// pub fn get_array_ptr() -> *const u8 {
	// let dummy_array: [u8; 5] = [0; 5];
	// return dummy_array.as_ptr();
// }


