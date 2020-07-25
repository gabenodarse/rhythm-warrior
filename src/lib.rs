
// TODO
// handle losing focus on window / possible browser events that disrupt the game

// object collision??
	// object collision detecting more precise than using a minimum bounding rectangle
// check-sum on loaded songs 
// Precise ticking even for longer delta times
// create the data structure to hold objects in order of layer

// music_mercenary.js uses workaround because instantiateStreaming doesn't function correctly (MIME type not working??)
	// https://stackoverflow.com/questions/52239924/webassembly-instantiatestreaming-wrong-mime-type 
	// -- defaulting to using "instantiate" instead of "instantiateStreaming"
// Make sure things work in all browsers, especially ESModules
// expand on read-MIDI functionality, and add options to control generated output such as only use certain program numbers (instruments)
	// or channels to generate notes, criteria for excluding notes if there are too many, etc.
// stick with sqlite/sqljs?

// TESTS
// test that objects have correct dimensions

// !!! pausing/unpausing messes up character pos
// !!! size offset, x offset, y offset for graphics that are sized differently than their objects
// !!! cleanup cargo.toml, include features that are best for game.
// !!! fix and extend midi-reader / song converter
// !!! are as casts what I want / are they idiomatic Rust? Also, types seem to be arbitrary...
	// (define floats and integer forms of constants so casting isn't needed?)

// >:< dash has to go through to hit note


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

mod game {
	use crate::*;
	use std::collections::VecDeque;
	use objects::Object; // needed to use member's methods that are implemented as a part of trait Object
	use objects::Brick;
	use objects::BrickType;
	use objects::Player;
	use objects::TempObjectState;
	use objects::Direction;
	
	fn note_pos_to_x(pos: u8) -> f32 {
		let pos = match pos >= objects::MAX_NOTES_PER_SCREEN_WIDTH {
			true => objects::MAX_NOTES_PER_SCREEN_WIDTH - 1,
			false => pos
		};
		
		return (objects::BRICK_WIDTH * pos as u32) as f32;
	}
	fn note_pos_from_x(x: f32) -> u8 {
		let pos = x as u8 / objects::BRICK_WIDTH as u8;
		let pos = match pos >= objects::MAX_NOTES_PER_SCREEN_WIDTH {
			true => objects::MAX_NOTES_PER_SCREEN_WIDTH - 1,
			false => pos
		};
		
		return pos;
	}
	
	#[derive(Clone, Copy)]
	pub struct UpcomingNote {
		note_type: BrickType,
		x: f32,
		time: f32, // time the note is meant to be played
	}
	
	struct Song {
		notes: BTreeSet<UpcomingNote>,
		bpm: u32,
		// !!! better location for brick speed? (inside brick struct so it isn't passed for every single brick? limitations?)
		brick_speed: f32,
		duration: f32,
		thresholds: TimingThresholds
	}
	
	// within this many ms of when the note is meant to be played
	struct TimingThresholds { 
		perfect: f32,
		good: f32,
		ok: f32,
	}
	
	impl TimingThresholds {
		fn from_brick_speed(brick_speed: f32) -> TimingThresholds {
			let perfect = if 4.0 / brick_speed > 0.01 { // how many seconds it takes to travel 4 pixels
				4.0 / brick_speed
			} else {
				0.01
			};
			
			TimingThresholds {
				perfect,
				good: perfect * 2.0,
				ok: perfect * 4.0,
			}
		}
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
			if self.time < other.time      { Ordering::Less }
			else if self.time > other.time { Ordering::Greater }
			// arbitrary comparisons so that notes of the same time can exist within the same set
			else if (self.note_type as u8) < (other.note_type as u8) { Ordering::Less }
			else if (self.note_type as u8) > (other.note_type as u8) { Ordering::Greater }
			else if self.x < other.x { Ordering::Less }
			else if self.x > other.x { Ordering::Greater }
			else { Ordering::Equal }
		}
	}
	
	#[wasm_bindgen]
	pub struct Game {
		// !!! create a copy of the reference to player and bricks in a data structure for ordering objects
			// the objects either point to subsequently positioned objects or not (Option type)
		time_running: f32, // invariant: should never be negative
		score: i32,
		player: Player,
		bricks: VecDeque<Brick>,
		// !!! create a song type to hold song notes and meta data
		song: Song, 
		upcoming_note: Option<UpcomingNote>
	}
	#[wasm_bindgen]
	impl Game {
		pub fn new(bpm: u32, brick_speed: f32, duration: f32) -> Game {
			return Game {
				time_running: 0.0,
				player: Player::new((GAME_WIDTH / 2) as f32, 0.0),
				bricks: VecDeque::new(), // bricks on screen, ordered by time they are meant to be played
				score: 0,
				song: Song { 
					notes: BTreeSet::new(),
					bpm,
					brick_speed,
					duration,
					thresholds: TimingThresholds::from_brick_speed(brick_speed)
				},
				upcoming_note: None
			};
		}
				
		pub fn tick(&mut self, seconds_passed: f32) {
			self.time_running += seconds_passed;
			
			self.player.tick(seconds_passed);
			
			// tick bricks while discarding any bricks off screen 
			// TODO might not need to check on screen for all notes
			let len = self.bricks.len();
			let mut del = 0;
			for i in 0..len {
				if self.bricks[i].bounds().bottom_y < 0.0 {
					del += 1;
				} else {
					self.bricks[i].tick(self.song.brick_speed, seconds_passed);
					if del > 0 {
						self.bricks.swap(i - del, i);
					}
				}
			}
			if del > 0 {
				self.bricks.truncate(len - del);
			}
			
			// get the destruction bounds for slashing or dashing
			let destruction_bounds = [
				match self.player.slashing() {
					Some(slash) => {
						match slash.state() {
							TempObjectState::Active(_) => Some(slash.bounds()),
							_ => None
						}
					},
					None => None
				},
				match self.player.dashing() {
					Some(dash) => {
						match dash.state() {
							TempObjectState::Active(_) => Some(dash.bounds()),
							_ => None
						}
					},
					None => None
				}
			];
			
			// check for brick destruction 
			// TODO: might be a little faster to do as bricks are updated
			// TODO more efficient way than checking ALL bricks
			let t = self.time_running;
			let score = &mut self.score;
			let bricks = &mut self.bricks;
			let thresholds = &self.song.thresholds;
			for bounds in destruction_bounds.iter() {
				if let Some(bounds) = bounds {
					bricks.retain(|&brick| -> bool {
						let intersect = objects::intersect(&bounds, &brick.bounds());
						if intersect {
							let time_difference = if t > brick.time() { t - brick.time() } else { brick.time() - t };
							*score += 
								if time_difference < thresholds.perfect {
									100
								} 
								else if time_difference < thresholds.good {
									90
								}
								else if time_difference < thresholds.ok {
									80
								}
								else {
									70
								};
							return false;
						}
						return true;
					});
				}
			}
			
			// !!! detecting end of song?
			self.add_upcoming_notes();
		}
		
		// !!! let javascript keep a pointer to the rendering instructions inside wasm, and only update them with this function
			// so there are no races?
		pub fn rendering_instructions(&self) -> Array {
			let mut instructions = vec!(
				PositionedGraphic { // !!! use a background canvas instead of rerendering background each tick?
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
		
		pub fn score(&self) -> i32 {
			return self.score;
		}
		
		pub fn max_score(&self) -> i32 {
			let mut max = 0;
			for _ in self.song.notes.iter() {
				max += 100;
			}
			return max;
		}
		
		pub fn beat_interval(&self) -> f32 {
			let secs_per_beat = 60.0 / self.song.bpm as f32;
			return secs_per_beat;
		}
		
		pub fn brick_speed(&self) -> f32 {
			return self.song.brick_speed;
		}
		
		pub fn song_time(&self) -> f32 {
			return self.time_running;
		}
		
		pub fn song_duration(&self) -> f32 {
			return self.song.duration;
		}
		
		// >:< THIS NEEDS TO BE RELIABLE, OR ELSE USER CREATED SONG DATA MAY BE LOST
		pub fn song_notes_json(&self) -> String {
			let mut res = String::new();
			
			res.push_str("[");
			for note in self.song.notes.iter() {
				res.push_str(&format!("{{\"brickType\": {}, \"time\": {}, \"xPos\": {}}},", 
					note.note_type as u8, 
					note.time, 
					note_pos_from_x(note.x) ));
			}
			res.pop(); // pop trailing comma
			res.push_str("]");
			
			return res;
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
		pub fn toggle_brick (&mut self, bt: BrickType, time: f32, pos: u8) {
			if time > self.song.duration {
				return;
			}
			// !!! just as there is a max time, there should be a min time. During the intro min time a metronome can establish tempo
			
			let brick = UpcomingNote{
				note_type: bt,
				x: note_pos_to_x(pos),
				time
			};
			if !self.song.notes.insert( brick ) {
				self.song.notes.remove( &brick );
			}
			
			match self.song.notes.iter().next() {
				Some(note) => self.upcoming_note = Some(*note),
				None => self.upcoming_note = None
			}
		}
		
		// add any bricks from song that have reached the time to appear
			// uses range weirdness because seeking through a B-tree of non-primitives is weird
		fn add_upcoming_notes(&mut self) {
			if let Some(upcoming_note) = &self.upcoming_note {
				// time that notes should be played plus a buffer time where they travel up the screen
				let appearance_buffer = self.time_running + GAME_HEIGHT as f32 / self.song.brick_speed;
				if upcoming_note.time < appearance_buffer {
					
					let upcoming_notes = self.song.notes.range(*upcoming_note..); // !!! range bounds with a float possible?
					
					for upcoming_note in upcoming_notes {
						if upcoming_note.time > appearance_buffer {
							self.upcoming_note = Some(*upcoming_note);
							return;
						}
						
						let time_difference = appearance_buffer - upcoming_note.time;
						
						let mut brick = Brick::new(
							upcoming_note.note_type,
							upcoming_note.x,
							GAME_HEIGHT as f32 + GROUND_POS - objects::BRICK_HEIGHT as f32,
							upcoming_note.time );
						brick.tick(self.song.brick_speed, time_difference);
						self.bricks.push_back(brick);
					}
					
					self.upcoming_note = None;
				}
				
			}
		}
		
		pub fn seek(&mut self, time: f32) {
			self.time_running = time;
			self.bricks = VecDeque::new();
			self.score = 0;
			
			let min_time = time - (GROUND_POS / self.song.brick_speed);
			let appearance_buffer = time + GAME_HEIGHT as f32 / self.song.brick_speed;
			
			for note in self.song.notes.iter() {
				if note.time > min_time {
					if note.time > appearance_buffer {
						self.upcoming_note = Some(*note);
						return;
					}
					
					let time_difference = appearance_buffer - note.time;
					
					let mut brick = Brick::new(
						note.note_type,
						note.x,
						GAME_HEIGHT as f32 + GROUND_POS - objects::BRICK_HEIGHT as f32,
						note.time);
					brick.tick(self.song.brick_speed, time_difference);
					self.bricks.push_back(brick);
				}
			}
			
			self.upcoming_note = None;
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
pub fn max_graphics(g: GraphicGroup) -> u32 {
	match g {
		GraphicGroup::Background => 1,
		GraphicGroup::Player => 1,
		GraphicGroup::Brick => 64,
		GraphicGroup::SlashRight | GraphicGroup::SlashLeft => 1,
		GraphicGroup::Dash => 1,
		GraphicGroup::DashR0 | GraphicGroup::DashL0 => 1,
		GraphicGroup::DashR1 | GraphicGroup::DashL1 => 1,
		GraphicGroup::DashR2 | GraphicGroup::DashL2 => 1,
		GraphicGroup::DashR3 | GraphicGroup::DashL3 => 1
	}
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, EnumVariantCount)]
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
pub fn ground_pos() -> f32 {
	return GROUND_POS as f32;
}
		
#[wasm_bindgen]
pub fn num_graphic_groups() -> usize {
	return GraphicGroup::num_variants();
}

#[wasm_bindgen]
pub fn num_possible_inputs() -> usize {
	return Input::num_variants();
}
