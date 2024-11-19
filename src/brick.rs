
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
pub struct HittableBrick {
	brick_type: BrickType,
	x: f32,
	hold_segments: u8, // how many brick segments follow the brick
	parts_destroyed: u8 // the number of segments destroyed, with the first part being the brick itself
}

impl HittableBrick {
	pub fn new (brick_type: BrickType, x: f32, hold_segments: u8) -> HittableBrick {
		return HittableBrick {
			brick_type,
			x,
			hold_segments,
			parts_destroyed: 0
		};
	}
	
	// get the bounds of the brick given the top of the brick (or where the top was if the brick is a broken hold note)
	pub fn bounds(&self, top_y: f32) -> Option<ObjectBounds> {
		if self.parts_destroyed > self.hold_segments {
			return None;
		}
		
		let bounds_left_x;
		let bounds_top_y;
		let bounds_right_x;
		let bounds_bottom_y;
		if self.parts_destroyed == 0 {
			bounds_left_x = self.x;
			bounds_top_y = top_y;
			bounds_right_x = bounds_left_x + BRICK_WIDTH as f32;
			bounds_bottom_y = bounds_top_y + BRICK_HEIGHT as f32;
		}
		else {
			bounds_left_x = self.x + (BRICK_WIDTH as f32 - BRICK_SEGMENT_WIDTH as f32) / 2.0;
			bounds_top_y = top_y + BRICK_HEIGHT as f32 + BRICK_SEGMENT_GAP as f32 
				+ (BRICK_SEGMENT_HEIGHT as f32 + BRICK_SEGMENT_GAP as f32) * (self.parts_destroyed - 1) as f32;
			bounds_right_x = bounds_left_x + BRICK_SEGMENT_WIDTH as f32;
			bounds_bottom_y = bounds_top_y + BRICK_SEGMENT_HEIGHT as f32;
		}
		
		return Some( ObjectBounds {
			left_x: bounds_left_x,
			right_x: bounds_right_x,
			top_y: bounds_top_y,
			bottom_y: bounds_bottom_y
		});
	}
	
	pub fn x(&self) -> f32 {
		return self.x;
	}
	
	pub fn brick_type(&self) -> BrickType {
		return self.brick_type;
	}
	
	pub fn hold_segments(&self) -> u8 {
		return self.hold_segments;
	}

	pub fn parts_destroyed(&self) -> u8 {
		return self.parts_destroyed;
	}

	pub fn is_hold_segment(&self) -> bool {
		return if self.hold_segments > 0 && self.parts_destroyed > 0 { true } else { false };
	}
	
	pub fn is_hold_note(&self) -> bool {
		return if self.hold_segments > 0 { true } else { false };
	}
	
	// return true if the brick is broken completely
	pub fn is_broken(&self) -> bool {
		if self.parts_destroyed > self.hold_segments {
			return true;
		}
		
		return false;
	}

	// return true if the brick is broken completely
	pub fn attempt_break(&mut self) -> bool {
		self.parts_destroyed += 1;
		if self.parts_destroyed > self.hold_segments {
			return true;
		}
		
		return false;
	}
	
	// get the rendering instructions of the brick given the top of the brick (or where the top was if the brick is a broken hold note)
	pub fn rendering_instructions(&self, top_y: f32) -> Vec<PositionedGraphic> {
		let mut positioned_graphics = Vec::with_capacity(self.hold_segments as usize + 1);

		if self.parts_destroyed > self.hold_segments {
			return positioned_graphics;
		}
		
		let (brick_graphic_group, segment_graphic_group) = match self.brick_type {
			BrickType::Type1 => {(GraphicGroup::Brick1, GraphicGroup::Brick1Segment)},
			BrickType::Type2 => {(GraphicGroup::Brick2, GraphicGroup::Brick2Segment)},
			BrickType::Type3 => {(GraphicGroup::Brick3, GraphicGroup::Brick3Segment)}
		};

		let mut num_brick_segments;
		let segment_left_x = self.x + (BRICK_WIDTH as f32 - BRICK_SEGMENT_WIDTH as f32) / 2.0;
		let mut segment_top_y = top_y + BRICK_HEIGHT as f32 + BRICK_SEGMENT_GAP as f32;

		// push the brick if it hasn't been destroyed
		if self.parts_destroyed == 0 {
			let graphic = Graphic { g: brick_graphic_group, frame: 0, flags: 0, arg: 0 };
			positioned_graphics.push(PositionedGraphic::new(graphic, self.x, top_y));
			num_brick_segments = self.hold_segments;
		} else {
			num_brick_segments = self.hold_segments - self.parts_destroyed + 1;
			segment_top_y += (BRICK_SEGMENT_HEIGHT as f32 + BRICK_SEGMENT_GAP as f32) * (self.parts_destroyed - 1) as f32;
		}
		
		// push any segments
		let segment_graphic = Graphic { g: segment_graphic_group, frame: 0, flags: 0, arg: 0 };
		while num_brick_segments > 0 {
			positioned_graphics.push(PositionedGraphic::new(segment_graphic, segment_left_x, segment_top_y));

			num_brick_segments -= 1;
			segment_top_y += (BRICK_SEGMENT_HEIGHT + BRICK_SEGMENT_GAP) as f32;
		}

		return positioned_graphics;
	}
}