
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
	Brick,
	Brick2,
	Brick3,
	SlashRight,
	SlashRight2,
	SlashRight3,
	SlashLeft,
	SlashLeft2,
	SlashLeft3,
	Dash0,
	Dash,
	Dash2,
	Dash3,
}

// the maximum amount of the graphic that can appear on screen
#[wasm_bindgen]
pub fn max_graphics(g: GraphicGroup) -> u32 {
	match g {
		GraphicGroup::Background => 1,
		GraphicGroup::Walking => 1,
		GraphicGroup::Running => 1,
		GraphicGroup::Brick | GraphicGroup::Brick2 | GraphicGroup::Brick3 => 32,
		GraphicGroup::SlashRight | GraphicGroup::SlashRight2 | GraphicGroup::SlashRight3
		| GraphicGroup::SlashLeft | GraphicGroup::SlashLeft2 | GraphicGroup::SlashLeft3 => 1,
		GraphicGroup::Dash0 => 1,
		GraphicGroup::Dash => 1,
		GraphicGroup::Dash2 => 1,
		GraphicGroup::Dash3 => 1,
		
	}
}

#[wasm_bindgen]
pub fn graphic_size(g: GraphicGroup) -> Position {
	return match g {
		GraphicGroup::Background => { Position {
			x: GAME_WIDTH as i32,
			y: GAME_HEIGHT as i32,
		}},
		GraphicGroup::Walking => { Position {
			x: objects::PLAYER_WIDTH as i32,
			y: objects::PLAYER_HEIGHT as i32,
		}},
		GraphicGroup::Running => { Position {
			x: objects::PLAYER_WIDTH as i32,
			y: objects::PLAYER_HEIGHT as i32,
		}},
		GraphicGroup::Brick | GraphicGroup::Brick2 | GraphicGroup::Brick3 => { Position {
			x: objects::BRICK_WIDTH as i32,
			y: objects::BRICK_HEIGHT as i32,
		}},
		GraphicGroup::SlashRight | GraphicGroup::SlashRight2 | GraphicGroup::SlashRight3
		| GraphicGroup::SlashLeft | GraphicGroup::SlashLeft2 | GraphicGroup::SlashLeft3 => { 
			Position {
				x: objects::SLASH_WIDTH as i32,
				y: objects::SLASH_HEIGHT as i32
			}
		},
		GraphicGroup::Dash0 | GraphicGroup::Dash | GraphicGroup::Dash2 | GraphicGroup::Dash3 => { Position {
			x: objects::DASH_WIDTH as i32,
			y: objects::DASH_HEIGHT as i32
		}},
	};
}

#[wasm_bindgen]
pub fn num_graphic_groups() -> usize {
	return GraphicGroup::num_variants();
}