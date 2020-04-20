
// TODO
// Should SVGs be translated to a jpg/png of an appropriate size?
// Create a function to load a new image for a given identifier
// Instead of all rendering onto a single canvas, use multiple canvases?

// mark every #[wasm_bindgen] with just javascript or offline also
// handle losing focus on window / possible browser events that disrupt the game

// do wasm functions always happen synchronously ??? (if I have an event handler for key presses, will it only trigger after the
	// wasm function ends??
// decide if objects should be global or exported structures
// add support for different controls
// I would like to create a member in Game like so: objects: Vec<T: objects::Object>, but as of 1-17-20 it is not possible
	// follow https://github.com/rust-lang/rust/issues/52662
// best way to detect object collision??
	// object collision detecting more precise than using a minimum bounding rectangle
// add pausing
// check-sum on loaded songs 
// consider storing songs in a different format than JSON
// Precise ticking even for longer delta times
// More precise key-press timings than checking state every tick
// Get time of event occurence rather than time receiving the event???
// create the data structure to hold objects in order of layer

// music_mercenary.js uses workaround because instantiateStreaming doesn't function correctly (MIME type not working??)
	// https://stackoverflow.com/questions/52239924/webassembly-instantiatestreaming-wrong-mime-type 
	// -- defaulting to using "instantiate" instead of "instantiateStreaming"
// Make sure things work in all browsers, especially ESModules
// expand on read-MIDI functionality, and add options to control generated output such as only use certain program numbers (instruments)
	// or channels to generate notes, criteria for excluding notes if there are too many, etc.

// TESTS
// test that objects have correct dimensions
// fuzzing for memory leaks

use wasm_bindgen::prelude::*;
use js_sys::Array;

const GAME_WIDTH: u32 = 1920;
const GAME_HEIGHT: u32 = 1080;
const LEFT_BOUNDARY: f32 = 0.0;
const RIGHT_BOUNDARY: f32 = LEFT_BOUNDARY + GAME_WIDTH as f32;
const TOP_BOUNDARY: f32 = 0.0;
const GROUND_POS: f32 = TOP_BOUNDARY + 240.0; // !!! associate with the graphic for the ground

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
	use objects::Brick;
	use objects::Player;
	use objects::Slash;
	
	
	// TODO UpcomingBrick type can match format of JSON data: [type, time, pos], doesn't need to hold a whole Brick object
	struct UpcomingBrick {
		brick: Brick,
		time: f32, // time of appearance in seconds since the start of the program
	}
	
	#[wasm_bindgen]
	pub struct Game {
		// !!! create a copy of the reference to player and bricks in a data structure for ordering objects
			// the objects either point to subsequently positioned objects or not (Option type)
		time_running: f32,
		brick_speed: f32, // the speed at which bricks move up the screen
		player: Player,
		bricks: VecDeque<Brick>,
		slash: Option<Slash>,
		// TODO if values are only loaded all at once, and only popped from 1 side, 
			// there might be a slightly more efficient data structure than a deque
		upcoming_bricks: VecDeque<UpcomingBrick>, // a vector of the upcoming bricks, ordered by time of appearance
	}
	#[wasm_bindgen]
	impl Game {
		pub fn new () -> Game {
			Game {
				time_running: 0.0,
				brick_speed: 250.0,
				player: Player::new(),
				bricks: VecDeque::new(),
				slash: None,
				upcoming_bricks: VecDeque::new(),
			}
		}
		
		
		pub fn tick(&mut self, seconds_passed: f32) {
			self.time_running += seconds_passed;
			self.player.tick(seconds_passed);
			for brick in self.bricks.iter_mut() {
				brick.tick(self.brick_speed, seconds_passed);
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
			
			// !!! account for object collisions
			
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
		
		
		pub fn input_command (&mut self, input: Input) {
			match input {
				Input::Jump => {
					self.player.jump();
				}
				Input::Left => {
					self.player.move_left();
				}
				Input::Right => {
					self.player.move_right();
				}
				Input::Ability1 => {
					self.slash = Some(
					if self.player.is_facing_right() {
						Slash::new( PositionedGraphic {
							g: Graphic::SlashRight,
							x: self.player.get_right_x(),
							y: self.player.get_top_y(),
						}, true)
					} else {
						Slash::new( PositionedGraphic {
							g: Graphic::SlashLeft,
							x: self.player.get_left_x(),
							y: self.player.get_top_y(),
						}, false)
					}
					);
				}
				Input::Ability2 => {}
				Input::Ability3 => {}
				Input::Ability4	=> {}
			}
		}
		pub fn stop_command (&mut self, key: Input) {
			match key {
				Input::Jump => {
					return; // TODO can make jumping by holding space bar possible, pros/cons ??
				}
				Input::Left => {
					self.player.stop_left();
				}
				Input::Right => {
					self.player.stop_right();
				}
				Input::Ability1 => {}
				Input::Ability2 => {}
				Input::Ability3 => {}
				Input::Ability4 => {}
			}
		}
		
		// !!! assert that the appearance_time of bricks is in order
		pub fn load_brick (&mut self, bt: BrickType, time: f32, pos: i32) {
			// TODO create a method load_song instead, and if possible make it more efficient than loading brick by brick
			self.upcoming_bricks.push_back(
				UpcomingBrick{ 
					brick: Brick::new(
						PositionedGraphic {
							g: Graphic::Brick, // !!! match graphic to brick type
							x: pos,
							y: GAME_HEIGHT as i32,
						}
					),
					time: time - 5.0, // >:< calculate from game height, brick height, ground height, and brick speed
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
							upcoming_brick.brick.tick(self.brick_speed, time_difference);
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
}

#[wasm_bindgen]
#[repr(u8)]
pub enum BrickType {
	Default = 0,
}

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

//>:<
// #[wasm_bindgen]
// #[repr(u8)]
// #[derive(Clone, Copy, Debug)]
// pub enum InputKey {
	// Space = 32,
	// Comma = 188,
	// Period = 190,
	// Q = 81,
	// W = 87,
	// E = 69,
	// R = 82,
// }

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Input {
	Jump,
	Left,
	Right,
	Ability1,
	Ability2,
	Ability3,
	Ability4,
}

