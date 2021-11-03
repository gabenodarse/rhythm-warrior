
use wasm_bindgen::prelude::*;

use macros::EnumVariantCount;

use crate::player;
use crate::objects;
use crate::Position;
use crate::GAME_WIDTH;
use crate::GAME_HEIGHT;

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
	return match g {
		GraphicGroup::Background => { Position {
			x: GAME_WIDTH as f32,
			y: GAME_HEIGHT as f32,
		}},
		GraphicGroup::Walking => { Position {
			x: objects::PLAYER_WIDTH as f32,
			y: objects::PLAYER_HEIGHT as f32,
		}},
		GraphicGroup::Running => { Position {
			x: objects::PLAYER_WIDTH as f32,
			y: objects::PLAYER_HEIGHT as f32,
		}},
		GraphicGroup::Slashing1 | GraphicGroup::Slashing2 | GraphicGroup::Slashing3 => { Position {
			x: objects::PLAYER_WIDTH as f32 + objects::SLASH_WIDTH as f32,
			y: objects::PLAYER_HEIGHT as f32,
		}},
		GraphicGroup::Brick1 | GraphicGroup::Brick2 | GraphicGroup::Brick3 => { Position {
			x: objects::BRICK_WIDTH as f32,
			y: objects::BRICK_HEIGHT as f32,
		}},
		GraphicGroup::Brick1Segment | GraphicGroup::Brick2Segment | GraphicGroup::Brick3Segment => { Position {
			x: objects::BRICK_WIDTH as f32,
			y: objects::BRICK_SEGMENT_HEIGHT as f32,
		}},
		GraphicGroup::Dash0 | GraphicGroup::Dash1 | GraphicGroup::Dash2 | GraphicGroup::Dash3 => { Position {
			x: objects::MIN_DASH_WIDTH as f32,
			y: objects::DASH_HEIGHT as f32
		}},
		GraphicGroup::Hold1 | GraphicGroup::Hold2 | GraphicGroup::Hold3 => { Position {
			x: objects::HOLD_HITBOX_WIDTH as f32,
			y: objects::HOLD_HITBOX_HEIGHT as f32
		}},
		GraphicGroup::PreHolding1 | GraphicGroup::PreHolding2 | GraphicGroup::PreHolding3
		| GraphicGroup::Holding1 | GraphicGroup::Holding2 | GraphicGroup::Holding3 => { Position {
			x: objects::PLAYER_WIDTH as f32 + objects::SLASH_WIDTH as f32,
			y: objects::PLAYER_HEIGHT as f32 + objects::SLASH_WIDTH as f32,
		}}
	};
}

#[wasm_bindgen]
pub fn num_graphic_groups() -> usize {
	return GraphicGroup::num_variants();
}