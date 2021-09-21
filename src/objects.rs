
use std::cmp::Ordering;
use wasm_bindgen::prelude::*;
use std::collections::VecDeque;
use std::collections::vec_deque;

use crate::note_pos_from_x; // !!! calculates note pos back from x when it would be better to store note pos in each brick
use crate::GraphicGroup;
use crate::Graphic;
use crate::GraphicFlags;
use crate::PositionedGraphic;
use crate::GROUND_POS;
use crate::LEFT_BOUNDARY;
use crate::RIGHT_BOUNDARY;
use crate::TOP_BOUNDARY;
use crate::F32_ZERO;
use crate::MAX_TIME_BETWEEN_TICKS;
use crate::log;

pub const MAX_NOTES_PER_SCREEN_WIDTH: u8 = 32;
pub const PLAYER_WIDTH: u32 = 50;
pub const PLAYER_HEIGHT: u32 = 100; 
pub const BRICK_WIDTH: u32 = (RIGHT_BOUNDARY - LEFT_BOUNDARY) as u32 / MAX_NOTES_PER_SCREEN_WIDTH as u32;
pub const BRICK_HEIGHT: u32 = 100;
pub const SLASH_WIDTH: u32 = 60;
pub const SLASH_HEIGHT: u32 = 90;
pub const DASH_WIDTH: u32 = BRICK_WIDTH; // !!! remove as constant
pub const DASH_HEIGHT: u32 = PLAYER_HEIGHT * 9 / 10;
pub const DASH_CD: f32 = 0.12;
pub const NUM_MOVEMENT_FRAMES: u8 = 23;
pub const BRICK_DATA_BUFFER_SIZE: usize = 4;


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
	graphic: Graphic, // !!! all objects store Graphic
	movement_frame: u8,
	movement_frame_t: f32,
	
	bounds: ObjectBounds,
	dx: f32, // in pixels per second
	
	movement: Option<Movement>,
	slash: Option<Slash>,
	dash: Option<Dash>,
	hit_dir: Direction,
	face_dir: Direction
}

#[derive(Clone, Copy)]
enum MovementType {
	Constant(f32), // speed
	Accelerating,
	Decelerating,
}

#[derive(Clone)]
struct MovementInterval {
	movement_type: MovementType,
	direction: Direction,
	effective_start_t: f32, // start_t if starting at 0 speed (accelerating) or MAX_SPEED (decelerating)
	effective_start_x: f32, // start_pos if starting at 0 speed / MAX_SPEED
	end_t: f32
} 

struct Movement {
	intervals: Vec<MovementInterval>,
	current_interval: usize
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
	graphic: Graphic,
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
	graphic: Graphic,
	bounds: ObjectBounds,
}

pub struct Dash {
	brick_type: Option<BrickType>,
	state: TempObjectState,
	graphic: Graphic,
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

impl std::fmt::Display for Movement {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let mut interval_id = self.current_interval;
        while interval_id < self.intervals.len() {
			let interval = &self.intervals[interval_id];
			
			write!(f, "Interval {}\n", interval_id)?;
			match interval.movement_type {
				MovementType::Constant(s) => write!(f, "movement_type: Constant({})\n", s)?,
				MovementType::Accelerating => write!(f, "movement_type: Accelerating\n")?,
				MovementType::Decelerating => write!(f, "movement_type: Decelerating\n")?
			}
			match interval.direction {
				Direction::Left => write!(f, "direction: Left\n")?,
				Direction::Right => write!(f, "direction: Right\n")?
			}
			write!(f, "Effective Start T: {}\n", interval.effective_start_t)?;
			write!(f, "Effective Start X: {}\n", interval.effective_start_x)?;
			write!(f, "End T: {}\n\n", interval.end_t)?;
			
			interval_id += 1;
		}
		
		return Ok(());
    }
}

impl Object for Player {
	
	fn bounds(&self) -> ObjectBounds {
		self.bounds
	}
}
impl Player {
	
	pub fn new(x: f32) -> Player {
		Player {
			graphic: Graphic { g: GraphicGroup::Walking, frame: 0, flags: 0 },
			bounds: ObjectBounds {
				left_x: x,
				top_y: GROUND_POS as f32 - PLAYER_HEIGHT as f32,
				right_x: x + PLAYER_WIDTH as f32, 
				bottom_y: GROUND_POS as f32
			},
			
			dx: 0.0,
			movement: None,
			slash: None,
			dash: None,
			hit_dir: Direction::Right,
			face_dir: Direction::Right,
			movement_frame: 0,
			movement_frame_t: 0.0
		}
	}
	
	pub fn slash (&mut self, brick_type: BrickType, t_since_tick: f32) {
		self.slash = Some(
			match self.hit_dir {
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
			if let Direction::Right = self.hit_dir {
				self.dash = Some(Dash::new( self.bounds.right_x, self.bounds.top_y, t_since_tick, self.hit_dir)); }
			else {
				self.dash = Some(Dash::new( self.bounds.left_x, self.bounds.top_y, t_since_tick, self.hit_dir)); }}
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
						let direction = self.hit_dir;
						
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
					}
					self.regular_move(seconds_passed, bricks_iter, time_running);
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
		// get nearest brick
		// run to it
		// make deterministic based on time of initial calculation
		// !!! do calculations before game starts
		
		struct TargetInfo {
			time: f32,
			left_brick: f32, // x position of leftmost brick
			right_brick: f32 // x position of rightmost brick
		}
		
		const TIME_BUFFER: f32 = 0.025;
		let mut target_info = None;
		
		
		for brick in bricks_iter {
			if brick.time < time_running {
				continue;
			} 
			
			match &mut target_info {
				None => {
					target_info = Some( TargetInfo {
						time: brick.time,
						left_brick: brick.bounds.left_x,
						right_brick: brick.bounds.left_x
					});
				},
				Some(ti) => {
					if ti.time + TIME_BUFFER < brick.time {
						break; // >:< always chases the highest brick after time running
					}
					
					if brick.bounds.left_x < ti.left_brick {
						ti.left_brick = brick.bounds.left_x;
					} else if brick.bounds.left_x > ti.right_brick {
						ti.right_brick = brick.bounds.left_x;
					}
				}
			}
			
		}
		
		// >:< 
		if let Some(ti) = target_info {
			self.bounds.right_x = ti.left_brick;
			self.bounds.left_x = ti.left_brick - PLAYER_WIDTH as f32;
			self.face_dir = Direction::Right;
		}
		
		// log(&format!("{} => {} => {}", time_running, time_running / 0.01667, (time_running / 0.01667) % 256.0));
		self.graphic = Graphic { g: GraphicGroup::Running, frame: ((time_running / 0.01667) % 256.0) as u8, flags: 0 };
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
		let frame = 0;
		let flags = 0;
		let graphic = match brick_type {
			BrickType::Type1 => Graphic{ g: GraphicGroup::Brick, frame, flags },
			BrickType::Type2 => Graphic{ g: GraphicGroup::Brick2, frame, flags },
			BrickType::Type3 => Graphic{ g: GraphicGroup::Brick3, frame, flags }
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
