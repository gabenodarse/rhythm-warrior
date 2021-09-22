
use crate::PositionedGraphic;
use crate::resources::GraphicGroup;
use crate::Graphic;

use crate::objects::Object;
use crate::objects::ObjectBounds;
use crate::objects::BrickType;
use crate::objects::TempObjectState;
use crate::objects::Direction;

use crate::objects::PLAYER_WIDTH;
use crate::objects::SLASH_HEIGHT;
use crate::objects::SLASH_WIDTH;
use crate::objects::DASH_WIDTH;

pub struct Slash {
	graphic: Graphic,
	state: TempObjectState,
	pub brick_type: BrickType,
	pub bounds: ObjectBounds,
}

impl Object for Slash {
	fn bounds(&self) -> ObjectBounds {
		self.bounds
	}
}

impl Slash {
	pub fn new(x: f32, y: f32, brick_type: BrickType, t_since_tick: f32, dir: Direction) -> Slash {
		
		let graphic_group;
		let frame = 0;
		let flags = 0;
		
		let left_x;
		let right_x;
		
		match dir {
			Direction::Left => {
				left_x = x - SLASH_WIDTH as f32;
				right_x = x;
				
				match brick_type {
					BrickType::Type1 => graphic_group = GraphicGroup::SlashLeft,
					BrickType::Type2 => graphic_group = GraphicGroup::SlashLeft2,
					BrickType::Type3 => graphic_group = GraphicGroup::SlashLeft3
				}
			},
			Direction::Right => {
				left_x = x;
				right_x = x + SLASH_WIDTH as f32;
				
				match brick_type {
					BrickType::Type1 => graphic_group = GraphicGroup::SlashRight,
					BrickType::Type2 => graphic_group = GraphicGroup::SlashRight2,
					BrickType::Type3 => graphic_group = GraphicGroup::SlashRight3
				}
			}
		}
		
		return Slash {
			brick_type,
			state: TempObjectState::New(t_since_tick),
			graphic: Graphic{ g: graphic_group, frame, flags },
			bounds: ObjectBounds {
				left_x,
				top_y: y,
				right_x,
				bottom_y: y + SLASH_HEIGHT as f32,
			}
		};
	}
	
	pub fn link_dash_slash(&mut self, x: f32, dir: Direction, sync_t: f32) {
		self.state = TempObjectState::New(sync_t);
		match dir {
			Direction::Left => {
				self.bounds.left_x = x - DASH_WIDTH as f32 - PLAYER_WIDTH as f32 - SLASH_WIDTH as f32;
				self.bounds.right_x = x - DASH_WIDTH as f32 - PLAYER_WIDTH as f32;
				self.graphic.g = match self.brick_type {
					BrickType::Type1 => GraphicGroup::SlashLeft,
					BrickType::Type2 => GraphicGroup::SlashLeft2,
					BrickType::Type3 => GraphicGroup::SlashLeft3
				}
			},
			Direction::Right => {
				self.bounds.left_x = x + DASH_WIDTH as f32 + PLAYER_WIDTH as f32;
				self.bounds.right_x = x + DASH_WIDTH as f32 + PLAYER_WIDTH as f32 + SLASH_WIDTH as f32;
				self.graphic.g = match self.brick_type {
					BrickType::Type1 => GraphicGroup::SlashRight,
					BrickType::Type2 => GraphicGroup::SlashRight2,
					BrickType::Type3 => GraphicGroup::SlashRight3
				}
			}
		}
	}
	
	pub fn state(&self) -> &TempObjectState {
		&self.state
	}
	
	pub fn brick_type(&self) -> BrickType {
		return self.brick_type;
	}
	
	pub fn tick(&mut self, seconds_passed: f32) {
		match &mut self.state {
			TempObjectState::New(t) => {
				// compromise between event register time and tick time
				let effective_slash_t = (-seconds_passed + *t) / 2.0;
				
				// delay slash activation by a tiny amount for reliable detection of simultaneous key presses
				if effective_slash_t > -0.008 {
					self.state = TempObjectState::New( effective_slash_t );
				}
				else {
					self.state = TempObjectState::Active( effective_slash_t );
				}
			},
			TempObjectState::Active(t) => self.state = TempObjectState::Lingering(0.06 + *t - seconds_passed),
			TempObjectState::Lingering(t) => self.state = TempObjectState::Lingering(*t - seconds_passed)
		}
	}
	
	pub fn rendering_instruction(&self) -> Option<PositionedGraphic> {
		if let TempObjectState::New(_) = self.state {
			return None;
		} else {
			return Some( PositionedGraphic {
				g: self.graphic,
				x: self.bounds.left_x as i32,
				y: self.bounds.top_y as i32,
			});
		}
		
	}
}