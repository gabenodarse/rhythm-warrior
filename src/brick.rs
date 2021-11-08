
use std::cmp::Ordering;

use crate::PositionedGraphic;
use crate::resources::GraphicGroup;
use crate::Graphic;

use crate::objects::Object;
use crate::objects::ObjectBounds;
use crate::objects::BrickType;

use crate::objects::BRICK_HEIGHT;
use crate::objects::BRICK_WIDTH;

#[derive(Clone, Copy)]
pub struct Brick {
	graphic: Graphic,
	pub brick_type: BrickType,
	pub bounds: ObjectBounds,
	pub is_hold_note: bool
}

impl PartialEq for Brick {
	fn eq(&self, other: &Brick) -> bool {
		self.bounds.left_x == other.bounds.left_x && self.bounds.top_y == other.bounds.top_y
		&& self.bounds.right_x == other.bounds.right_x && self.bounds.bottom_y == other.bounds.bottom_y
	}
}

impl Eq for Brick {}

impl PartialOrd for Brick {
	fn partial_cmp(&self, other: &Brick) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Brick {
	fn cmp(&self, other: &Brick) -> Ordering {
		if self.bounds.top_y < other.bounds.top_y      { Ordering::Less }
		else if self.bounds.top_y > other.bounds.top_y { Ordering::Greater }
		else                             { Ordering::Equal }
	}
}

impl Object for Brick {
	fn bounds(&self) -> ObjectBounds {
		self.bounds
	}
}

impl Brick {
	pub fn new (graphic_group: GraphicGroup, brick_type: BrickType, bounds: ObjectBounds, is_hold_note: bool) -> Brick {
		let frame = 0;
		let flags = 0;
		let arg = 0;
		let graphic = Graphic{ g: graphic_group, frame, flags, arg };
		
		return Brick {
			graphic,
			brick_type,
			bounds,
			is_hold_note
		};
	}
	
	pub fn brick_type(&self) -> BrickType {
		return self.brick_type;
	}
	
	pub fn rendering_instruction(&self) -> PositionedGraphic {
		return PositionedGraphic::new(self.graphic, self.bounds.left_x as f32, self.bounds.top_y as f32);
	}
}