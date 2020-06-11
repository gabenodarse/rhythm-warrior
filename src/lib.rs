
// TODO
// handle losing focus on window / possible browser events that disrupt the game

// do wasm functions always happen synchronously ??? (if I have an event handler for key presses, will it only trigger after the
	// wasm function ends??
// object collision??
	// object collision detecting more precise than using a minimum bounding rectangle
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
// find a better alternative to sqljs

// TESTS
// test that objects have correct dimensions

// !!! pausing/unpausing messes up character pos
// !!! probably better to use shifted ints rather than floats

use std::collections::btree_set::BTreeSet; 
use std::cmp::Ordering;
use macros;

use wasm_bindgen::prelude::*;
use js_sys::Array;
use macros::EnumVariantCount;

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
	use objects::TempObjectState;
	use objects::Direction;
	
	#[derive(Clone, Copy)]
	pub struct UpcomingNote {
		note_type: BrickType,
		x: f32,
		time: f32, // time the note is meant to be played
	}
	
	struct Song {
		song_name: String,
		notes: BTreeSet<UpcomingNote>, 
		bpm: u32,
		brick_speed: f32
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
		time_running: f32, // invariant: should never be negative
		// !!! better location for brick speed? (inside brick struct so it isn't passed for every single brick? limitations?)
		brick_speed: f32,
		player: Player,
		bricks: VecDeque<Brick>,
		// !!! create a song type to hold song notes and meta data
		song: Song, 
		upcoming_note: Option<UpcomingNote>
	}
	#[wasm_bindgen]
	impl Game {
		pub fn new () -> Game {
			Game {
				time_running: 0.0,
				brick_speed: 500.0,
				player: Player::new((GAME_WIDTH / 2) as f32, 0.0),
				bricks: VecDeque::new(), // bricks on screen, ordered by time they are meant to be played
				song: Song {
					song_name: String::from(""),
					notes: BTreeSet::new(),
					bpm: 96,
					brick_speed: 500.0
				},
				upcoming_note: None
			}
		}
		
		
		pub fn tick(&mut self, seconds_passed: f32) {
			self.time_running += seconds_passed;
			
			// check for brick destruction 
			// !!! consolidate destructive objects (replace TempObjectState with DestructiveObjectState?)
			// !!! collide functions on these destructive objects, taking an ObjectBounds?
			// TODO: might be a little faster to do as bricks are updated
			match self.player.slashing() {
				Some(slash) => {
					match slash.state(){
						TempObjectState::New(_) => {
							// Remove bricks that are slashed
							// TODO more efficient way than checking ALL bricks
							let slash_bounds = slash.bounds();
							self.bricks.retain(|&brick| -> bool {
								!objects::intersect(&slash_bounds, &brick.bounds())
							});
						},
						_ => ()
					}
				}
				None => {}
			}
			// ticks after checking for destruction. All graphically overlapping dashes/bricks should be / are destroyed, 1 frame 
				// after the overlap
			match self.player.dashing() {
				Some(dash) => {
					match dash.state(){
						TempObjectState::Active(_) => {
							// Remove bricks that are dashed
							// TODO more efficient way than checking ALL bricks
							let dash_bounds = dash.bounds();
							self.bricks.retain(|&brick| -> bool {
								!objects::intersect(&dash_bounds, &brick.bounds())
							});
						},
						_ => ()
					}
				}
				None => {}
			}
			
			self.player.tick(seconds_passed);
			
			// tick bricks while discarding any bricks off screen 
			// TODO might not need to check on screen for all notes
			let len = self.bricks.len();
			let mut del = 0;
			for i in 0..len {
				if self.bricks[i].bounds().bottom_y < 0.0 {
					del += 1;
				} else {
					self.bricks[i].tick(self.brick_speed, seconds_passed);
					if del > 0 {
						self.bricks.swap(i - del, i);
					}
				}
			}
			if del > 0 {
				self.bricks.truncate(len - del);
			}
			
			self.add_upcoming_notes();
		}
		
		// !!! let javascript keep a pointer to the rendering instructions inside wasm, and only update them with this function
			// so there are no races?
		pub fn rendering_instructions(&self) -> Array {
			let mut instructions = vec!(
				PositionedGraphic {
					g: GraphicGroup::Background,
					x: 0,
					y: 0
				},
			);
			
			instructions.push(self.player.rendering_instruction());
			
			for brick in &self.bricks {
				instructions.push(brick.rendering_instruction());
			}
			
			if let Some(slash) = self.player.slashing() {
				instructions.push(slash.rendering_instruction()); };
			
			if let Some(dash) = self.player.dashing() {
				let ri = dash.rendering_instruction();
				if let Some(ri) = ri {
					instructions.push(ri);
				}
			}
			
			instructions.into_iter().map(JsValue::from).collect()
		}
		
		pub fn ground_pos(&self) -> f32 {
			return GROUND_POS as f32;
		}
		
		pub fn beat_interval(&self) -> f32 {
			let secs_per_beat = 60.0 / self.song.bpm as f32;
			return secs_per_beat;
		}
		
		pub fn brick_speed(&self) -> f32 {
			return self.brick_speed;
		}
		
		pub fn song_time(&self) -> f32 {
			return self.time_running;
		}
		
		pub fn input_command (&mut self, input: Input, t_since_tick: f32) {
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
					self.player.slash(t_since_tick);
				}
				Input::Ability2 => {
					self.player.dash(t_since_tick);
				}
				Input::Ability3 => {}
				Input::Ability4	=> {}
			}
		}
		
		// TODO precision on press but not on release? (no t param)
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
		
		// TODO create a method load_song
		pub fn load_brick (&mut self, bt: BrickType, time: f32, pos: f32) {
			self.song.notes.insert(
				UpcomingNote{
					note_type: bt,
					x: pos * objects::BRICK_WIDTH as f32 + (GAME_WIDTH / 2) as f32,
					time
				}
			);
			
			match self.song.notes.iter().next() {
				Some(note) => self.upcoming_note = Some(*note),
				None => self.upcoming_note = None
			}
		}
		
		pub fn set_song_metadata(&mut self, song_name: String, bpm: u32, brick_speed: f32){
			self.song = Song {
				song_name,
				notes: BTreeSet::new(),
				bpm,
				brick_speed,
			}
		}
		
		// add any bricks from song that have reached the time to appear
		fn add_upcoming_notes(&mut self) {
			if let Some(upcoming_note) = &self.upcoming_note {
				// time that notes should be played plus a buffer time where they travel up the screen
				let appearance_buffer = self.time_running + GAME_HEIGHT as f32 / self.brick_speed;
				if upcoming_note.time < appearance_buffer {
					
					let upcoming_notes = self.song.notes.range(*upcoming_note..); // !!! range bounds with a float possible?
					
					for upcoming_note in upcoming_notes {
						if upcoming_note.time > appearance_buffer {
							self.upcoming_note = Some(*upcoming_note);
							return;
						}
						
						let time_difference = appearance_buffer - upcoming_note.time;
						
						let mut brick = Brick::new(upcoming_note.x, GAME_HEIGHT as f32 + GROUND_POS - objects::BRICK_HEIGHT as f32);
						brick.tick(self.brick_speed, time_difference); // !!! alters location of song bricks
						self.bricks.push_back(brick);
					}
					
					self.upcoming_note = None;
				}
				
			}
		}
	}
	
	#[wasm_bindgen]
	pub fn game_dimensions() -> PositionedGraphic {
		PositionedGraphic {
			g: GraphicGroup::Background, // TODO dummy value. Might be smarter just to get height and width separately
			x: GAME_WIDTH as i32,
			y: GAME_HEIGHT as i32,
		}
	}
}



// !!! use graphics groups with sub IDs instead of single graphics?
#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
struct Graphic {
	group: GraphicGroup,
	sub_id: u8
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, EnumVariantCount)]
pub enum GraphicGroup {
	Background,
	Player,
	Brick,
	SlashRight,
	SlashLeft,
	Dash,
	DashR0, 
	DashR1,
	DashR2,
	DashR3,
	DashL0,
	DashL1,
	DashL2,
	DashL3
}
#[wasm_bindgen]
pub struct PositionedGraphic {
	pub g: GraphicGroup,
	pub x: i32,
	pub y: i32,
}

// TODO split object dimensions and graphic dimensions
#[wasm_bindgen]
pub fn graphic_size(g: GraphicGroup) -> PositionedGraphic {
	return match g {
		GraphicGroup::Background => { PositionedGraphic {
			g,
			x: GAME_WIDTH as i32,
			y: GAME_HEIGHT as i32,
		}},
		GraphicGroup::Player => { PositionedGraphic {
			g,
			x: objects::PLAYER_WIDTH as i32,
			y: objects::PLAYER_HEIGHT as i32,
		}},
		GraphicGroup::Brick => { PositionedGraphic {
			g,
			x: objects::BRICK_WIDTH as i32,
			y: objects::BRICK_HEIGHT as i32,
		}},
		GraphicGroup::SlashRight | GraphicGroup::SlashLeft => { PositionedGraphic {
			g,
			x: objects::SLASH_WIDTH as i32,
			y: objects::SLASH_HEIGHT as i32
		}},
		GraphicGroup::Dash => { PositionedGraphic {
			g,
			x: objects::DASH_WIDTH as i32,
			y: objects::DASH_HEIGHT as i32
		}},
		GraphicGroup::DashR0 | GraphicGroup::DashL0 => { PositionedGraphic {
			g,
			x: objects::DASH_WIDTH as i32 / 5,
			y: objects::DASH_HEIGHT as i32
		}},
		GraphicGroup::DashR1 | GraphicGroup::DashL1 => { PositionedGraphic {
			g,
			x: objects::DASH_WIDTH as i32 * 2 / 5 ,
			y: objects::DASH_HEIGHT as i32
		}},
		GraphicGroup::DashR2 | GraphicGroup::DashL2 => { PositionedGraphic {
			g,
			x: objects::DASH_WIDTH as i32 * 3 / 5,
			y: objects::DASH_HEIGHT as i32
		}},
		GraphicGroup::DashR3 | GraphicGroup::DashL3 => { PositionedGraphic {
			g,
			x: objects::DASH_WIDTH as i32 * 4 / 5,
			y: objects::DASH_HEIGHT as i32
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

#[wasm_bindgen]
pub fn num_graphic_groups() -> usize {
	return GraphicGroup::num_variants();
}
