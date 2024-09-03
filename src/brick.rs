
use std::cmp::Ordering;

use crate::PositionedGraphic;
use crate::resources::GraphicGroup;
use crate::Graphic;

use crate::objects::Object;
use crate::objects::ObjectBounds;
use crate::objects::BrickType;

use crate::objects::BRICK_HEIGHT;
use crate::objects::BRICK_WIDTH;
use crate::objects::BRICK_SEGMENT_HEIGHT;
use crate::objects::BRICK_SEGMENT_WIDTH;
use crate::objects::BRICK_SEGMENT_GAP;

// struct for bricks that are already in the game with valid hitboxes
// the bounds describe the bounds of the brick, or the bounds of the topmost segment if it's a hold note
#[derive(Clone, Copy)]
pub struct Brick {
	pub brick_type: BrickType,
	pub bounds: ObjectBounds,
	pub appearance_y: f32, // the y value for when the brick appears, including how much the song has to scroll
	pub end_y: f32, // the y value for where the brick and all segments are done
	hold_segments: u8, // how many brick segments follow the brick
	parts_destroyed: u8 // the number of segments destroyed, with the first part being the brick itself
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
	pub fn new (brick_type: BrickType, bounds: ObjectBounds, appearance_y: f32, end_y: f32, hold_segments: u8) -> Brick {
		return Brick {
			brick_type,
			bounds,
			appearance_y,
			end_y,
			hold_segments,
			parts_destroyed: 0
		};
	}
	
	pub fn brick_type(&self) -> BrickType {
		return self.brick_type;
	}

	pub fn is_hold_segment(&self) -> bool {
		return if self.parts_destroyed == 0 { false } else { true };
	}

	pub fn is_hold_note(&self) -> bool {
		return if self.hold_segments > 0 { true } else { false };
	}

	// return true if the brick is broken completely
	pub fn attempt_break(&mut self) -> bool {
		self.parts_destroyed += 1;
		if self.parts_destroyed > self.hold_segments {
			return true;
		}
		self.bounds.bottom_y += (BRICK_SEGMENT_HEIGHT + BRICK_SEGMENT_GAP) as f32;
		self.bounds.top_y = self.bounds.bottom_y - BRICK_SEGMENT_HEIGHT as f32;
		// !!! !!! !!! left x and right x
		return false;
	}
	
	pub fn rendering_instructions(&self) -> Vec<PositionedGraphic> {
		let mut positioned_graphics = Vec::with_capacity(self.hold_segments as usize + 1);

		let (brick_graphic_group, segment_graphic_group) = match self.brick_type {
			BrickType::Type1 => {(GraphicGroup::Brick1, GraphicGroup::Brick1Segment)},
			BrickType::Type2 => {(GraphicGroup::Brick2, GraphicGroup::Brick2Segment)},
			BrickType::Type3 => {(GraphicGroup::Brick3, GraphicGroup::Brick3Segment)}
		};
		
		let mut num_brick_segments;

		// push the brick if it hasn't been destroyed
		if self.parts_destroyed == 0 {
			let graphic = Graphic { g: brick_graphic_group, frame: 0, flags: 0, arg: 0 };
			positioned_graphics.push(PositionedGraphic::new(graphic, self.bounds.left_x as f32, self.bounds.top_y as f32));
			num_brick_segments = self.hold_segments;
		} else {
			num_brick_segments = self.hold_segments - self.parts_destroyed + 1;
		}

		// push any segments
		let mut i = self.parts_destroyed + 1;
		let mut y = 
			if self.parts_destroyed == 0 { self.bounds.bottom_y + BRICK_SEGMENT_GAP as f32 }
			else { self.bounds.top_y };
		let x = self.bounds.left_x + (BRICK_WIDTH - BRICK_SEGMENT_WIDTH) as f32 / 2.0;
		let segment_graphic = Graphic { g: segment_graphic_group, frame: 0, flags: 0, arg: 0 };
		while num_brick_segments > 0 {
			positioned_graphics.push(PositionedGraphic::new(segment_graphic, x, y));

			num_brick_segments -= 1;
			y += (BRICK_SEGMENT_HEIGHT + BRICK_SEGMENT_GAP) as f32;
		}

		return positioned_graphics;
	}
}