
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
// Log objects going beyond boundaries
// Valid to create/delete menu if it means better performance

// TESTS
// test that objects have correct dimensions

// !!! Way to trigger GC? Prevent GC? Prime cache before playing?
// !!! logging - panics to logs
// !!! size offset, x offset, y offset for graphics that are sized differently than their objects
// !!! cleanup cargo.toml, include features that are best for game.
// !!! fix and extend midi-reader / song converter
// !!! are as casts what I want / are they idiomatic Rust? Also, types seem to be arbitrary...
	// (define floats and integer forms of constants so casting isn't needed?)
	// why are objects (in objects.rs) and GAME_WIDTH and GAME_HEIGHT integers while most everythings else are floats?

mod objects;
mod resources;
mod player;
mod brick;
mod game;

use std::cmp::Ordering;
use macros;

use wasm_bindgen::prelude::*;
use macros::EnumVariantCount;

use resources::GraphicGroup;
use objects::BrickType;

const GAME_WIDTH: i32 = 1920;
const GAME_HEIGHT: i32 = 1080;
const LEFT_BOUNDARY: f32 = 0.0;
const RIGHT_BOUNDARY: f32 = LEFT_BOUNDARY + GAME_WIDTH as f32;
const TOP_BOUNDARY: f32 = 0.0;
const GROUND_POS: f32 = TOP_BOUNDARY + 240.0; // !!! associate with the graphic for the ground
const TIME_ZERO_BRICK_POS: f32 = GROUND_POS - (objects::PLAYER_HEIGHT as f32 / 2.0) - (objects::BRICK_HEIGHT as f32 / 2.0);
const FRAME_TIME: f32 = 0.00833; // 120 fps

const F32_ZERO: f32 = 0.000001; // approximately zero for f32. any num between -F32_ZERO and +F32_ZERO is essentially 0

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct GameData {
	pub bpm: f32,
	pub beat_interval: f32, // time in seconds of 1 beat
	// !!! better location for brick speed? (inside brick struct so it isn't passed for every single brick? limitations?)
	pub brick_speed: f32,
	pub time_running: f32,
	pub score: i32,
	pub max_score: i32,
	pub duration: f32,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct BrickData {
	pub brick_type: BrickType,
	pub beat_pos: i32,
	pub end_beat_pos: i32,
	pub x_pos: i32,
	pub is_triplet: bool, // is a logic error if more than one of is_triplet, is_trailing, or is_leading is true
	pub is_trailing: bool,
	pub is_leading: bool,
	pub is_hold_note: bool
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
// fits within 32 bits
pub struct Graphic {
	pub g: GraphicGroup,
	pub frame: u8, // each frame adds 1 to frame mod 256. From timer javascript code chooses animation frame.
	pub flags: u8,
	pub arg: u8 // argument for if one of the flag requires it (opacity flag)
}

#[wasm_bindgen]
pub enum GraphicFlags {
	HorizontalFlip = 1,
	VerticalFlip = 2,
	Opacity = 4
}

#[wasm_bindgen]
pub struct RenderingInstructions {
	pub num_graphics: usize,
	pub graphics_ptr: *const PositionedGraphic
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct PositionedGraphic {
	pub g: Graphic,
	pub x: f32,
	pub y: f32,
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct Position {
	pub x: f32,
	pub y: f32
}

#[derive(Clone)]
pub struct LingeringGraphic {
	positioned_graphic: PositionedGraphic,
	start_t: f32,
	end_t: f32
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, EnumVariantCount)]
pub enum Input {
	Dash,
	Slash1,
	Slash2,
	Slash3,
}

#[wasm_bindgen]
pub fn ground_pos() -> i32 {
	return GROUND_POS as i32;
}

#[wasm_bindgen]
pub fn max_notes_per_screen_width() -> u8 {
	return objects::MAX_NOTES_PER_SCREEN_WIDTH;
}

#[wasm_bindgen]
pub fn time_zero_brick_pos() -> f32 {
	return TIME_ZERO_BRICK_POS;
}

#[wasm_bindgen]
pub fn game_dimensions() -> Position {
	Position {
		x: GAME_WIDTH as f32,
		y: GAME_HEIGHT as f32,
	}
}

#[wasm_bindgen]
pub fn player_dimensions() -> Position {
	return Position {
		x: objects::PLAYER_WIDTH as f32,
		y: objects::PLAYER_HEIGHT as f32
	}
}

#[wasm_bindgen]
pub fn brick_dimensions() -> Position {
	return Position {
		x: objects::BRICK_WIDTH as f32,
		y: objects::BRICK_HEIGHT as f32,
	};
}

fn frame_number(time_since_start: f32) -> u8 {
	return ((time_since_start / FRAME_TIME) % 256.0) as u8;
}

#[wasm_bindgen]
pub fn num_possible_inputs() -> usize {
	return Input::num_variants();
}

// --- methods and trait implementation ---

impl PositionedGraphic {
	pub fn new(g: Graphic, x: f32, y: f32) -> PositionedGraphic {
		let offset = resources::GRAPHIC_OFFSETS[g.g as usize];
		return PositionedGraphic {
			g,
			x: x - offset.x,
			y: y - offset.y
		}
	}
}

// equality and order are determined solely on the start position of the note and its x pos, 
	// not the brick type or whether it's a hold note or approximate time
impl PartialEq for BrickData {
	fn eq(&self, other: &BrickData) -> bool {
		return self.beat_pos == other.beat_pos && self.x_pos == other.x_pos
		&& self.is_triplet == other.is_triplet && self.is_trailing == other.is_trailing && self.is_leading == other.is_leading;
	}
}
impl Eq for BrickData {}

impl PartialOrd for BrickData {
	fn partial_cmp(&self, other: &BrickData) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for BrickData {
	fn cmp(&self, other: &BrickData) -> Ordering { // !!! !!! !!! why use appearance y rather than beat_pos?
		let self_top_y = self.appearance_y(60.0, 100.0); // dummy bpm and brick speed values
		let other_top_y = other.appearance_y(60.0, 100.0);
		if self_top_y < other_top_y { Ordering::Less }
		else if self_top_y > other_top_y { Ordering::Greater }
		else if self.x_pos < other.x_pos { Ordering::Less }
		else if self.x_pos > other.x_pos { Ordering::Greater }
		else { Ordering::Equal }
	}
}

#[wasm_bindgen]
impl BrickData {
	// here for construction of BrickData structures from javascript
	pub fn new(brick_type: BrickType, beat_pos: i32, end_beat_pos: i32, x_pos: i32, is_triplet: bool,
	is_trailing: bool, is_leading: bool, is_hold_note: bool) -> BrickData {
		
		return BrickData {
			brick_type,
			beat_pos,
			end_beat_pos,
			x_pos,
			is_triplet,
			is_trailing,
			is_leading,
			is_hold_note
		};
	}
	
	// the y value at which the note should appear. At time = 0 the top of the screen is y = 0
		// and a note that should be hit at time = 0 has appearance_y of TIME_ZERO_BRICK_POS
		// notes off the bottom of the screen have appearance_y's corresponding to how much has to be scrolled before they show up
	pub fn appearance_y(&self, bpm: f32, brick_speed: f32) -> f32 {
		let minutes_per_beat = 1.0 / bpm;
		let seconds_per_beat = 60.0 * minutes_per_beat;
		let pixels_per_beat = brick_speed * seconds_per_beat;
		let beats_passed = self.beat_pos as f32 / 4.0;
		
		let mut pixels_passed = pixels_per_beat * beats_passed;
		if self.is_leading {
			pixels_passed -= pixels_per_beat / 8.0;
		} else if self.is_trailing {
			pixels_passed += pixels_per_beat / 8.0;
		} else if self.is_triplet {
			
		}
		
		return pixels_passed + TIME_ZERO_BRICK_POS;
	}
	
	pub fn end_appearance_y(&self, bpm: f32, brick_speed: f32) -> f32 {
		let minutes_per_beat = 1.0 / bpm;
		let seconds_per_beat = 60.0 * minutes_per_beat;
		let pixels_per_beat = brick_speed * seconds_per_beat;
		let beats_passed = self.end_beat_pos as f32 / 4.0;
		
		// don't account for whether the note is a leading note or not, end on the last beat pos
		let pixels_passed = pixels_per_beat * beats_passed;
		
		return pixels_passed + TIME_ZERO_BRICK_POS;
	}
	
	pub fn x(&self) -> f32 {
		return (self.x_pos * objects::BRICK_WIDTH) as f32;
	}
	
	pub fn approx_time(beat_pos: i32, bpm: f32) -> f32 {
		let minutes_per_beat = 1.0 / bpm;
		let seconds_per_beat = 60.0 * minutes_per_beat;
		let beats_passed = beat_pos as f32 / 4.0;
		return seconds_per_beat * beats_passed;
	}
	
	pub fn closest_beat_pos(time: f32, bpm: f32) -> i32 {
		let minutes_per_beat = 1.0 / bpm;
		let seconds_per_beat = 60.0 * minutes_per_beat;
		
		let num_beats_passed = time / seconds_per_beat;
		let beat_pos_passed = num_beats_passed * 4.0;
		return beat_pos_passed.floor() as i32;
	}
}

// !!! logging
#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}