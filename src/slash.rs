
use crate::PositionedGraphic;
use crate::resources::GraphicGroup;
use crate::Graphic;
use crate::GraphicFlags;

use crate::objects::Object;
use crate::objects::ObjectBounds;
use crate::objects::BrickType;
use crate::objects::Direction;

use crate::objects::PLAYER_WIDTH;
use crate::objects::SLASH_HEIGHT;
use crate::objects::SLASH_WIDTH;
use crate::objects::DASH_WIDTH;

pub struct Slash {
	pub direction: Direction,
	pub brick_type: BrickType,
	pub bounds: ObjectBounds,
}

impl Object for Slash {
	fn bounds(&self) -> ObjectBounds {
		self.bounds
	}
}

impl Slash {
	pub fn new(x: f32, y: f32, brick_type: BrickType, direction: Direction) -> Slash {
		
		return Slash {
			brick_type,
			direction,
			bounds: ObjectBounds {
				left_x: x,
				top_y: y,
				right_x: x + SLASH_WIDTH as f32,
				bottom_y: y + SLASH_HEIGHT as f32,
			}
		};
	}
	
	pub fn rendering_instruction(&self) -> PositionedGraphic {
		let graphic_group;
		let frame = 0;
		let flags;
		match self.direction {
			Direction::Left => {
				match self.brick_type {
					BrickType::Type1 => graphic_group = GraphicGroup::Slash1,
					BrickType::Type2 => graphic_group = GraphicGroup::Slash2,
					BrickType::Type3 => graphic_group = GraphicGroup::Slash3
				}
				flags = GraphicFlags::HorizontalFlip as u8;
			},
			Direction::Right => {
				match self.brick_type {
					BrickType::Type1 => graphic_group = GraphicGroup::Slash1,
					BrickType::Type2 => graphic_group = GraphicGroup::Slash2,
					BrickType::Type3 => graphic_group = GraphicGroup::Slash3
				}
				flags = 0;
			}
		}
		
		return PositionedGraphic {
			g: Graphic{ g: graphic_group, frame, flags },
			x: self.bounds.left_x as i32,
			y: self.bounds.top_y as i32,
		};
	}
}