
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

struct UpcomingBricksData {
	time: f32,
	leftmost_brick_pos: u8,
	rightmost_brick_pos: u8,
	left_of_x: f32, // x position player has to be left of bricks (i.e. x pos of left brick - PLAYER_WIDTH)
	right_of_x: f32, // x position player has to be right of bricks (i.e. rightmost brick's right bound)
	group: bool
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
	face_dir: Direction,
	upcoming_bricks_data: VecDeque<UpcomingBricksData>
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
			graphic: Graphic { g: GraphicGroup::Player, sub_id: 0, flags: 0 },
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
			movement_frame_t: 0.0,
			upcoming_bricks_data: VecDeque::with_capacity(BRICK_DATA_BUFFER_SIZE)
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
		// INVARIANT: seconds_passed is never greater than MAX_TIME_BETWEEN_TICKS
		
		// movement is deterministic based on time of calculation, targets, and speed
			// means easier debuggability and more importantly no intra-movement variation as tick time varies
		// >:< 
			// How to handle calculation when the next bricks aren't showing yet on bricks_iter, but they will matter?
				// e.g. when there haven't been notes on the screen so the next ones are new, 
				// but the ones after haven't been accounted for yet

		// !!! 
			// take out as many branches as possible / group branches together
			// storing speed and direction (as well as dx) in self... may cause less branching
			// create tests for this function because it is too complex
			// lower max tick time is important, lower average tick time is not (consistent tick times are preferable)
				// how to profile branches and ticks to see what causes longest ticks?
			// ever want to back up before running?
			// ever want to run further past the bricks than necessary to gather speed?
			// Acceleration factor determined then saved? - changes curve parameters?
				// or just use a constant factor which will also reduce max speed
				// if used, store in MovementInterval struct
		
		// position given by: MAIN_COEFFICIENT/3 * (t + T_OFFSET)^3 + SECONDARY_COEFFICIENT * t + ZERO_T_POS_OFFSET
			// example: -8000/3 * (t - 0.25)^3 + 500t + c
			// the velocity is given by the derivative, up to MAX_SPEED_TIME at which the velocity is maxed out
		const MAIN_COEFFICIENT: f32 = -8000.0;
		const SECONDARY_COEFFICIENT: f32 = 500.0;
		const T_OFFSET: f32 = -0.25;
		const MAX_SPEED_TIME: f32 = 0.2; // time to reach max speed, effectively determining what the max speed is
		
		const ZERO_T_POS_OFFSET: f32 = -((MAIN_COEFFICIENT / 3.0) * T_OFFSET * T_OFFSET * T_OFFSET);
		const MAX_SPEED: f32 = MAIN_COEFFICIENT * (MAX_SPEED_TIME + T_OFFSET) * (MAX_SPEED_TIME + T_OFFSET) + SECONDARY_COEFFICIENT;
		const MAX_SPEED_POS: f32 = 
			(MAIN_COEFFICIENT / 3.0) * ((MAX_SPEED_TIME + T_OFFSET) * (MAX_SPEED_TIME + T_OFFSET) * (MAX_SPEED_TIME + T_OFFSET)) 
			+ (SECONDARY_COEFFICIENT * MAX_SPEED_TIME) + ZERO_T_POS_OFFSET;
		
		const RUN_GRAPHIC_THRESHOLD: f32 = 200.0; // threshold past which the player graphic will be running
		
		struct Target {
			target: f32,
			left_bound: f32,
			right_bound: f32,
			target_v: f32, // !!! Option? for when approaching a group?
			priority: u8
		}
		
		let solve_s_given_t = |time: f32| {
			if time > MAX_SPEED_TIME {
				return MAX_SPEED;
			} else if time >= 0.0{
				return -8000.0 * (time - 0.25) * (time - 0.25) + 500.0;
			} else {
				return 0.0;
			}
		};
		
		// if accelerating is false, returns p travelled if decelerating from max speed
		let solve_p_given_t = |time: f32, accelerating: bool| {
			
			if time < 0.0 { return 0.0; }
			if accelerating {
				let accel_time;
				let overtime;
				if time > MAX_SPEED_TIME {
					accel_time = MAX_SPEED_TIME;
					overtime = time - MAX_SPEED_TIME;
				} else {
					accel_time = time;
					overtime = 0.0;
				}
				
				let pos = (-8000.0 / 3.0) 
					* ((accel_time - 0.25) * (accel_time - 0.25) * (accel_time - 0.25)) 
					+ (500.0 * accel_time) + ZERO_T_POS_OFFSET;
				let overpos = MAX_SPEED * overtime;
				
				return pos + overpos;
			} else {
				if time >= MAX_SPEED_TIME {
					return MAX_SPEED_POS;
				} else {
					let remaining_decel_time = MAX_SPEED_TIME - time;
					let remaining_pos = (-8000.0 / 3.0) 
						* ((remaining_decel_time - 0.25) * (remaining_decel_time - 0.25) * (remaining_decel_time - 0.25))
						+ (500.0 * remaining_decel_time) + ZERO_T_POS_OFFSET;
					
					return MAX_SPEED_POS - remaining_pos;
				}
			}
		};
		
		// if accelerating is false, returns time to decelerate to s from MAX_SPEED
		let solve_t_given_s = |speed: f32, accelerating: bool| {
			
			if speed >= MAX_SPEED { 
				if accelerating { return MAX_SPEED_TIME; }
				else { return 0.0; } 
			}
			if speed <= 0.0 { 
				if accelerating { return 0.0; }
				else { return MAX_SPEED_TIME; } 
			} 
			
			let (base, multiplier) = 
				if accelerating { (0.0, 1.0) } 
				else { (MAX_SPEED_TIME, -1.0) };
			
			// !!! binary search through possible times. For efficiency change into a lookup table
			let mut low = 0.0;
			let mut high = MAX_SPEED_TIME;
			let mut mid = MAX_SPEED_TIME / 2.0;
			loop {
				let mid_speed = solve_s_given_t(mid);
				if mid_speed - speed > 1.0 {
					high = mid;
					mid = (high + low) / 2.0;
				} else if speed - mid_speed > 1.0 {
					low = mid;
					mid = (high + low) / 2.0;
				} else {
					return base + multiplier * mid;
				}
			}
		};
		
		// returns 0 if p <= 0, logic error if trying to get the time to travel a negative pos (same as time to travel a positive pos)
		let solve_t_given_p = |pos: f32| {
			if pos <= 0.0 { return 0.0; } 
			if pos > MAX_SPEED_POS { return MAX_SPEED_TIME + (pos - MAX_SPEED_POS) / MAX_SPEED; }
			
			// !!! binary search through possible times. For efficiency change into a lookup table
			let mut low = 0.0;
			let mut high = MAX_SPEED_TIME;
			let mut mid = MAX_SPEED_TIME / 2.0;
			loop {
				let mid_pos = solve_p_given_t(mid, true);
				if mid_pos - pos > 1.0 {
					high = mid;
					mid = (high + low) / 2.0;
				} else if pos - mid_pos > 1.0 {
					low = mid;
					mid = (high + low) / 2.0;
				} else {
					return mid;
				}
			}
			
		};
		
		// get data on upcoming bricks, update self.upcoming_bricks_data
		// !!! should not be calculated at runtime, should all be stored ahead of time
		// TODO may be able to store more information/calculations in between ticks, but may cause varying tick calculation times
		// this is the time threshold before which the player will go after a note
			// !!! smarter buffer determination + dash/slash (hit_dir) that goes after a brick if it's still hittable
		let time_with_buffer = time_running - 0.06; 
		if let Some(bricks_data) = self.upcoming_bricks_data.get(0) {
			if bricks_data.time < time_with_buffer {
				self.upcoming_bricks_data.pop_front();
			}
		}
		if self.upcoming_bricks_data.len() < BRICK_DATA_BUFFER_SIZE {
			let min_time = match self.upcoming_bricks_data.back() {
				Some(bricks_data) => bricks_data.time,
				None => time_with_buffer
			};
			while let Some(brick) = bricks_iter.next() {
				
				// if the brick comes after the last bricks with stored data, record them
				if brick.time() > min_time {
					let mut bricks_time = brick.time();
					let mut leftmost_brick_pos = note_pos_from_x(brick.bounds().left_x);
					let mut rightmost_brick_pos = leftmost_brick_pos;
					let mut left_of_x = brick.bounds().left_x - PLAYER_WIDTH as f32;
					let mut right_of_x = brick.bounds().right_x;
					let mut group = false;
					
					while let Some(brick) = bricks_iter.next() {
						if brick.time() - bricks_time > F32_ZERO {
							self.upcoming_bricks_data.push_back( UpcomingBricksData {
								time: bricks_time,
								leftmost_brick_pos,
								rightmost_brick_pos,
								left_of_x,
								right_of_x,
								group
							});
							
							if self.upcoming_bricks_data.len() < BRICK_DATA_BUFFER_SIZE {
								bricks_time = brick.time();
								leftmost_brick_pos = note_pos_from_x(brick.bounds().left_x);
								rightmost_brick_pos = leftmost_brick_pos;
								left_of_x = brick.bounds().left_x;
								right_of_x = brick.bounds().right_x;
								group = false;
								continue;
							}
							break;
						}
						else {
							let left_of = brick.bounds().left_x - PLAYER_WIDTH as f32;
							let right_of = brick.bounds().right_x;
							if left_of < left_of_x {
								left_of_x = left_of;
								leftmost_brick_pos = note_pos_from_x(brick.bounds().left_x);
							}
							else if right_of > right_of_x { 
								right_of_x = right_of;
								rightmost_brick_pos = note_pos_from_x(brick.bounds().left_x);
							}
						}
						
						// bug prevention measure each group of bricks has the time of the last brick added
						bricks_time = brick.time(); 
						group = true;
					}
						
					self.upcoming_bricks_data.push_back(UpcomingBricksData{
						time: bricks_time,
						leftmost_brick_pos,
						rightmost_brick_pos,
						left_of_x: left_of_x,
						right_of_x: right_of_x,
						group
					});
					break;
				}
			}
		}
		
		// get both targets and use them to update self.movement
		if let None = self.movement { match self.upcoming_bricks_data.get(0) {
			None => {}, // no movement
			Some(bricks_data) => {
				
				const SEPARATION_SPACE: f32 = (BRICK_WIDTH - PLAYER_WIDTH) as f32 / 2.0; // so player is evenly between bricks
				const MIN_SEPARATION_SPACE: f32 = SEPARATION_SPACE / 2.0;
				const HIGH_SEPARATION_SPACE: f32 = SLASH_WIDTH as f32 / 2.0;
				const MAX_SEPARATION_SPACE: f32 = SLASH_WIDTH as f32 / 1.5;
				
				// get target approach types, positions, and speed based on following brick data
					// HIGH_SEPARATION_SPACE used when slashing a single brick in different direction of next bricks
					// ever a time to go HIGH_SEPARATION_SPACE away on a group:
						// can't make it to other side on time and have to turn after dashing?
					// mistiming when approaching at a run may cause player to be slightly within brick at slash time
				// calculating information for both targets rather than just the likely one is better 
					// because there will be instances where both must be calculated, and consistent tick time is preferable
				let left_target;
				let right_target;
				match self.upcoming_bricks_data.get(1) {
					None => {
						left_target = Target {
							target: bricks_data.left_of_x - SEPARATION_SPACE,
							left_bound: bricks_data.left_of_x - MAX_SEPARATION_SPACE,
							right_bound: bricks_data.left_of_x - MIN_SEPARATION_SPACE,
							target_v: 0.0,
							priority: 1
						};
						right_target = Target {
							target: bricks_data.right_of_x + SEPARATION_SPACE,
							left_bound: bricks_data.right_of_x + MIN_SEPARATION_SPACE,
							right_bound: bricks_data.right_of_x + MAX_SEPARATION_SPACE,
							target_v: 0.0,
							priority: 1
						};
					},
					Some(bricks_data2) => {
						let left_pos = bricks_data.leftmost_brick_pos - 1;
						let right_pos = bricks_data.rightmost_brick_pos + 1;
						let left_pos2 = bricks_data2.leftmost_brick_pos - 1;
						let right_pos2 = bricks_data2.rightmost_brick_pos + 1;
						let group = bricks_data.group; // determines whether the bricks will be dashed through
						
						// !!! 
							// possible 3rd note lookahead if direction between first and second bricks is ambiguous but important
							// check if it's a recognized brick group structure (store in brick data?)
								// if left_target + 2 == right_target - 2
							// don't necessarily need to be stopping when approaching a group
						match group {
							false => {
								if left_pos - right_pos2 > 0 {
									left_target = Target {
										target: bricks_data.left_of_x - HIGH_SEPARATION_SPACE,
										left_bound: bricks_data.left_of_x - MAX_SEPARATION_SPACE,
										right_bound: bricks_data.left_of_x - MIN_SEPARATION_SPACE,
										target_v: 0.0,
										priority: 2
									};
									right_target = Target {
										target: bricks_data.right_of_x + SEPARATION_SPACE,
										left_bound: bricks_data.right_of_x + MIN_SEPARATION_SPACE,
										right_bound: bricks_data.right_of_x + MAX_SEPARATION_SPACE,
										target_v: -(bricks_data.right_of_x - bricks_data2.right_of_x)
											/ (bricks_data2.time - bricks_data.time),
										priority: 1
									};
								} else if left_pos2 - right_pos > 0 {
									left_target = Target {
										target: bricks_data.left_of_x - SEPARATION_SPACE,
										left_bound: bricks_data.left_of_x - MAX_SEPARATION_SPACE,
										right_bound: bricks_data.left_of_x - MIN_SEPARATION_SPACE,
										target_v: (bricks_data2.left_of_x - bricks_data.left_of_x)
											/ (bricks_data2.time - bricks_data.time),
										priority: 1
									};
									right_target = Target {
										target: bricks_data.right_of_x + HIGH_SEPARATION_SPACE,
										left_bound: bricks_data.right_of_x + MIN_SEPARATION_SPACE,
										right_bound: bricks_data.right_of_x + MAX_SEPARATION_SPACE,
										target_v: 0.0,
										priority: 2
									};
								} else {
									// !!! ambiguous (or following pos may be the same as one of the target pos)
									left_target = Target {
										target: bricks_data.left_of_x - SEPARATION_SPACE,
										left_bound: bricks_data.left_of_x - MAX_SEPARATION_SPACE,
										right_bound: bricks_data.left_of_x - MIN_SEPARATION_SPACE,
										target_v: 0.0,
										priority: 1
									};
									right_target = Target {
										target: bricks_data.right_of_x + SEPARATION_SPACE,
										left_bound: bricks_data.right_of_x + MIN_SEPARATION_SPACE,
										right_bound: bricks_data.right_of_x + MAX_SEPARATION_SPACE,
										target_v: 0.0,
										priority: 1
									};
								}
							},
							true => {
								let end_dash_pos = left_pos + 2; // assume a dash
								let left_priority;
								let right_priority;
								
								// highly preferable to dash from opposite side of following bricks
								if left_pos2 >= end_dash_pos {
									left_priority = 3;
									right_priority = 1;
								} else if end_dash_pos >= right_pos2 {
									left_priority = 1;
									right_priority = 3;
								} else {
									// !!! ambiguous
									left_priority = 1;
									right_priority = 1;
								}
								
								// !!! when approaching a group, the bound for how close you must be is tighter
								left_target = Target {
									target: bricks_data.left_of_x - SEPARATION_SPACE,
									left_bound: bricks_data.left_of_x - SEPARATION_SPACE,
									right_bound: bricks_data.left_of_x - MIN_SEPARATION_SPACE,
									target_v: 0.0,
									priority: left_priority
								};
								right_target = Target {
									target: bricks_data.right_of_x + SEPARATION_SPACE,
									left_bound: bricks_data.right_of_x + MIN_SEPARATION_SPACE,
									right_bound: bricks_data.right_of_x + SEPARATION_SPACE,
									target_v: 0.0,
									priority: right_priority
								};
							}
						}
					}
				}
					
				// Choose between the left and the right target
				// !!! if it's possible to offload some work from calculation ticks to other ticks, that is preferable
				// >:< include accelerated intervals, decelerated/stopping intervals
					// ever boost/insta-stop for target speed? Or boost always after slash for next movement.
					// shifting stops / tiny movements, avoid by being within bounds and otherwise going for target?
					// given a desired speed, how to calculate how to reach bricks at that speed?
					
				let left_target = 
					if left_target.target_v <= MAX_SPEED { // !!! invariant, should be positive or 0
						left_target 
					} else {
						Target {
							target_v: MAX_SPEED,
							.. left_target
						}
					};
				let right_target = 
					if right_target.target_v >= -MAX_SPEED { // !!! invariant, should be negative or 0
						right_target 
					} else {
						Target {
							target_v: -MAX_SPEED,
							.. right_target
						}
					};
				let start_v = self.dx;
				let start_x = self.bounds.left_x;
				let start_t = time_running;
				let brick_t = bricks_data.time;
				
				let time_left;
				let time_right;
				let mut movement_left = Movement {
					intervals: Vec::new(),
					current_interval: 0
				};
				let mut movement_right = Movement {
					intervals: Vec::new(),
					current_interval: 0
				};
				
				let abs_start_v = if start_v >= 0.0 { start_v } else { -start_v };
				// if both targets are right of player, else if both targets are left, else in between targets
				if left_target.right_bound > start_x {
					
					// if heading right toward targets else heading left away
					if start_v >= 0.0 {
						let implicit_accel_time = solve_t_given_s(abs_start_v, true);
						let effective_start_t = start_t - implicit_accel_time;
						let effective_start_x = start_x - solve_p_given_t(implicit_accel_time, true);
						
						let interval0_left = MovementInterval {
							movement_type: MovementType::Accelerating,
							direction: Direction::Right,
							effective_start_t,
							effective_start_x,
							end_t: solve_t_given_p(left_target.target - effective_start_x) + effective_start_t
						};
						
						let interval0_right = MovementInterval {
							movement_type: MovementType::Accelerating,
							direction: Direction::Right,
							effective_start_t,
							effective_start_x,
							end_t: solve_t_given_p(right_target.target - effective_start_x) + effective_start_t
						};
						
						time_left = solve_t_given_p(left_target.left_bound - effective_start_x) + effective_start_t;
						time_right = solve_t_given_p(right_target.left_bound - effective_start_x) + effective_start_t;
						movement_left.intervals.push(interval0_left);
						movement_right.intervals.push(interval0_right);
					} else {
						let implicit_decel_time = solve_t_given_s(abs_start_v, false);
						let end_t0 = start_t + solve_t_given_s(abs_start_v, true);
						let interval0 = MovementInterval {
							movement_type: MovementType::Decelerating,
							direction: Direction::Left,
							effective_start_t: start_t - implicit_decel_time,
							effective_start_x: start_x + solve_p_given_t(implicit_decel_time, false),
							end_t: end_t0
						};
						
						let turn_pos = interval0.effective_start_x - MAX_SPEED_POS;
						let interval1_left = MovementInterval {
							movement_type: MovementType::Accelerating,
							direction: Direction::Right,
							effective_start_t: end_t0,
							effective_start_x: turn_pos,
							end_t: solve_t_given_p(left_target.target - turn_pos) + end_t0
						};
						
						let interval1_right = MovementInterval {
							movement_type: MovementType::Accelerating,
							direction: Direction::Right,
							effective_start_t: end_t0,
							effective_start_x: turn_pos,
							end_t: solve_t_given_p(right_target.target - turn_pos) + end_t0
						};
						
						time_left = solve_t_given_p(right_target.left_bound - turn_pos) + end_t0;
						time_right = solve_t_given_p(right_target.left_bound - turn_pos) + end_t0;
						movement_left.intervals.push(interval0.clone());
						movement_left.intervals.push(interval1_left);
						movement_right.intervals.push(interval0);
						movement_right.intervals.push(interval1_right);
					}
				} else if right_target.left_bound < start_x {
					
					// if heading left toward targets else heading right away
					if start_v <= 0.0 {
						let implicit_accel_time = solve_t_given_s(abs_start_v, true);
						let effective_start_t = start_t - implicit_accel_time;
						let effective_start_x = start_x + solve_p_given_t(implicit_accel_time, true);
						
						let interval0_left = MovementInterval {
							movement_type: MovementType::Accelerating,
							direction: Direction::Left,
							effective_start_t,
							effective_start_x,
							end_t: solve_t_given_p(effective_start_x - left_target.target) + effective_start_t
						};
						
						let interval0_right = MovementInterval {
							movement_type: MovementType::Accelerating,
							direction: Direction::Left,
							effective_start_t,
							effective_start_x,
							end_t: solve_t_given_p(effective_start_x - right_target.target) + effective_start_t
						};
						
						time_left = solve_t_given_p(effective_start_x - left_target.right_bound) + effective_start_t;
						time_right = solve_t_given_p(effective_start_x - right_target.right_bound) + effective_start_t;
						movement_left.intervals.push(interval0_left);
						movement_right.intervals.push(interval0_right);
					} else {
						let implicit_decel_time = solve_t_given_s(abs_start_v, false);
						let end_t0 = start_t + solve_t_given_s(abs_start_v, true);
						let interval0 = MovementInterval {
							movement_type: MovementType::Decelerating,
							direction: Direction::Right,
							effective_start_t: start_t - implicit_decel_time,
							effective_start_x: start_x - solve_p_given_t(implicit_decel_time, false),
							end_t: end_t0
						};
						
						let turn_pos = interval0.effective_start_x + MAX_SPEED_POS;
						let interval1_left = MovementInterval {
							movement_type: MovementType::Accelerating,
							direction: Direction::Left,
							effective_start_t: end_t0,
							effective_start_x: turn_pos,
							end_t: solve_t_given_p(turn_pos - left_target.target) + end_t0
						};
						
						let interval1_right = MovementInterval {
							movement_type: MovementType::Accelerating,
							direction: Direction::Left,
							effective_start_t: end_t0,
							effective_start_x: turn_pos,
							end_t: solve_t_given_p(turn_pos - right_target.target) + end_t0
						};
						
						time_left = solve_t_given_p(turn_pos - right_target.right_bound) + end_t0;
						time_right = solve_t_given_p(turn_pos - right_target.right_bound) + end_t0;
						movement_left.intervals.push(interval0.clone());
						movement_left.intervals.push(interval1_left);
						movement_right.intervals.push(interval0);
						movement_right.intervals.push(interval1_right);
					}
				} else {
					// if heading right towards right target, else heading left towards left target
					
					if start_v >= 0.0 {
					
						let implicit_accel_time = solve_t_given_s(abs_start_v, true);
						let implicit_decel_time = solve_t_given_s(abs_start_v, false);
						
						let effective_start_x_accel = start_x - solve_p_given_t(implicit_accel_time, true);
						let effective_start_x_decel = start_x - solve_p_given_t(implicit_decel_time, false);
						
						let effective_start_t_right = start_t - implicit_accel_time;
						let interval0_right = MovementInterval {
							movement_type: MovementType::Accelerating,
							direction: Direction::Right,
							effective_start_t: effective_start_t_right,
							effective_start_x: effective_start_x_accel,
							end_t: solve_t_given_p(right_target.target - effective_start_x_accel) + effective_start_t_right
						};
						
						let end_t0_left = start_t + implicit_accel_time;
						let interval0_left = MovementInterval {
							movement_type: MovementType::Decelerating,
							direction: Direction::Right,
							effective_start_t: start_t - implicit_decel_time,
							effective_start_x: effective_start_x_decel,
							end_t: end_t0_left
						};
						
						let turn_pos = interval0_left.effective_start_x + MAX_SPEED_POS;
						let interval1_left = MovementInterval {
							movement_type: MovementType::Accelerating,
							direction: Direction::Left,
							effective_start_t: end_t0_left,
							effective_start_x: turn_pos,
							end_t: solve_t_given_p(turn_pos - left_target.target) + end_t0_left
						};
						
						time_left = solve_t_given_p(turn_pos - left_target.right_bound) + end_t0_left;
						time_right = solve_t_given_p(right_target.left_bound - effective_start_x_accel) + effective_start_t_right;
						movement_left.intervals.push(interval0_left);
						movement_left.intervals.push(interval1_left);
						movement_right.intervals.push(interval0_right);
					} else {
						
						let implicit_accel_time = solve_t_given_s(abs_start_v, true);
						let implicit_decel_time = solve_t_given_s(abs_start_v, false);
						
						let effective_start_x_accel = start_x + solve_p_given_t(implicit_accel_time, true);
						let effective_start_x_decel = start_x + solve_p_given_t(implicit_decel_time, false);
						
						let effective_start_t_left = start_t - implicit_accel_time;
						let interval0_left = MovementInterval {
							movement_type: MovementType::Accelerating,
							direction: Direction::Left,
							effective_start_t: effective_start_t_left,
							effective_start_x: effective_start_x_accel,
							end_t: solve_t_given_p(effective_start_x_accel - left_target.target) + effective_start_t_left
						};
						
						let end_t0_right = start_t + implicit_accel_time;
						let interval0_right = MovementInterval {
							movement_type: MovementType::Decelerating,
							direction: Direction::Left,
							effective_start_t: start_t - implicit_decel_time,
							effective_start_x: effective_start_x_decel,
							end_t: end_t0_right
						};
						
						let turn_pos = interval0_right.effective_start_x - MAX_SPEED_POS;
						let interval1_right = MovementInterval {
							movement_type: MovementType::Accelerating,
							direction: Direction::Right,
							effective_start_t: end_t0_right,
							effective_start_x: turn_pos,
							end_t: solve_t_given_p(right_target.target - turn_pos) + end_t0_right
						};
						
						time_left = solve_t_given_p(effective_start_x_accel - left_target.right_bound) + effective_start_t_left;
						time_right = solve_t_given_p(right_target.left_bound - turn_pos) + end_t0_right;
						movement_left.intervals.push(interval0_left);
						movement_right.intervals.push(interval0_right);
						movement_right.intervals.push(interval1_right);
					}
				}
				
				// >:< 
				// log information on targets and choice
				{
					log(&format!("Left Target: {} - {} - {}, target velocity: {}, priority: {}\n", 
						left_target.left_bound, left_target.target, left_target.right_bound, 
						left_target.target_v, left_target.priority));
					log(&format!("Right Target: {} - {} - {}, target velocity: {}, priority: {}\n", 
						right_target.left_bound, right_target.target, right_target.right_bound, 
						right_target.target_v, right_target.priority));
					log(&format!("Player: left x: {}, right x: {}", self.bounds.left_x, self.bounds.right_x));
					
					let mut interval_id = 0;
					log(&format!("LEFT MOVEMENT:\n{}", movement_left));
					while interval_id < movement_left.intervals.len() {	
						let interval = &movement_left.intervals[interval_id];
						let t = interval.end_t - interval.effective_start_t;
						let change_multiplier = match interval.direction {
							Direction::Left => -1.0,
							Direction::Right => 1.0
						};
						let (change_x, end_speed) = match interval.movement_type {
							MovementType::Constant(s) => (s * t, s),
							MovementType::Accelerating => (solve_p_given_t(t, true), solve_s_given_t(t)),
							MovementType::Decelerating => (solve_p_given_t(t, false), solve_s_given_t(MAX_SPEED_TIME - t))
						};
						
						log(&format!("projected end x: {}\n projected end speed: {}\n\n", 
							change_x * change_multiplier + interval.effective_start_x, end_speed * change_multiplier));
						
						interval_id += 1;
					}
					log(&format!("\n\n"));
					
					log(&format!("RIGHT MOVEMENT\n{}", movement_right));
					interval_id = 0;
					while interval_id < movement_right.intervals.len() {	
						let interval = &movement_right.intervals[interval_id];
						let t = interval.end_t - interval.effective_start_t;
						let change_multiplier = match interval.direction {
							Direction::Left => -1.0,
							Direction::Right => 1.0
						};
						let (change_x, end_speed) = match interval.movement_type {
							MovementType::Constant(s) => (s * t, s),
							MovementType::Accelerating => (solve_p_given_t(t, true), solve_s_given_t(t)),
							MovementType::Decelerating => (solve_p_given_t(t, false), solve_s_given_t(MAX_SPEED_TIME - t))
						};
						
						log(&format!("projected end x: {}\n projected end speed: {}\n\n", 
							change_x * change_multiplier + interval.effective_start_x, end_speed * change_multiplier));
							
						interval_id += 1;
					}
					log(&format!("\n\n"));
				}
					
				// >:< Priority enum? LeftMuchHigher LeftHigher Same RightHigher RightMuchHigher
				let priority_difference = left_target.priority as i8 - right_target.priority as i8;
				let left_higher = priority_difference > 0;
				let right_higher = priority_difference < 0;
				// let left_much_higher = priority_difference > 1;
				// let right_much_higher = priority_difference < -1;
				let in_time_left = time_left <= brick_t;
				let in_time_right = time_right <= brick_t;
				match (in_time_left, in_time_right) {
					(true, false) => {
						self.movement = Some(movement_left);
					},
					(false, true) => {
						self.movement = Some(movement_right);
					},
					_ => {
						self.movement = 
							if left_higher {
								Some(movement_left)
							} else if right_higher {
								Some(movement_right)
							} else if time_left < time_right {
								Some(movement_left)
							} else {
								Some(movement_right)
							};
					}
				}
			}
		} }
		
		// move player
		if let Some(movement) = &mut self.movement {
			
			loop {
				if movement.current_interval < movement.intervals.len() {
					let this_interval = &movement.intervals[movement.current_interval];
					if this_interval.end_t >= time_running {
						
						let time_difference = time_running - this_interval.effective_start_t;
						let (pos_difference, speed) = match (this_interval.movement_type) {
							MovementType::Constant(s) => (s * time_difference, s),
							MovementType::Accelerating => (solve_p_given_t(time_difference, true), solve_s_given_t(time_difference)),
							MovementType::Decelerating => 
								(MAX_SPEED_POS - solve_p_given_t(MAX_SPEED_TIME - time_difference, true), 
								solve_s_given_t(MAX_SPEED_TIME - time_difference))
						};
						let difference_multiplier = match this_interval.direction {
							Direction::Left => -1.0,
							Direction::Right => 1.0
						};
						
						self.dx = speed * difference_multiplier;
						self.bounds.left_x = this_interval.effective_start_x + pos_difference * difference_multiplier;
						self.bounds.right_x = self.bounds.left_x + PLAYER_WIDTH as f32;
						self.face_dir = this_interval.direction;
						
						break;
					} else {
						movement.current_interval += 1;
						
						continue;
					}
				}
				
				if movement.intervals.len() == 0 {
					self.movement = None;
					break;
				}
				
				// the last movement has ended end the final movement before decelerating to 0
				// >:< always decelerating to 0? What if continuing as soon as acquiring next target?
					// mix logic? of moving / ending movements and target acquisition?
				let last_movement = &movement.intervals[movement.intervals.len() - 1];
				let last_direction = last_movement.direction;
				let last_time = last_movement.end_t - last_movement.effective_start_t;
				
				let (start_x, start_v) = match (&last_movement.movement_type, last_direction) {
					(MovementType::Constant(s), Direction::Right) => (last_movement.effective_start_x + s * last_time, *s),
					(MovementType::Constant(s), Direction::Left) => (last_movement.effective_start_x - s * last_time, -*s),
					(MovementType::Accelerating, Direction::Left) => 
						(last_movement.effective_start_x - solve_p_given_t(last_time, true), solve_s_given_t(last_time)),
					(MovementType::Accelerating, Direction::Right) => 
						(last_movement.effective_start_x + solve_p_given_t(last_time, true), solve_s_given_t(last_time)),
					(MovementType::Decelerating, Direction::Left) => 
						(last_movement.effective_start_x - solve_p_given_t(last_time, false), 
							solve_s_given_t(MAX_SPEED_TIME - last_time)),
					(MovementType::Decelerating, Direction::Right) => 
						(last_movement.effective_start_x + solve_p_given_t(last_time, false), 
							solve_s_given_t(MAX_SPEED_TIME - last_time))
				};
				
				self.dx = start_v;
				self.bounds.left_x = start_x;
				self.bounds.right_x = self.bounds.left_x + PLAYER_WIDTH as f32;
				self.face_dir = last_direction;
				
				if start_v < 1.0 && start_v > -1.0 {
					self.movement = None;
					break;
				} 
				
				let (implicit_decel_time, effective_start_x) = match last_direction {
					Direction::Left => {
						let t = solve_t_given_s(-start_v, false);
						(t, start_x + solve_p_given_t(t, false))
					},
					Direction::Right => {
						let t = solve_t_given_s(start_v, false);
						(t, start_x - solve_p_given_t(t, false))
					}
				};
				
				*movement = Movement {
					intervals: Vec::new(),
					current_interval: 0
				};
				
				movement.intervals.push( MovementInterval {
					movement_type: MovementType::Decelerating,
					direction: last_direction,
					effective_start_t: time_running - implicit_decel_time,
					effective_start_x,
					end_t: time_running + MAX_SPEED_TIME - implicit_decel_time
				} );
				
				continue;
			}
		}
		
		// check to not go past any boundaries
		if self.bounds.left_x < LEFT_BOUNDARY {
			self.bounds.right_x -= self.bounds.left_x - LEFT_BOUNDARY;
			self.bounds.left_x -= self.bounds.left_x - LEFT_BOUNDARY;
			self.dx = 0.0;
			self.face_dir = Direction::Right;
			self.hit_dir = Direction::Right;
		} else if self.bounds.right_x > RIGHT_BOUNDARY {
			self.bounds.left_x -= self.bounds.right_x - RIGHT_BOUNDARY;
			self.bounds.right_x -= self.bounds.right_x - RIGHT_BOUNDARY;
			self.dx = 0.0;
			self.face_dir = Direction::Left;
			self.hit_dir = Direction::Left;
		}
		
		// update the graphic
		// >:< how to do a turning graphic?
		if self.dx > F32_ZERO || self.dx < -F32_ZERO {
			self.movement_frame_t += seconds_passed; 
			// >:< common shortest frame time (16.7? or less to avoid accidentally long frames?) variable for graphics
				// have all images stored with assumption of the fps
			while self.movement_frame_t > 0.0167 { 
				self.movement_frame_t -= 0.0167;
				self.movement_frame = (self.movement_frame + 1) % NUM_MOVEMENT_FRAMES;
			}
		} else {
			self.movement_frame = 0;
			self.movement_frame_t = 0.0;
		}
		
		let flags = match self.face_dir {
			Direction::Right => 0,
			Direction::Left => GraphicFlags::HorizontalFlip as u8
		};
		
		// !!! more robust way of indexing graphics?
		let sub_id = if self.dx < RUN_GRAPHIC_THRESHOLD && self.dx > -RUN_GRAPHIC_THRESHOLD { 
			self.movement_frame } else { 
			self.movement_frame + NUM_MOVEMENT_FRAMES };
			
		self.graphic = Graphic { g: GraphicGroup::Player, sub_id, flags };
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
		let sub_id = 0;
		let flags = 0;
		let graphic = match brick_type {
			BrickType::Type1 => Graphic{ g: GraphicGroup::Brick, sub_id, flags },
			BrickType::Type2 => Graphic{ g: GraphicGroup::Brick2, sub_id, flags },
			BrickType::Type3 => Graphic{ g: GraphicGroup::Brick3, sub_id, flags }
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
		let sub_id = 0;
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
			graphic: Graphic{ g: graphic_group, sub_id, flags },
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
		let sub_id = 0;
		let graphic = Graphic{ g: GraphicGroup::Dash0, sub_id, flags };
		
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
