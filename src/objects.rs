
use wasm_bindgen::prelude::*;

use crate::LEFT_BOUNDARY;
use crate::RIGHT_BOUNDARY;

pub const MAX_NOTES_PER_SCREEN_WIDTH: u8 = 32;
pub const PLAYER_WIDTH: u32 = 50;
pub const PLAYER_HEIGHT: u32 = 100; 
pub const BRICK_WIDTH: u32 = (RIGHT_BOUNDARY - LEFT_BOUNDARY) as u32 / MAX_NOTES_PER_SCREEN_WIDTH as u32;
pub const BRICK_HEIGHT: u32 = 100;
pub const SLASH_WIDTH: u32 = 60;
pub const SLASH_HEIGHT: u32 = PLAYER_HEIGHT * 9 / 10;
pub const DASH_WIDTH: u32 = BRICK_WIDTH * 3; // >:< remove as constant
pub const MIN_DASH_WIDTH: u32 = 50;
pub const DASH_HEIGHT: u32 = SLASH_HEIGHT;
pub const DASH_CD: f32 = 0.12;
pub const NUM_MOVEMENT_FRAMES: u8 = 23;
pub const BRICK_DATA_BUFFER_SIZE: usize = 4;


pub trait Object {
	fn bounds (&self) -> ObjectBounds; // TODO copying the full object bounds may be extra work in some instances
}

// storing all bounds rather than pos+size because more comparisons between objects are possible than updates of positions
#[derive(Clone, Copy)]
pub struct ObjectBounds { 
	pub left_x: f32,
	pub right_x: f32,
	pub top_y: f32,
	pub bottom_y: f32
}

pub struct HitBox {
	pub bounds: ObjectBounds,
	pub brick_type: BrickType
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
	Left,
	Right,
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BrickType {
	Type1,
	Type2,
	Type3
}

// checks if two object bounds intersect
pub fn intersect(obj1: &ObjectBounds, obj2: &ObjectBounds) -> bool {
	if obj1.top_y > obj2.bottom_y
	|| obj1.right_x < obj2.left_x
	|| obj1.left_x > obj2.right_x
	|| obj1.bottom_y < obj2.top_y {
		return false;
	}
	return true;
}

