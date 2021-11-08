
use wasm_bindgen::prelude::*;

use macros::EnumVariantCount;

use crate::player;
use crate::objects;
use crate::Position;

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, EnumVariantCount)]
pub enum GraphicGroup {
	Background,
	Walking,
	Running,
	Slashing1,
	Slashing2,
	Slashing3,
	Brick1,
	Brick2,
	Brick3,
	Brick1Segment,
	Brick2Segment,
	Brick3Segment,
	Dash0,
	Dash1,
	Dash2,
	Dash3,
	PreHolding1,
	PreHolding2,
	PreHolding3,
	Holding1,
	Holding2,
	Holding3,
	Hold1,
	Hold2,
	Hold3
}

// !!! more robust way to determine the size of GraphicGroup than the (maybe) last enumeration + 1
pub const GRAPHIC_OFFSETS: [Position; GraphicGroup::Hold3 as usize + 1] = [
	Position {x: 0.0, y: 0.0},// Background,
	Position {x: 50.0, y: 25.0},// Walking,
	Position {x: 50.0, y: 25.0},// Running,
	Position {x: 80.0, y: 55.0},// Slashing1,
	Position {x: 80.0, y: 55.0},// Slashing2,
	Position {x: 80.0, y: 55.0},// Slashing3,
	Position {x: 40.0, y: 40.0},// Brick1,
	Position {x: 40.0, y: 40.0},// Brick2,
	Position {x: 40.0, y: 40.0},// Brick3,
	Position {x: 0.0, y: 0.0},// Brick1Segment,
	Position {x: 0.0, y: 0.0},// Brick2Segment,
	Position {x: 0.0, y: 0.0},// Brick3Segment,
	Position {x: 0.0, y: 0.0},// Dash0,
	Position {x: 0.0, y: 0.0},// Dash1,
	Position {x: 0.0, y: 0.0},// Dash2,
	Position {x: 0.0, y: 0.0},// Dash3,
	Position {x: 0.0, y: 0.0},// PreHolding1,
	Position {x: 0.0, y: 0.0},// PreHolding2,
	Position {x: 0.0, y: 0.0},// PreHolding3,
	Position {x: 50.0, y: 50.0},// Holding1,
	Position {x: 50.0, y: 50.0},// Holding2,
	Position {x: 50.0, y: 50.0},// Holding3,
	Position {x: 50.0, y: 50.0},// Hold1,
	Position {x: 50.0, y: 50.0},// Hold2,
	Position {x: 50.0, y: 50.0},// Hold3
];

pub const GRAPHIC_SIZES: [Position; GraphicGroup::Hold3 as usize + 1] = [
	Position {
		x: crate::GAME_WIDTH as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Background as usize].x, 
		y: crate::GAME_HEIGHT as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Background as usize].y},// Background,
	Position {
		x: objects::PLAYER_WIDTH as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Walking as usize].x, 
		y: objects::PLAYER_HEIGHT as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Walking as usize].y},// Walking,
	Position {
		x: objects::PLAYER_WIDTH as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Running as usize].x, 
		y: objects::PLAYER_HEIGHT as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Running as usize].y},// Running,
	Position {
		x: objects::PLAYER_WIDTH as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Slashing1 as usize].x, 
		y: objects::PLAYER_HEIGHT as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Slashing1 as usize].y},// Slashing1,
	Position {
		x: objects::PLAYER_WIDTH as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Slashing2 as usize].x, 
		y: objects::PLAYER_HEIGHT as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Slashing2 as usize].y},// Slashing2,
	Position {
		x: objects::PLAYER_WIDTH as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Slashing3 as usize].x, 
		y: objects::PLAYER_HEIGHT as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Slashing3 as usize].y},// Slashing3,
	Position {
		x: objects::BRICK_WIDTH as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Brick1 as usize].x, 
		y: objects::BRICK_HEIGHT as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Brick1 as usize].y},// Brick1,
	Position {
		x: objects::BRICK_WIDTH as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Brick2 as usize].x, 
		y: objects::BRICK_HEIGHT as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Brick2 as usize].y},// Brick2,
	Position {
		x: objects::BRICK_WIDTH as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Brick3 as usize].x, 
		y: objects::BRICK_HEIGHT as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Brick3 as usize].y},// Brick3,
	Position {
		x: objects::BRICK_WIDTH as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Brick1Segment as usize].x, 
		y: objects::BRICK_SEGMENT_HEIGHT as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Brick1Segment as usize].y},// Brick1Segment,
	Position {
		x: objects::BRICK_WIDTH as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Brick2Segment as usize].x, 
		y: objects::BRICK_SEGMENT_HEIGHT as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Brick2Segment as usize].y},// Brick2Segment,
	Position {
		x: objects::BRICK_WIDTH as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Brick3Segment as usize].x, 
		y: objects::BRICK_SEGMENT_HEIGHT as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Brick3Segment as usize].y},// Brick3Segment,
	Position {
		x: objects::MIN_DASH_WIDTH as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Dash0 as usize].x, 
		y: objects::DASH_HEIGHT as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Dash0 as usize].y},// Dash0,
	Position {
		x: objects::MIN_DASH_WIDTH as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Dash1 as usize].x, 
		y: objects::DASH_HEIGHT as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Dash1 as usize].y},// Dash1,
	Position {
		x: objects::MIN_DASH_WIDTH as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Dash2 as usize].x, 
		y: objects::DASH_HEIGHT as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Dash2 as usize].y},// Dash2,
	Position {
		x: objects::MIN_DASH_WIDTH as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Dash3 as usize].x, 
		y: objects::DASH_HEIGHT as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Dash3 as usize].y},// Dash3,
	Position {
		x: objects::PLAYER_WIDTH as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::PreHolding1 as usize].x, 
		y: objects::PLAYER_HEIGHT as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::PreHolding1 as usize].y},// PreHolding1,
	Position {
		x: objects::PLAYER_WIDTH as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::PreHolding2 as usize].x, 
		y: objects::PLAYER_HEIGHT as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::PreHolding2 as usize].y},// PreHolding2,
	Position {
		x: objects::PLAYER_WIDTH as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::PreHolding3 as usize].x, 
		y: objects::PLAYER_HEIGHT as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::PreHolding3 as usize].y},// PreHolding3,
	Position {
		x: objects::PLAYER_WIDTH as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Holding1 as usize].x, 
		y: objects::PLAYER_HEIGHT as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Holding1 as usize].y},// Holding1,
	Position {
		x: objects::PLAYER_WIDTH as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Holding2 as usize].x, 
		y: objects::PLAYER_HEIGHT as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Holding2 as usize].y},// Holding2,
	Position {
		x: objects::PLAYER_WIDTH as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Holding3 as usize].x, 
		y: objects::PLAYER_HEIGHT as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Holding3 as usize].y},// Holding3,
	Position {
		x: objects::HOLD_HITBOX_WIDTH as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Hold1 as usize].x, 
		y: objects::HOLD_HITBOX_HEIGHT as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Hold1 as usize].y},// Hold1,
	Position {
		x: objects::HOLD_HITBOX_WIDTH as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Hold2 as usize].x, 
		y: objects::HOLD_HITBOX_HEIGHT as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Hold2 as usize].y},// Hold2,
	Position {
		x: objects::HOLD_HITBOX_WIDTH as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Hold3 as usize].x, 
		y: objects::HOLD_HITBOX_HEIGHT as f32 + 2.0 * GRAPHIC_OFFSETS[GraphicGroup::Hold3 as usize].y},// Hold3
];

// the maximum amount of the graphic that can appear on screen
#[wasm_bindgen]
pub fn max_graphics(g: GraphicGroup) -> u32 {
	match g {
		GraphicGroup::Background => 1,
		GraphicGroup::Walking => 24,
		GraphicGroup::Running => 24,
		GraphicGroup::Slashing1 | GraphicGroup::Slashing2 | GraphicGroup::Slashing3 => 1,
		GraphicGroup::Brick1 | GraphicGroup::Brick2 | GraphicGroup::Brick3 => 32,
		GraphicGroup::Brick1Segment | GraphicGroup::Brick2Segment | GraphicGroup::Brick3Segment => 128,
		GraphicGroup::Dash0 | GraphicGroup::Dash1 | GraphicGroup::Dash2 | GraphicGroup::Dash3 => 10,
		GraphicGroup::PreHolding1 | GraphicGroup::PreHolding2 | GraphicGroup::PreHolding3
		| GraphicGroup::Holding1 | GraphicGroup::Holding2 | GraphicGroup::Holding3 => 1,
		GraphicGroup::Hold1 | GraphicGroup::Hold2 | GraphicGroup::Hold3 => 32
		
	}
}

// returns the intended size of different graphics
#[wasm_bindgen]
pub fn graphic_size(g: GraphicGroup) -> Position {
	return GRAPHIC_SIZES[g as usize];
}

#[wasm_bindgen]
pub fn num_graphic_groups() -> usize {
	return GraphicGroup::num_variants();
}

