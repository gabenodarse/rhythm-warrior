
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
// Send commands on tick rather than asynchronously

// TESTS
// test that objects have correct dimensions

use std::collections::btree_set;
use std::collections::btree_set::BTreeSet; 
use std::cmp::Ordering;
use std::ops::Bound;

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
	use crate::objects::Object; // needed to use member's methods that are implemented as a part of trait Object
	use std::collections::VecDeque;
	use objects::Brick;
	use objects::BrickType;
	use objects::Player;
	use objects::Slash;
	use objects::Direction;
	
	#[derive(Clone, Copy)]
	struct UpcomingNote {
		note_type: BrickType,
		x: f32,
		time: f32, // time of appearance in seconds since the start of the program
	}
	
	struct Song {
		song_name: String,
		notes: BTreeSet<UpcomingNote>, 
		bpm: u8
	}

	impl PartialEq for UpcomingNote {
		fn eq(&self, other: &UpcomingNote) -> bool {
			self.note_type == other.note_type
			&& self.x == other.x
			&& self.time == other.time
		}
	}
	impl Eq for UpcomingNote {}

	impl PartialOrd for UpcomingNote {
		fn partial_cmp(&self, other: &UpcomingNote) -> Option<Ordering> {
			Some(self.cmp(other))
		}
	}

	impl Ord for UpcomingNote {
		fn cmp(&self, other: &UpcomingNote) -> Ordering {
			if self.time < other.time         { Ordering::Less }
			else if self.time > other.time { Ordering::Greater }
			else                              { Ordering::Equal }
		}
	}
	
	#[wasm_bindgen]
	pub struct Game {
		// !!! create a copy of the reference to player and bricks in a data structure for ordering objects
			// the objects either point to subsequently positioned objects or not (Option type)
		time_running: f32,
		// !!! better location for brick speed? (inside brick struct so it isn't passed for every single brick? limitations?)
		brick_speed: f32, // the speed at which bricks move up the screen
		player: Player,
		bricks: VecDeque<Brick>,
		slash: Option<Slash>,
		// !!! create a song type to hold song notes and meta data
		song: Song, 
		upcoming_note: Option<UpcomingNote>
	}
	#[wasm_bindgen]
	impl Game {
		pub fn new () -> Game {
			Game {
				time_running: 0.0,
				brick_speed: 250.0, 
				player: Player::new(),
				bricks: VecDeque::new(), // bricks on screen, ordered by time of appearance (height)
				slash: None,
				song: Song {
					song_name: String::from(""),
					notes: BTreeSet::new(),
					bpm: 96
				},
				upcoming_note: None
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
					if slash.get_last_time() < 0.0 {
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
			
			self.add_upcoming_notes();
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
						match self.player.face_direction() {
							Direction::Right => {
								Slash::new( PositionedGraphic {
									g: Graphic::SlashRight,
									x: self.player.get_right_x(),
									y: self.player.get_top_y(),
								}, true)
							}
							Direction::Left => {
								Slash::new( PositionedGraphic {
									g: Graphic::SlashLeft,
									x: self.player.get_left_x(),
									y: self.player.get_top_y(),
								}, false)
							}
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
		
		// >:< create a method load_song instead, and if possible make it more efficient than loading brick by brick
		pub fn load_brick (&mut self, bt: BrickType, time: f32, pos: f32) {
			self.song.notes.insert(
				UpcomingNote{
					note_type: bt, // !!! match graphic to brick type
					x: pos * get_graphic_size(Graphic::Brick).x as f32 + (GAME_WIDTH / 2) as f32,
					// time the note is played minus the time it takes to get up the screen
					time: time - (GAME_HEIGHT as f32 - GROUND_POS + get_graphic_size(Graphic::Brick).y as f32) / self.brick_speed,
				}
			);
			
			match self.song.notes.iter().next() {
				Some(note) => self.upcoming_note = Some(*note),
				None => self.upcoming_note = None
			}
		}
		
		// pub fn load_song (song: [(BrickType, f32, i32)]) {
		// }
		
		
		// add any bricks from song that have reached the time to appear
		fn add_upcoming_notes(&mut self) {
			if let Some(upcoming_note) = &self.upcoming_note {
				if upcoming_note.time < self.time_running {
					
					let upcoming_notes = self.song.notes.range(*upcoming_note..); // !!! range bounds with a float possible?
					
					//>:<< loop
					for upcoming_note in upcoming_notes {
						if upcoming_note.time > self.time_running {
							self.upcoming_note = Some(*upcoming_note);
							return;
						}
						
						let time_difference = self.time_running - upcoming_note.time;
						
						let mut pg = get_graphic_size(Graphic::Brick);
						pg.x = upcoming_note.x as i32;
						pg.y = GAME_HEIGHT as i32;
						let mut brick = Brick::new(pg);
						brick.tick(self.brick_speed, time_difference); // !!! alters location of song bricks
						self.bricks.push_back(brick);
					}
					
					self.upcoming_note = None;
				}
				
			}
		}
	}
}


struct AnimationFrame<'a> {
	frame: Graphic,
	next_frame: Option<&'a AnimationFrame<'a>>
}

// !!! GraphicID istead of Graphic
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

