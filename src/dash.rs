
use crate::PositionedGraphic;
use crate::resources::GraphicGroup;
use crate::Graphic;

use crate::objects::Object;
use crate::objects::ObjectBounds;
use crate::objects::BrickType;
use crate::objects::TempObjectState;
use crate::objects::Direction;

use crate::objects::DASH_CD;
use crate::objects::DASH_WIDTH;
use crate::objects::DASH_HEIGHT;

pub struct Dash {
	graphic: Graphic,
	state: TempObjectState,
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
	pub fn new(x: f32, y: f32, t_since_tick: f32, dir: Direction) -> Dash {
		let flags = 0;
		let frame = 0;
		let graphic = Graphic{ g: GraphicGroup::Dash0, frame, flags };
		
		if let Direction::Right = dir {
			Dash {
				brick_type: None,
				state: TempObjectState::New(t_since_tick),
				graphic,
				direction: Direction::Right,
				bounds: ObjectBounds {
					left_x: x,
					top_y: y,
					right_x: x,
					bottom_y: y + DASH_HEIGHT as f32,
				}
			}
		} 
		else {
			Dash {
				brick_type: None,
				state: TempObjectState::New(t_since_tick),
				graphic,
				direction: Direction::Left,
				bounds: ObjectBounds {
					left_x: x,
					top_y: y,
					right_x: x,
					bottom_y: y + DASH_HEIGHT as f32,
				}
			}
		}
	}
	
	pub fn direction (&self) -> Direction {
		self.direction	
	}
	
	pub fn state (&self) -> &TempObjectState {
		&self.state
	}
	
	pub fn tick(&mut self, seconds_passed: f32) {
		match &mut self.state {
			TempObjectState::New(t) => {
				let flags = 0;
				// compromise between event register time and tick time
				let effective_slash_t = (-seconds_passed + *t) / 2.0;
				
				// delay dash activation by a tiny amount for reliable detection of simultaneous key presses
				if effective_slash_t > -0.008 {
					self.state = TempObjectState::New( effective_slash_t );
				}
				else {
					match self.direction {
						Direction::Right => self.bounds.right_x = self.bounds.left_x + DASH_WIDTH as f32,
						Direction::Left => self.bounds.left_x = self.bounds.right_x - DASH_WIDTH as f32
					}
					match self.brick_type {
						None => self.graphic.g = GraphicGroup::Dash0,
						Some(brick_type) => {
							match brick_type {
								BrickType::Type1 => self.graphic.g = GraphicGroup::Dash,
								BrickType::Type2 => self.graphic.g = GraphicGroup::Dash2,
								BrickType::Type3 => self.graphic.g = GraphicGroup::Dash3
							}
						}
					}
					
					self.state = TempObjectState::Active( effective_slash_t );
				}
			},
			TempObjectState::Active(t) => {
				// update bounds and graphic
				self.state = TempObjectState::Lingering(*t - seconds_passed + DASH_CD);
			},
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