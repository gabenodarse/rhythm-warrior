
use crate::PositionedGraphic;
use crate::resources::GraphicGroup;
use crate::Graphic;

use crate::objects::Object;
use crate::objects::ObjectBounds;
use crate::objects::BrickType;
use crate::objects::Direction;

use crate::objects::DASH_WIDTH;
use crate::objects::DASH_HEIGHT;

#[derive(Clone)]
pub struct Dash {
	pub brick_type: Option<BrickType>,
	pub direction: Direction,
	pub bounds: ObjectBounds,
}

impl Object for Dash {
	fn bounds(&self) -> ObjectBounds {
		self.bounds
	}
}

impl Dash {
	pub fn new(x: f32, y: f32, brick_type: Option<BrickType>, dir: Direction) -> Dash {
		
		if let Direction::Right = dir {
			Dash {
				brick_type,
				direction: Direction::Right,
				bounds: ObjectBounds {
					left_x: x,
					top_y: y,
					right_x: x + DASH_WIDTH as f32,
					bottom_y: y + DASH_HEIGHT as f32,
				}
			}
		} 
		else {
			Dash {
				brick_type,
				direction: Direction::Left,
				bounds: ObjectBounds {
					left_x: x,
					top_y: y,
					right_x: x + DASH_WIDTH as f32,
					bottom_y: y + DASH_HEIGHT as f32,
				}
			}
		}
	}
	
	pub fn rendering_instruction(&self) -> PositionedGraphic {
		let g;
		let flags = 0;
		let frame = 0;
			match self.brick_type {
				None => g = Graphic{ g: GraphicGroup::Dash0, frame: 0, flags: 0 },
				Some(bt) => {
					match bt {
						BrickType::Type1 => g = Graphic{ g: GraphicGroup::Dash1, frame, flags },
						BrickType::Type2 => g = Graphic{ g: GraphicGroup::Dash2, frame, flags },
						BrickType::Type3 => g = Graphic{ g: GraphicGroup::Dash3, frame, flags }
					}
				}
			}
		
		
		return PositionedGraphic {
			g,
			x: self.bounds.left_x as i32,
			y: self.bounds.top_y as i32,
		};
	}
}