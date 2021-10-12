
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
	pub time: f32,
	pub bounds: ObjectBounds,
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
	pub fn new (brick_type: BrickType, x: f32, y: f32, t: f32) -> Brick {
		let frame = 0;
		let flags = 0;
		let arg = 0;
		let graphic = match brick_type {
			BrickType::Type1 => Graphic{ g: GraphicGroup::Brick1, frame, flags, arg },
			BrickType::Type2 => Graphic{ g: GraphicGroup::Brick2, frame, flags, arg },
			BrickType::Type3 => Graphic{ g: GraphicGroup::Brick3, frame, flags, arg }
		};
		
		return Brick {
			brick_type,
			time: t,
			graphic,
			bounds: ObjectBounds {
				left_x: x,
				top_y: y,
				right_x: x + BRICK_WIDTH as f32,
				bottom_y: y + BRICK_HEIGHT as f32,
			}
		};
	}
	
	pub fn tick(&mut self, brick_speed: f32, seconds_passed: f32) {
		self.bounds.top_y -= brick_speed * seconds_passed;
		self.bounds.bottom_y -= brick_speed * seconds_passed;
	}
	
	pub fn time(&self) -> f32 {
		return self.time;
	}
	
	pub fn brick_type(&self) -> BrickType {
		return self.brick_type;
	}
	
	pub fn rendering_instruction(&self) -> PositionedGraphic {
		PositionedGraphic {
			g: self.graphic,
			x: self.bounds.left_x as f32,
			y: self.bounds.top_y as f32,
		}
	}
}