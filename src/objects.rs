
// TODO
// Log objects going beyond boundaries

use std::cmp::Ordering;
use wasm_bindgen::prelude::*;

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

#[derive(Clone, Copy)]
pub enum Direction {
	Left,
	Right,
}

pub struct Player {
	graphic: GraphicGroup,
	
	dash_time: f32,
	
	bounds: ObjectBounds,
	dx: f32, // in pixels per second
	
	slash: Option<Slash>,
	dash: Option<Dash>,
	moving_left: bool,
	moving_right: bool,
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
			dash_time: 0.01,
			bounds: ObjectBounds {
				left_x: x,
				top_y: GROUND_POS as f32 - PLAYER_HEIGHT as f32,
				right_x: x + PLAYER_WIDTH as f32, 
				bottom_y: GROUND_POS as f32
			},
			
			dx: 0.0,
			slash: None,
			dash: None,
			moving_left: false,
			moving_right: false,
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
	pub fn move_left (&mut self) {
		self.moving_left = true;
		self.face_dir = Direction::Left;
	}
	pub fn move_right (&mut self) {
		self.moving_right = true;
		self.face_dir = Direction::Right;
	}
	pub fn stop_left (&mut self) {
		self.moving_left = false;
		if self.moving_right {
			self.face_dir = Direction::Right;
		}
	}
	pub fn stop_right (&mut self) {
		self.moving_right = false;
		if self.moving_left {
			self.face_dir = Direction::Left;
		}
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
	pub fn tick(&mut self, seconds_passed: f32, upcoming_bricks_time_until: f32, 
	upcoming_bricks_leftmost_x: f32, upcoming_bricks_rightmost_x: f32) {
		
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
						let direction = self.face_dir;
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
				TempObjectState::Lingering(t) if *t < 0.0 => {
					self.dash = None;
					self.regular_move(seconds_passed, upcoming_bricks_time_until, 
						upcoming_bricks_leftmost_x, upcoming_bricks_rightmost_x);
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
			self.regular_move(seconds_passed, upcoming_bricks_time_until, 
				upcoming_bricks_leftmost_x, upcoming_bricks_rightmost_x);
		}
	}
	
	fn regular_move(&mut self, seconds_passed: f32, upcoming_bricks_time_until: f32, 
	upcoming_bricks_leftmost_x: f32, upcoming_bricks_rightmost_x: f32) {
		const MAX_PLAYER_SPEED: f32 = 400.0;
		const MIN_PLAYER_SPEED: f32 = 100.0;
		
		let mut target_speed; // absolute value of velocity dx needed to reach the brick in time to hit it
		let same_direction; // whether player has a dx in the same direction as the target direction
		let current_speed; // absolute value of current dx
		
		// calculate target speed
		// !!! calculate target speed based on integral of current speed from now until upcoming brick, and use this better
			// target speed to command a finer grainer / better feeling acceleration
		// TODO positions/times of upcoming bricks aren't changing as often as every tick
			// ever worth it to store some information/calculations in between ticks for the many instances they don't change?
		if self.moving_right ^ self.moving_left {
			// the order of the direction check in both directions is important due to how acceleration is handled for a self.dx == 0
				// this can be made less flimsy by adding a single extra calculation check for self.dx == 0.0
				// in which same_direction is false when moving left and true when moving right
			// if the distance needed is sufficiently far, specify a target speed to overshoot the distance, so that player
				// will go faster than strictly necessary to reach brick, and then slow down once approaching the brick
			let mut distance_away;
			if self.moving_left {
				if upcoming_bricks_rightmost_x < self.bounds.left_x {
					distance_away = (self.bounds.left_x - upcoming_bricks_rightmost_x);
				} else { // else get to the left of the leftmost brick
					distance_away = (self.bounds.right_x - upcoming_bricks_leftmost_x);
				}
				
				if self.dx < 0.0 { 
					same_direction = true;
					current_speed = -self.dx;
					if distance_away > DASH_WIDTH as f32 { 
						distance_away += PLAYER_WIDTH as f32;
					}
				} else { 
					same_direction = false;
					current_speed = self.dx;
					distance_away += PLAYER_WIDTH as f32;
				}
			}
			else {
				if upcoming_bricks_leftmost_x > self.bounds.right_x {
					distance_away = (upcoming_bricks_leftmost_x - self.bounds.right_x);
				} else { // else get to the right of the rightmost brick
					distance_away = (upcoming_bricks_rightmost_x - self.bounds.left_x);
				}
				
				if self.dx < 0.0 { 
					same_direction = false;
					current_speed = -self.dx;
					distance_away += PLAYER_WIDTH as f32;
				} else { 
					same_direction = true;
					current_speed = self.dx;
					if distance_away > DASH_WIDTH as f32 { 
						distance_away += PLAYER_WIDTH as f32;
					}
				};
			}
			
			target_speed = distance_away / upcoming_bricks_time_until;
			if target_speed > MAX_PLAYER_SPEED {
				target_speed = MAX_PLAYER_SPEED;
				// TODO go slightly over max speed depending on how far away brick is?
			} else if target_speed < MIN_PLAYER_SPEED {
				target_speed = MIN_PLAYER_SPEED;
			}
		}
		else {
			target_speed = 0.0;
			same_direction = true;
			if self.dx > 0.0 { 
				current_speed = self.dx;
			} else {
				current_speed = -self.dx;
			};
		}
		
		// accelerate/decelerate
		if target_speed - current_speed > F32_ZERO || current_speed - target_speed > F32_ZERO || !same_direction {
			let speeding_up = if (target_speed > current_speed) && same_direction { true } else { false };
			let mut new_speed;
			if current_speed < MIN_PLAYER_SPEED {
				if speeding_up {
					new_speed = current_speed + (800.0 * seconds_passed);
				} else {
					new_speed = current_speed - (1600.0 * seconds_passed);
				}
			}
			else {
				if speeding_up {
					new_speed = current_speed + (600.0 * seconds_passed);
				} else {
					new_speed = current_speed - (1200.0 * seconds_passed);
				}
			}
			if speeding_up {
				if new_speed > target_speed {
					new_speed = target_speed;
				}
			} else if same_direction && new_speed < target_speed {
				new_speed = target_speed;
			}
			
			self.dx = if self.dx < 0.0 { -new_speed} else { new_speed }
		}
		
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
			TempObjectState::Active(t) => self.state = TempObjectState::Lingering(0.1 + *t - seconds_passed),
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
