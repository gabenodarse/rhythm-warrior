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
// add support for different controls



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

