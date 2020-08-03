
// TODO
// Log objects going beyond boundaries

use std::cmp::Ordering;
use wasm_bindgen::prelude::*;
use std::collections::vec_deque;

use crate::GraphicGroup;
use crate::PositionedGraphic;
use crate::GROUND_POS;
use crate::LEFT_BOUNDARY;
use crate::RIGHT_BOUNDARY;
use crate::TOP_BOUNDARY;
use crate::F32_ZERO;


pub const MAX_NOTES_PER_SCREEN_WIDTH: u8 = 32;
pub const PLAYER_WIDTH: u32 = 50;
pub const PLAYER_HEIGHT: u32 = 100; 
pub const BRICK_WIDTH: u32 = (RIGHT_BOUNDARY - LEFT_BOUNDARY) as u32 / MAX_NOTES_PER_SCREEN_WIDTH as u32;
pub const BRICK_HEIGHT: u32 = 120;
pub const SLASH_WIDTH: u32 = 65;
pub const SLASH_HEIGHT: u32 = 100;
pub const DASH_WIDTH: u32 = BRICK_WIDTH * 4 / 3;
pub const DASH_HEIGHT: u32 = 100;
pub const DASH_CD: f32 = 0.12;

pub trait Object {
	fn bounds (&self) -> ObjectBounds; // TODO copying the full object bounds may be extra work in some instances
}

// storing all bounds rather than pos+size because more comparisons between objects are possible than updates of positions
#[derive(Clone, Copy)]
pub struct ObjectBounds { 
	pub left_x: f32,
	pub right_x: f32,
	pub top_y: f32,
	pub bottom_y: f32
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
	Left,
	Right,
}

pub struct Player {
	graphic: GraphicGroup,
	
	bounds: ObjectBounds,
	dx: f32, // in pixels per second
	
	slash: Option<Slash>,
	dash: Option<Dash>,
	dash_dir: Direction,
	face_dir: Direction,
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BrickType {
	Type1,
	Type2,
	Type3
}

// !!! store brick type, match graphic to brick type
#[derive(Clone, Copy)]
pub struct Brick {
	brick_type: BrickType,
	time: f32,
	graphic: GraphicGroup,
	bounds: ObjectBounds,
}

pub enum TempObjectState {
	New(f32), // Stores how long into the tick the object was created, since inputs happen asynchronously with ticks
	Active(f32), // Stores how much longer the objct is active. May be negative, meaning it will become inactive on next check
	Lingering(f32) // Stores how much longer the object is lingering. If negative, object should be deleted
}

pub struct Slash {
	brick_type: BrickType,
	state: TempObjectState,
	graphic: GraphicGroup,
	bounds: ObjectBounds,
}

pub struct Dash {
	brick_type: Option<BrickType>,
	state: TempObjectState,
	graphic: Option<GraphicGroup>,
	direction: Direction,
	bounds: ObjectBounds,
}

pub fn intersect(obj1: &ObjectBounds, obj2: &ObjectBounds) -> bool {
	if obj1.top_y > obj2.bottom_y
	|| obj1.right_x < obj2.left_x
	|| obj1.left_x > obj2.right_x
	|| obj1.bottom_y < obj2.top_y {
		return false;
	}
	return true;
}

impl Object for Player {
	
	fn bounds(&self) -> ObjectBounds {
		self.bounds
	}
}
impl Player {
	
	pub fn new(x: f32) -> Player {
		Player {
			graphic: GraphicGroup::Player,
			bounds: ObjectBounds {
				left_x: x,
				top_y: GROUND_POS as f32 - PLAYER_HEIGHT as f32,
				right_x: x + PLAYER_WIDTH as f32, 
				bottom_y: GROUND_POS as f32
			},
			
			dx: 0.0,
			slash: None,
			dash: None,
			dash_dir: Direction::Right,
			face_dir: Direction::Right,
		}
	}
	
	pub fn slash (&mut self, brick_type: BrickType, t_since_tick: f32) {
		self.slash = Some(
			match self.face_dir {
				Direction::Right => {
					Slash::new( self.bounds.right_x, self.bounds.top_y, brick_type, t_since_tick, Direction::Right)
				}
				Direction::Left => {
					Slash::new( self.bounds.left_x, self.bounds.top_y, brick_type, t_since_tick, Direction::Left)
				}
			}
		);
	}
	pub fn dash (&mut self, t_since_tick: f32) {
		if let None = self.dash {
			if let Direction::Right = self.face_dir {
				self.dash = Some(Dash::new( self.bounds.right_x, self.bounds.top_y, t_since_tick, self.face_dir)); }
			else {
				self.dash = Some(Dash::new( self.bounds.left_x, self.bounds.top_y, t_since_tick, self.face_dir)); }}
	}
	
	pub fn face_direction(&self) -> &Direction {
		&self.face_dir
	}
	
	pub fn slashing(&self) -> &Option<Slash> {
		&self.slash
	}
	
	pub fn dashing(&self) -> &Option<Dash> {
		&self.dash
	}
	
	// tick the players state, taking into account input commands
	pub fn tick(&mut self, seconds_passed: f32, bricks_iter: vec_deque::Iter<Brick>, time_running: f32) {
		
		// TODO move a new slash unconditionally to player's position, so its position is set when it actually appears?
		// Tick slash. If it's new check if the dash is also new, in which case they link
		
		// link the dash and slash if they are both new
		let dash = &mut self.dash;
		let slash = &mut self.slash;
		match (slash, dash) {
			(Some(slash), Some(dash)) => {
				match (slash.state(), dash.state()) {
					(TempObjectState::New(slash_t), TempObjectState::New(dash_t)) => {
						// in case a movement input was immediately after 1 or both inputs
						let direction = self.dash_dir;
						// position slash to the end of the dash
						slash.link_dash_slash(dash.bounds.left_x, direction, *dash_t);
							
						dash.direction = direction;
						dash.brick_type = Some(slash.brick_type);
					},
					_ => ()
				}
			},
			_ => ()
		}
		
		// tick slash
		if let Some(slash) = &mut self.slash {
			match slash.state() {
				TempObjectState::Lingering(t) if *t < 0.0 => self.slash = None,
				_ => slash.tick(seconds_passed)
			}
		}
		
		// tick dash and/or move
		if let Some(dash) = &mut self.dash {
			match dash.state() {
				TempObjectState::Lingering(t) => {
					if *t < 0.0 {
						self.dash = None;
					}
					else {
						dash.tick(seconds_passed);
						self.regular_move(seconds_passed, bricks_iter, time_running);
					}
				},
				_ => {
					dash.tick(seconds_passed);
					if let Direction::Right = dash.direction() {
						self.bounds.left_x = dash.bounds.right_x;
						self.bounds.right_x = self.bounds.left_x + PLAYER_WIDTH as f32;
					}
					else {
						self.bounds.right_x = dash.bounds.left_x;
						self.bounds.left_x = self.bounds.right_x - PLAYER_WIDTH as f32;
					}
					self.dx = 0.0;
				}
			}
		}
		else {
			self.regular_move(seconds_passed, bricks_iter, time_running);
		}
	}
	
	fn regular_move(&mut self, seconds_passed: f32, mut bricks_iter: vec_deque::Iter<Brick>, time_running: f32) {
		const MAX_PLAYER_SPEED: f32 = 440.0;
		const MID_PLAYER_SPEED: f32 = 300.0;
		const MIN_PLAYER_SPEED: f32 = 100.0;
		
		// get data on upcoming bricks
		// TODO positions/times of upcoming bricks aren't changing as often as every tick
			// ever worth it to store some information/calculations in between ticks for the many instances they don't change?
		let mut upcoming_bricks_data = None;
		// this is the time threshold before which the player will go after a note
		// !!! smarter buffer determination + dash/slash (dash_dir) that goes after a brick if it's still hittable
		let time_with_buffer = time_running - 0.06; 
		while let Some(brick) = bricks_iter.next() {
			if brick.time() > time_with_buffer {
				let bricks_time = brick.time();
				let mut bricks_leftmost_x = brick.bounds().left_x;
				let mut bricks_rightmost_x = brick.bounds().right_x;
				while let Some(brick) = bricks_iter.next() {
					if brick.time() - bricks_time > F32_ZERO {
						break;
					}
					else if brick.bounds().left_x < bricks_leftmost_x {
						bricks_leftmost_x = brick.bounds().left_x;
					}
					// should be mutually exclusive from other else if statement
					else if brick.bounds().right_x > bricks_rightmost_x { 
						bricks_rightmost_x = brick.bounds().right_x;
					}
				}
				
				upcoming_bricks_data = Some( (bricks_time, bricks_leftmost_x, bricks_rightmost_x) );
				break;
			}
		}
		
		enum MovementType{
			Stopping,
			Turning,
			Running(f32), // specifies target speed (what speed is desired)
		}
		let movement: MovementType;
		let target_direction: Direction;
		let current_speed: f32; // absolute value of current dx
		let current_direction: Option<Direction>;
		let upcoming_bricks_time_until: Option<f32>;
		
		// get current speed and direction
		if self.dx < -F32_ZERO { 
			current_speed = -self.dx;
			current_direction = Some(Direction::Left);
		} else if self.dx > F32_ZERO { 
			current_speed = self.dx;
			current_direction = Some(Direction::Right);
		} else {
			current_speed = self.dx;
			current_direction = None;
		}
		
		// get movement type and target direction
		match upcoming_bricks_data {
			None => {
				movement = MovementType::Stopping;
				target_direction = Direction::Right;
				upcoming_bricks_time_until = None;
			}
			Some(brick_data) => {
				upcoming_bricks_time_until = Some(brick_data.0 - time_running);
				let upcoming_bricks_time_until = upcoming_bricks_time_until.unwrap();
				let upcoming_bricks_leftmost_x = brick_data.1;
				let upcoming_bricks_rightmost_x = brick_data.2;
				
				let mut distance_left = self.bounds.left_x - upcoming_bricks_rightmost_x;
				let mut distance_right = upcoming_bricks_leftmost_x - self.bounds.right_x;
				let mut distance_away;
				const separation_space: f32 = 8.0;
				
				if distance_left > 0.0 {
					target_direction = Direction::Left;
					distance_away = distance_left - separation_space;
					self.dash_dir = Direction::Left;
				} else if distance_right > 0.0 {
					target_direction = Direction::Right;
					distance_away = distance_right - separation_space;
					self.dash_dir = Direction::Right;
				} 
				else if distance_left > distance_right {
					target_direction = Direction::Right;
					distance_away = -distance_left + separation_space;
					if distance_away < BRICK_WIDTH as f32 {
						self.dash_dir = Direction::Left;
					} else {
						self.dash_dir = Direction::Right;
					}
				} else {
					target_direction = Direction::Left;
					distance_away = -distance_right + separation_space;
					if distance_away < BRICK_WIDTH as f32 {
						self.dash_dir = Direction::Right;
					} else {
						self.dash_dir = Direction::Left;
					}
				}
				
				let same_direction = if let Some(direction) = current_direction {
					direction == target_direction
				} else {
					true
				};
				
				if !same_direction {
					movement = MovementType::Turning;
				} else if distance_away < DASH_WIDTH as f32 {
					if current_speed * upcoming_bricks_time_until > distance_away {
						movement = MovementType::Stopping;
					}
					else {
						movement = MovementType::Running(distance_away / upcoming_bricks_time_until);
					}
				} else {
					// overshoot the distance, so that player will go faster than strictly necessary to reach brick, 
						// and then slow down once approaching the brick
					let mut target_speed = distance_away * 1.2 / upcoming_bricks_time_until;
					target_speed = 
						if target_speed > MAX_PLAYER_SPEED { 
							MAX_PLAYER_SPEED
						} else if target_speed < MIN_PLAYER_SPEED {
							MIN_PLAYER_SPEED
						} else {
							target_speed
						};
					movement = MovementType::Running(target_speed);
				}
			}
		}
		
		// accelerate/decelerate at a rate depending on how fast the player is currently moving
		let mut new_speed;
		if current_speed < MIN_PLAYER_SPEED {
			match movement {
				MovementType::Stopping | MovementType::Turning => {
					new_speed = current_speed - (3000.0 * seconds_passed);
					if new_speed < 0.0 { new_speed = 0.0 };
				}
				MovementType::Running(target_speed) => {
					let speed_difference = target_speed - current_speed;
					let mut acceleration = speed_difference * 3.0 / upcoming_bricks_time_until.unwrap();
					if acceleration > 1600.0 {
						acceleration = 1600.0;
					}
					new_speed = current_speed + (acceleration * seconds_passed);
					if (new_speed - current_speed) / speed_difference > 1.0 {
						new_speed = target_speed;
					}
				}
			}
		} else if current_speed < MID_PLAYER_SPEED {
			match movement {
				MovementType::Stopping | MovementType::Turning => {
					new_speed = current_speed - (2200.0 * seconds_passed);
				}
				MovementType::Running(target_speed) => {
					let speed_difference = target_speed - current_speed;
					let mut acceleration = speed_difference * 3.0 / upcoming_bricks_time_until.unwrap();
					if acceleration > 1200.0 {
						acceleration = 1200.0;
					}
					new_speed = current_speed + (acceleration * seconds_passed);
					if (new_speed - current_speed) / speed_difference > 1.0 {
						new_speed = target_speed;
					}
				}
			}
		} else {
			match movement {
				MovementType::Stopping | MovementType::Turning => {
					new_speed = current_speed - (2200.0 * seconds_passed);
				}
				MovementType::Running(target_speed) => {
					let speed_difference = target_speed - current_speed;
					let mut acceleration = speed_difference * 3.0 / upcoming_bricks_time_until.unwrap();
					if acceleration > 800.0 {
						acceleration = 800.0;
					}
					new_speed = current_speed + (acceleration * seconds_passed);
					if (new_speed - current_speed) / speed_difference > 1.0 {
						new_speed = target_speed;
					}
				}
			}
		}
		
		// set dx to the new speed in the correct direction
		if let Some(direction) = current_direction { 
			self.dx = match direction {
				Direction::Left => -new_speed,
				Direction::Right => new_speed
			}
		} else {
			match target_direction {
				Direction::Left => {
					self.dx = -new_speed;
					self.face_dir = Direction::Left;
				},
				Direction::Right => {
					self.dx = new_speed;
					self.face_dir = Direction::Right;
				}
			}
		};
		
		// calculate resulting position, while checking to not go past any boundaries
		self.bounds.left_x += self.dx * seconds_passed;
		self.bounds.right_x += self.dx * seconds_passed;
		if self.bounds.left_x < LEFT_BOUNDARY {
			self.bounds.right_x -= self.bounds.left_x - LEFT_BOUNDARY;
			self.bounds.left_x -= self.bounds.left_x - LEFT_BOUNDARY;
			self.dx = 0.0;
		} else if self.bounds.right_x > RIGHT_BOUNDARY {
			self.bounds.left_x -= self.bounds.right_x - RIGHT_BOUNDARY;
			self.bounds.right_x -= self.bounds.right_x - RIGHT_BOUNDARY;
			self.dx = 0.0;
		}
	}
	
	pub fn rendering_instruction(&self) -> PositionedGraphic {
		PositionedGraphic {
			g: self.graphic,
			x: self.bounds.left_x as i32,
			y: self.bounds.top_y as i32,
		}
	}
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
		let graphic = match brick_type {
			BrickType::Type1 => GraphicGroup::Brick,
			BrickType::Type2 => GraphicGroup::Brick2,
			BrickType::Type3 => GraphicGroup::Brick3
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
			x: self.bounds.left_x as i32,
			y: self.bounds.top_y as i32,
		}
	}
}


impl Object for Slash {
	fn bounds(&self) -> ObjectBounds {
		self.bounds
	}
}

impl Slash {
	pub fn new(x: f32, y: f32, brick_type: BrickType, t_since_tick: f32, dir: Direction) -> Slash {
		
		let graphic;
		let left_x;
		let right_x;
		
		match dir {
			Direction::Left => {
				left_x = x - SLASH_WIDTH as f32;
				right_x = x;
				
				match brick_type {
					BrickType::Type1 => graphic = GraphicGroup::SlashLeft,
					BrickType::Type2 => graphic = GraphicGroup::SlashLeft2,
					BrickType::Type3 => graphic = GraphicGroup::SlashLeft3
				}
			},
			Direction::Right => {
				left_x = x;
				right_x = x + SLASH_WIDTH as f32;
				
				match brick_type {
					BrickType::Type1 => graphic = GraphicGroup::SlashRight,
					BrickType::Type2 => graphic = GraphicGroup::SlashRight2,
					BrickType::Type3 => graphic = GraphicGroup::SlashRight3
				}
			}
		}
		
		return Slash {
			brick_type,
			state: TempObjectState::New(t_since_tick),
			graphic,
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
				self.graphic = match self.brick_type {
					BrickType::Type1 => GraphicGroup::SlashLeft,
					BrickType::Type2 => GraphicGroup::SlashLeft2,
					BrickType::Type3 => GraphicGroup::SlashLeft3
				}
			},
			Direction::Right => {
				self.bounds.left_x = x + DASH_WIDTH as f32 + PLAYER_WIDTH as f32;
				self.bounds.right_x = x + DASH_WIDTH as f32 + PLAYER_WIDTH as f32 + SLASH_WIDTH as f32;
				self.graphic = match self.brick_type {
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
				if effective_slash_t > -0.005 {
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

impl Object for Dash {
	fn bounds(&self) -> ObjectBounds {
		self.bounds
	}
}

impl Dash {
	pub fn new(x: f32, y: f32, t_since_tick: f32, dir: Direction) -> Dash {
		if let Direction::Right = dir {
			Dash {
				brick_type: None,
				state: TempObjectState::New(t_since_tick),
				graphic: None,
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
				graphic: None,
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
				// compromise between event register time and tick time
				let effective_slash_t = (-seconds_passed + *t) / 2.0;
				
				// delay dash activation by a tiny amount for reliable detection of simultaneous key presses
				if effective_slash_t > -0.005 {
					self.state = TempObjectState::New( effective_slash_t );
				}
				else {
					if let Direction::Right = self.direction {
						self.graphic = Some(GraphicGroup::Dash);
						self.bounds.right_x = self.bounds.left_x + DASH_WIDTH as f32;
					}
					else {
						self.graphic = Some(GraphicGroup::Dash);
						self.bounds.left_x = self.bounds.right_x - DASH_WIDTH as f32;
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
		// !!! if dash is always going to be instantaneous, self.graphic does not have to be Option type
			// because here is only 1 graphic (per brick type), and if the dash's state is new return None for this function
		match self.graphic {
			None => None,
			Some(g) => {
				Some(PositionedGraphic {
					g,
					x: self.bounds.left_x as i32,
					y: self.bounds.top_y as i32,
				})
			}
		}
	}
}
