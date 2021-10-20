
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

#[allow(dead_code)]

mod objects;
mod resources;
mod player;
mod brick;
mod game;

use std::collections::btree_set::BTreeSet; 
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
const MAX_TIME_BETWEEN_TICKS: f32 = 0.025;
const FRAME_TIME: f32 = 0.00833; // 60 fps

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

#[derive(Clone, Copy)]
pub struct UpcomingNote {
	note_type: BrickType,
	x: f32,
	time: f32, // time the note is meant to be played
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
pub fn ground_pos() -> f32 {
	return GROUND_POS as f32;
}

// converts a note pos (discrete integer) to an x valued float
fn note_pos_to_x(pos: u8) -> f32 {
		let pos = match pos >= objects::MAX_NOTES_PER_SCREEN_WIDTH {
			true => objects::MAX_NOTES_PER_SCREEN_WIDTH - 1,
			false => pos
		};
		
		return (objects::BRICK_WIDTH * pos as i32) as f32;
	}
	
// converts a note x to a note pos (discrete integer)
fn note_pos_from_x(x: f32) -> u8 {
	let pos = (x / objects::BRICK_WIDTH as f32) as u8;
	let pos = match pos >= objects::MAX_NOTES_PER_SCREEN_WIDTH {
		true => objects::MAX_NOTES_PER_SCREEN_WIDTH - 1,
		false => pos
	};
	
	return pos;
}

fn frame_number(time_since_start: f32) -> u8 {
	return ((time_since_start / FRAME_TIME) % 256.0) as u8;
}

#[wasm_bindgen]
pub fn num_possible_inputs() -> usize {
	return Input::num_variants();
}

// --- trait implementations ---

impl PartialEq for UpcomingNote {
	fn eq(&self, other: &UpcomingNote) -> bool {
		self.note_type == other.note_type
		&& self.x == other.x
		&& self.time - other.time < F32_ZERO
		&& other.time - self.time < F32_ZERO
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
		if other.time - self.time > F32_ZERO      { Ordering::Less }
		else if self.time - other.time > F32_ZERO { Ordering::Greater }
		// arbitrary comparisons so that notes of the same time can exist within the same set
		else if (self.note_type as u8) < (other.note_type as u8) { Ordering::Less }
		else if (self.note_type as u8) > (other.note_type as u8) { Ordering::Greater }
		else if self.x < other.x { Ordering::Less }
		else if self.x > other.x { Ordering::Greater }
		else { Ordering::Equal }
	}
}

// !!! logging
#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}