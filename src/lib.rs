
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
// object collision detecting more precise than using a minimum bounding rectangle
// add pausing
// check-sum on loaded songs 

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
const GROUND_POS: f32 = 240.0; // !!! associate with the graphic for the ground

mod objects;

// TODO remove
#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

mod game {
	use crate::*;
	use crate::objects::Object; // needed to use member's methods that are implemented as a part of trait Object=
	use std::collections::VecDeque;
	// >:< use objects::Brick;
	// >:< use objects::Player;
	
	
	// TODO assert/test that the appearance_time of bricks is in order
	// TODO UpcomingBrick type can match format of JSON data: [type, time, pos], doesn't need to hold a whole Brick object
	struct UpcomingBrick {
		brick: objects::Brick,
		time: f32, // time of appearance in seconds since the start of the program
	}
	
	#[wasm_bindgen]
	pub struct Game {
		// !!! create a copy of the reference to player and bricks in a data structure for ordering objects
			// the objects either point to subsequently positioned objects or not (Option type)
		time_running: f32,
		player: objects::Player,
		bricks: VecDeque<objects::Brick>,
		slash: Option<objects::Slash>,
		// TODO if values are only loaded all at once, and only popped from 1 side, 
			// there might be a slightly more efficient data structure than a deque
		upcoming_bricks: VecDeque<UpcomingBrick>, // a vector of the upcoming bricks, ordered by time of appearance
	}
	#[wasm_bindgen]
	impl Game {
		pub fn new () -> Game {
			let mut game = Game {
				time_running: 0.0,
				player: objects::Player::new(),
				bricks: VecDeque::new(),
				slash: None,
				
				// !!! load bricks
				upcoming_bricks: VecDeque::new(),
			};
			
			// !!! load bricks instead
			game.upcoming_bricks.push_back(
				UpcomingBrick{ 
					brick: objects::Brick::new(
						PositionedGraphic {
							g: Graphic::Brick,
							x: 100,
							y: GAME_HEIGHT as i32,
						}
					),
					time: 1.0,
				},
			);
			game.upcoming_bricks.push_back(
				UpcomingBrick{ 
					brick: objects::Brick::new(
						PositionedGraphic {
							g: Graphic::Brick,
							x: 200,
							y: GAME_HEIGHT as i32,
						}
					),
					time: 2.0,
				}
			);
			
			game
		}
		
		
		pub fn tick(&mut self, seconds_passed: f32) {
			self.time_running += seconds_passed;
			self.player.tick(seconds_passed);
			for brick in self.bricks.iter_mut() {
				brick.tick(seconds_passed);
			}
			
			// check for brick destruction
			match &mut self.slash {
				Some(slash) => {
					if slash.get_lifetime() < 0.0 {
						self.slash = None;
					} else {
						slash.tick(seconds_passed);
						
						// Remove bricks that are slashed
						// TODO more efficient way than checking ALL bricks
						self.bricks.retain(|&brick| -> bool {
							brick.get_top_y() > slash.get_bottom_y() ||
							brick.get_right_x() < slash.get_left_x() ||
							brick.get_left_x() > slash.get_right_x() ||
							brick.get_bottom_y() < slash.get_top_y()
						});
						
					}
				}
				None => {}
			}
			
			self.add_upcoming_bricks();
			// >:< destroy/handle bricks that are off screen
		}
		
		
		pub fn get_instructions(&self) -> Array {
			let mut instructions = vec!(
				PositionedGraphic {
					g: Graphic::Background,
					x: 0,
					y: 0
				},
				self.player.get_rendering_instruction(),
			);
			for brick in &self.bricks {
				instructions.push(brick.get_rendering_instruction());
			}
			match &self.slash {
				Some(slash) => {
					instructions.push(slash.get_rendering_instruction());
				}
				None => {}
			}
			instructions.into_iter().map(JsValue::from).collect()
		}
		
		
		pub fn input_command (&mut self, key: InputKey) {
			match key {
				InputKey::Space => {
					self.player.jump();
				}
				InputKey::Comma => {
					self.player.move_left();
				}
				InputKey::Period => {
					self.player.move_right();
				}
				InputKey::Q => {
					self.slash = Some(
					if self.player.facing_right() {
						objects::Slash::new( PositionedGraphic {
							g: Graphic::SlashRight,
							x: self.player.get_right_x(),
							y: self.player.get_top_y(),
						}, true)
					} else {
						objects::Slash::new( PositionedGraphic {
							g: Graphic::SlashLeft,
							x: self.player.get_left_x(),
							y: self.player.get_top_y(),
						}, false)
					}
					);
				}
				InputKey::W => {}
				InputKey::E => {}
				InputKey::R => {}
			}
		}
		pub fn stop_command (&mut self, key: InputKey) {
			match key {
				InputKey::Space => {
					return; // !!! can make jumping by holding space bar possible, pros/cons ??
				}
				InputKey::Comma => {
					self.player.stop_left();
				}
				InputKey::Period => {
					self.player.stop_right();
				}
				InputKey::Q => {}
				InputKey::W => {}
				InputKey::E => {}
				InputKey::R => {}
			}
		}
		
		pub fn load_brick (&mut self, bt: BrickType, time: f32, pos: i32) {
			// TODO create a method load_song instead, and if possible make it more efficient than loading brick by brick
			self.upcoming_bricks.push_back(
				UpcomingBrick{ 
					brick: objects::Brick::new(
						PositionedGraphic {
							g: Graphic::Brick, // >:< match graphic to brick type
							x: pos,
							y: GAME_HEIGHT as i32,
						}
					),
					time: time + 5.0, // >:< calculate from game height, brick height, ground height, and brick speed
				},
			);
		}
		
		
		// add any bricks from upcoming_bricks that have reached the time to appear
		fn add_upcoming_bricks(&mut self) {
			loop {
				match self.upcoming_bricks.get_mut(0) {
					Some(upcoming_brick) => {
						if upcoming_brick.time < self.time_running {
							let time_difference = self.time_running - upcoming_brick.time;
							upcoming_brick.brick.tick(time_difference);
							self.bricks.push_back(self.upcoming_bricks.pop_front().unwrap().brick);
							
						} else {
							break;
						}
					}
					None => {
						break;
					}
				}
			}
		}
	}
}


struct AnimationFrame<'a> {
	frame: Graphic,
	next_frame: Option<&'a AnimationFrame<'a>>
	// !!! track # of frames remaining in animation ???
}


// !!! offline also
#[wasm_bindgen]
#[repr(u8)]
pub enum BrickType {
	Default = 0,
}


// !!! offline also
#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Graphic {
	Background = 0,
	Player = 1,
	Brick = 2,
	SlashRight = 3,
	SlashLeft = 4,
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
			x: 50,
			y: 100,
		}},
		Graphic::Brick => { PositionedGraphic {
			g,
			x: 60,
			y: 120,
		}},
		Graphic::SlashRight | Graphic::SlashLeft => { PositionedGraphic {
			g,
			x: 65,
			y: 100
		}}
	};
}


// !!! offline also
#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum InputKey {
	Space = 32,
	Comma = 188,
	Period = 190,
	Q = 81,
	W = 87,
	E = 69,
	R = 82,
}

