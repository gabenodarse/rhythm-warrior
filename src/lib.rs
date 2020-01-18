// >:<
// Gather ALL key presses

// TODO
// create the data structure to hold objects in order of layer
// Should SVGs be translated to a jpg/png of an appropriate size?
// Create a function to load a new image for a given identifier

// Create a test exporting the expected names of image files for each variant of Graphic and seeing if the actual image name matches

// mark every #[wasm_bindgen] with just javascript or offline also
// handle losing focus on window / possible browser events that disrupt the game
// best way to detect object collision??

// do wasm functions always happen synchronously ??? (if I have an event handler for key presses, will it only trigger after the
	// wasm function ends??
// decide if objects should be global or exported structures
// add support for different controls
// I would like to do create a member in Game like so: objects: Vec<T: objects::Object>, but as of 1/17 it is not possible
	// follow https://github.com/rust-lang/rust/issues/52662

// TESTS
	// test that objects have correct dimensions



use wasm_bindgen::prelude::*;
use js_sys::Array;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


const GAME_WIDTH: u32 = 1920;
const GAME_HEIGHT: u32 = 1080;

mod objects;

mod game {
	use crate::*;
	use crate::objects::Object; // needed to use member's methods that are implemented as a part of trait Object=
	// >:< use objects::Brick;
	// >:< use objects::Player;
	
	
	#[wasm_bindgen]
	pub struct Game {
		// !!! create a copy of the reference to player and bricks in a data structure for ordering objects
			// the objects either point to subsequently positioned objects or not (Option type)
		time_running: f32, // >:<
		player: objects::Player,
		bricks: Vec<objects::Brick>,
		upcoming_bricks: Vec<UpcomingBrick>,
	}
	#[wasm_bindgen]
	impl Game {
		pub fn new () -> Game {
			Game {
				time_running: 0.0,
				player: objects::Player::new(),
				bricks: vec!(),
				
				// !!! load bricks
				upcoming_bricks: vec!(
					UpcomingBrick{ 
						brick: objects::Brick::new(),
						time: 1.0,
					}
				),
			}
		}
		
		
		pub fn tick(&mut self, seconds_passed: f32) {
			self.time_running += seconds_passed;
			self.player.tick(seconds_passed);
			for brick in &mut self.bricks {
				brick.tick(seconds_passed);
			}
			
			// add any bricks from upcoming_bricks that have reached the time to appear
			let mut last_idx: i32 = self.upcoming_bricks.len() as i32 - 1;
			while last_idx >= 0 {
				if self.upcoming_bricks[last_idx as usize].time < self.time_running {
					let time_difference = self.time_running - self.upcoming_bricks[last_idx as usize].time;
					self.upcoming_bricks[last_idx as usize].brick.tick(time_difference);
					self.bricks.push(self.upcoming_bricks.pop().unwrap().brick);
					last_idx -= 1;
				} else {
					break;
				}
			}
			
			// >:< destroy/handle bricks that are off screen
		}
		
		
		pub fn get_instructions(&self) -> Array {
			let mut instructions = vec!(
				PositionedGraphic {
					g: Graphic::Background,
					x: 0,
					y: objects::GROUND_POS as i32
				},
				self.player.get_rendering_instruction(),
			);
			for brick in &self.bricks {
				instructions.push(brick.get_rendering_instruction());
			}
			instructions.into_iter().map(JsValue::from).collect()
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
		
		
		fn check_upcoming_bricks() {
			
		}
	}
	
	// TODO assert/test that the appearance_time of bricks is in order
	struct UpcomingBrick {
		brick: objects::Brick,
		time: f32, // time of appearance in seconds since the start of the program
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
			x: 60,
			y: 120,
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

