
extern crate console_error_panic_hook; // !!! set when a new game is created. Move to its own initialization

use std::collections::btree_set::BTreeSet; 
use std::collections::VecDeque;
use std::cmp::Ordering;

use crate::objects;
use crate::player;

use crate::player::Player;
use crate::brick::Brick;
use crate::BrickData;
use crate::GameData;
use crate::Input;
use crate::GraphicGroup;
use crate::Graphic;
use crate::PositionedGraphic;
use crate::RenderingInstructions;
use objects::Object;
use objects::BrickType;
use objects::ObjectBounds;
use objects::Direction;
use objects::PLAYER_WIDTH;
use objects::PLAYER_HEIGHT;

use wasm_bindgen::prelude::*;
use js_sys::Array;

use crate::GAME_HEIGHT;
use crate::GAME_WIDTH;
use crate::GROUND_POS;
use objects::BRICK_HEIGHT;
use objects::BRICK_SEGMENT_HEIGHT;
use objects::BRICK_WIDTH;
use objects::HOLD_HITBOX_WIDTH;
use player::RUN_SPEED;
use player::MAX_BOOST_DISTANCE;

const MAX_TIME_BETWEEN_TICKS: f32 = 0.025;
const BRICK_SCORE: i32 = 100;
const HOLD_SEGMENT_SCORE: i32 = 20;

#[derive(Clone, Copy)]
struct UpcomingBrick {
	brick_type: BrickType,
	x: f32,
	// the y value at which the note should appear. At time = 0 the top of the screen is y = 0
		// and a note that should be hit at time = 0 has appearance_y of TIME_ZERO_BRICK_POS
		// notes off the bottom of the screen have appearance_y's corresponding to how much has to be scrolled before they show up
	appearance_y: f32, // y value (including amount needed to scroll) of the brick
	end_y: f32, // y value (including amount needed to scroll) of the bottom of the brick and its hold segments
	hold_segments: u8
}

#[derive(Clone)]
pub struct TargetInfo {
	pub appearance_y: f32,
	pub end_y: f32,
	pub dest_x: f32, // where the player left_x should be
	pub post_hit_x: f32,
	pub is_hold_note: bool,
	pub hit_dir: Direction, // direction player must slash in order to hit targets once he arrives at dest_x
	pub boost_to_target: bool, // whether the player should boost in order to reach dest_x on time
	pub dash_to_target: bool, // whether the player should dash in order to reach dest_x on time
	pub hittable_time: f32,
	pub passed_time: f32
}

#[wasm_bindgen]
pub struct Game {
	player: Player,
	// !!! better data structures than VecDeques. Indexable BTrees
	bricks: VecDeque<UpcomingBrick>, // all bricks of the song, ordered
	targets: VecDeque<TargetInfo>, // all targets of the song, ordered
	upcoming_brick_idx: usize,
	target_idx: usize,
	// uses a vec instead of a btree because std lib btreeset is unindexable
	current_bricks: VecDeque<Brick>, // current bricks that are on screen or about to appear on screen, ordered
	scrolled_y: f32,
	end_y: f32,
	game_data: GameData, 
	notes: BTreeSet<BrickData>, // all notes of the song before conversion into bricks
	graphics: Vec<PositionedGraphic>,
	bricks_broken: u8
}

#[wasm_bindgen]
impl Game {
	pub fn new(bpm: f32, brick_speed: f32, duration: f32) -> Game {
		console_error_panic_hook::set_once();
		
		return Game {
			player: Player::new((BRICK_WIDTH * 2) as f32 - objects::PLAYER_WIDTH as f32 / 2.0),
			bricks: VecDeque::new(), // bricks on screen, ordered by time they are meant to be played
			targets: VecDeque::new(),
			upcoming_brick_idx: 0,
			target_idx: 0,
			current_bricks: VecDeque::new(),
			scrolled_y: 0.0,
			end_y: Game::end_y(0.0, brick_speed),
			game_data: GameData {
				bpm,
				beat_interval: 60.0 / bpm as f32,
				brick_speed,
				time_running: 0.0,
				score: 0,
				max_score: 0,
				duration,
			},
			notes: BTreeSet::new(),
			graphics: Vec::with_capacity(512), // TODO what should the upper limit be? Make it a hard limit
			bricks_broken: 0
		};
	}
			
	// tick the game state by the given amount of time
	pub fn tick(&mut self, mut seconds_passed: f32) {
		
		// prevent disproportionally long ticks
		if seconds_passed > MAX_TIME_BETWEEN_TICKS {
			self.tick(seconds_passed - MAX_TIME_BETWEEN_TICKS);
			seconds_passed = MAX_TIME_BETWEEN_TICKS;
		}
		
		let delta_y = seconds_passed * self.game_data.brick_speed;
		self.game_data.time_running += seconds_passed;
		self.scrolled_y += delta_y;
		self.end_y += delta_y;

		let hold_positions = self.find_hold_positions();

		// tick player
			// if target is on screen and target has not already passed, pass target info to player
		if let Some(target) = self.targets.get(self.target_idx){
			if target.appearance_y < self.end_y && target.passed_time >= self.game_data.time_running {
				self.player.tick(seconds_passed, &self.game_data, Some(target.clone()), hold_positions);
			} else {
				self.player.tick(seconds_passed, &self.game_data, None, hold_positions);
			}
		} else {
			self.player.tick(seconds_passed, &self.game_data, None, hold_positions);
		}

		// check for brick destruction (first of two times, to account for hold notes which may sneak past hitbox)
		self.destroy_bricks();

		// tick all current bricks
		for brick in &mut self.current_bricks {
			brick.bounds.top_y -= delta_y;
			brick.bounds.bottom_y -= delta_y;
		}

		// discard any bricks that are offscreen
		loop {
			if self.current_bricks.len() > 0 && self.current_bricks[0].end_y < self.scrolled_y {
				self.current_bricks.pop_front();
				continue;
			} else {
				break;
			}
		}
		
		// check for brick destruction (second of two times)
		self.destroy_bricks();
		
		self.add_to_current_bricks();

		// if there are no bricks remaining in the target, set the target to the next target
		if let Some(highest_brick) = self.current_bricks.get(0) {
			loop {
				if self.target_idx >= self.targets.len() {
					break;
				}

				if highest_brick.appearance_y <= self.targets[self.target_idx].appearance_y {
					break;
				}

				self.target_idx += 1;
				
				continue;
			}
		} else {
			// set the target to the first target beyond the games end y
			loop {
				if self.target_idx >= self.targets.len() {
					break;
				} else if self.targets[self.target_idx].appearance_y >= self.end_y {
					break;
				} else {
					self.target_idx += 1;
				}
			}
		}
	}
	
	// updates the displayed graphics and returns rendering instructions in the form of a pointer
	pub fn rendering_instructions(&mut self) -> RenderingInstructions {
		let graphics = &mut self.graphics;
		
		graphics.clear();
		
		graphics.push(
			PositionedGraphic::new(Graphic{ g: GraphicGroup::Background, frame: 0, flags: 0, arg: 0}, 0.0, 0.0)
		);
		
		graphics.append(&mut self.player.rendering_instructions(self.game_data.time_running));
		
		let minutes_per_beat = 1.0 / self.game_data.bpm;
		let seconds_per_beat = 60.0 * minutes_per_beat;
		let pixels_per_beat = self.game_data.brick_speed * seconds_per_beat;
		let beat_pos_pixels = pixels_per_beat / 4.0;
		for brick in &self.current_bricks {
			graphics.append(&mut brick.rendering_instructions());
		}
		
		graphics.append(&mut self.player.lg_rendering_instructions(self.game_data.time_running));
		
		return RenderingInstructions {
			num_graphics: graphics.len(),
			graphics_ptr: graphics.as_ptr()
		}
	}
	
	// returns the number of bricks broken since the last check
	pub fn bricks_broken(&mut self) -> u8 {
		let bb = self.bricks_broken;
		self.bricks_broken = 0;
		return bb;
	}
	
	// returns the songs game data
	pub fn game_data(&self) -> GameData {
		return self.game_data;
	}
	
	// returns all bricks of the song
	pub fn bricks(&self) -> Array {
		let array = Array::new_with_length(self.notes.len() as u32);
		
		let mut i = 0;
		for brick in &self.notes {
			array.set(i, JsValue::from(brick.clone()));
			i += 1;
		}
		return array;
	}
	
	// takes an input command and passes it forward to be handled
	pub fn input_command (&mut self, input: Input) {
		self.player.input(input, self.game_data.time_running);
	}
	
	// takes key release command and passes it forward to be handled
	pub fn stop_command (&mut self, input: Input) {
		self.player.end_input(input);
	}
	
	// select the brick which overlaps with the given brick pos and x pos
	pub fn select_brick(&self, beat_pos: i32, x_pos: i32) -> Option<BrickData> {
		for brick_data in &self.notes {
			if x_pos == brick_data.x_pos {
				if beat_pos == brick_data.beat_pos || (beat_pos > brick_data.beat_pos && beat_pos <= brick_data.end_beat_pos) {
					return Some(brick_data.clone());
				}
			}
			
			if beat_pos < brick_data.beat_pos {
				break;
			}
		}
		
		return None;
	}
	
	// TODO return true/false on success/failure add_brick and remove_brick
	// adds a brick according to the brick's brick data
	pub fn add_brick(&mut self, brick_data: BrickData) {
		self.notes.insert( brick_data );
		
		// !!! alternative data structure to avoid flushing and repopulating vec on each added note
		// !!! on initial load, expensive to do this for each brick
		self.prepare_song();
		self.seek(self.game_data.time_running);
	}
	
	// removes the brick equal to brick_data
	pub fn remove_brick(&mut self, brick_data: BrickData) {
		self.notes.remove( &brick_data ); // TODO alert/log when a value was already there and the brick wasn't updated
		
		// !!! alternative data structure to avoid flushing and repopulating vec on each removed note
		self.prepare_song();
		self.seek(self.game_data.time_running);
	}
	
	fn prepare_song(&mut self) {
		self.bricks = VecDeque::new();
		self.targets = VecDeque::new();

		// data for target info
		let mut player_start_x = 0.0;
		let mut player_start_time = 0.0;
		let mut group_appearance_y;
		let mut group_end_appearance_y;
		let mut group_leftmost;
		let mut group_rightmost;
		let mut group_hold;
		
		match self.notes.first() {
			Some(brick_data) => {
				group_appearance_y = brick_data.appearance_y(self.game_data.bpm, self.game_data.brick_speed);
				group_end_appearance_y = brick_data.end_appearance_y(self.game_data.bpm, self.game_data.brick_speed);
				group_leftmost = brick_data.x_pos;
				group_rightmost = brick_data.x_pos;
				group_hold = brick_data.is_hold_note;
			}
			None => { 
				self.game_data.max_score = 0;
				return; 
			}
		}
		
		for brick_data in &self.notes {
			let appearance_y = brick_data.appearance_y(self.game_data.bpm, self.game_data.brick_speed);
			let end_appearance_y = brick_data.end_appearance_y(self.game_data.bpm, self.game_data.brick_speed);
			let hold_segments;

			if brick_data.is_hold_note {
				// hold length from the bottom of the starting note 
					// to the bottom of the ending note, if there was a note at the end beat pos
				let hold_length_pixels = end_appearance_y - appearance_y;
				// how many brick segments fit into the hold length, floored to a u8
				hold_segments = (hold_length_pixels / (objects::BRICK_SEGMENT_HEIGHT + objects::BRICK_SEGMENT_GAP) as f32) as u8;
			} 
			else {
				hold_segments = 0;
			}
			
			// set brick group info
				// if the appearance y of this brick does not match the appearance y of the brick group,
				// push to target info and start a new brick group
			if appearance_y == group_appearance_y {
				let hold_length = brick_data.end_beat_pos - brick_data.beat_pos;
				group_leftmost = if brick_data.x_pos < group_leftmost { brick_data.x_pos } else { group_leftmost };
				group_rightmost = if brick_data.x_pos > group_rightmost { brick_data.x_pos } else { group_rightmost };
				group_hold = if brick_data.is_hold_note { true } else { group_hold };
				group_end_appearance_y = if end_appearance_y > group_end_appearance_y { end_appearance_y } else { group_end_appearance_y };
			}
			else {
				let ti = self.create_target_info(player_start_x, player_start_time, group_appearance_y, group_leftmost, group_rightmost, group_hold, group_end_appearance_y);

				// start new group
				player_start_x = ti.post_hit_x;
				player_start_time = ti.passed_time;
				group_appearance_y = appearance_y;
				group_end_appearance_y = end_appearance_y;
				group_leftmost = brick_data.x_pos;
				group_rightmost = brick_data.x_pos;
				group_hold = brick_data.is_hold_note;

				self.targets.push_back(ti);
			}

			self.bricks.push_back( UpcomingBrick {
				brick_type: brick_data.brick_type, 
				x: brick_data.x(),
				appearance_y,
				end_y: end_appearance_y + BRICK_HEIGHT as f32,
				hold_segments
			});
			
		}

		// push last target
		let ti = self.create_target_info(player_start_x, player_start_time, group_appearance_y, group_leftmost, group_rightmost, group_hold, group_end_appearance_y);
		self.targets.push_back(ti);
		
		for i in 1 .. self.bricks.len() {
			assert!(self.bricks[i-1] <= self.bricks[i]);
		}

		for i in 1 .. self.targets.len() {
			assert!(self.targets[i-1] < self.targets[i]);
		}

		let mut max_score = 0;
		for brick in &self.bricks {
			max_score += BRICK_SCORE;
			max_score += brick.hold_segments as i32 * HOLD_SEGMENT_SCORE;
		}
		self.game_data.max_score = max_score;
	}

	fn create_target_info(&self, player_start_x: f32, player_start_time: f32, group_appearance_y: f32,
	group_leftmost: i32, group_rightmost: i32, group_hold: bool, group_end_appearance_y: f32) -> TargetInfo {
		// variables to fill for target info
		let target_hit_dir;
		let dash_to_target;
		let boost_to_target;
		let left_target_x = (group_leftmost - 1) as f32 * BRICK_WIDTH as f32 + (BRICK_WIDTH - PLAYER_WIDTH) as f32;
		let right_target_x = (group_rightmost + 1) as f32 * BRICK_WIDTH as f32;
		let dest_x;
		let post_hit_x;
		let end_y = group_end_appearance_y + BRICK_HEIGHT as f32;
		let hittable_time;
		let passed_time;

		// if left of target, right of target, in between targets
		let distance_to_target;
		if left_target_x - player_start_x >= 0.0 {
			target_hit_dir = Direction::Right;
			dest_x = left_target_x;
			distance_to_target = left_target_x - player_start_x;
		} else if player_start_x - right_target_x >= 0.0 {
			target_hit_dir = Direction::Left;
			dest_x = right_target_x;
			distance_to_target = player_start_x - right_target_x;
		} else if player_start_x - left_target_x < right_target_x - player_start_x {
			target_hit_dir = Direction::Left;
			dest_x = right_target_x;
			distance_to_target = player_start_x - left_target_x;
		} else {
			target_hit_dir = Direction::Right;
			dest_x = left_target_x;
			distance_to_target = right_target_x - player_start_x;
		}

		let player_height_time = PLAYER_HEIGHT as f32 / self.game_data.brick_speed;
		let ground_pos_time = GROUND_POS as f32 / self.game_data.brick_speed;
		hittable_time = group_appearance_y / self.game_data.brick_speed - ground_pos_time; // brick rises above ground
		passed_time = end_y / self.game_data.brick_speed - ground_pos_time + player_height_time; // brick rises above player head
		let time_until_target = hittable_time - player_start_time;
		let max_run_distance = time_until_target * RUN_SPEED;

		if distance_to_target > max_run_distance + MAX_BOOST_DISTANCE {
			dash_to_target = true;
			boost_to_target = true;
		}
		else if distance_to_target > max_run_distance {
			dash_to_target = false;
			boost_to_target = true;
		}
		else {
			dash_to_target = false;
			boost_to_target = false;
		}

		match target_hit_dir {
			Direction::Left => { 
				post_hit_x = if group_rightmost - group_leftmost > 0 { left_target_x } else { right_target_x };
			},
			Direction::Right => { 
				post_hit_x = if group_rightmost - group_leftmost > 0 { right_target_x } else { left_target_x };
			}
		};

		let target_info = TargetInfo { appearance_y: group_appearance_y, end_y, dest_x, post_hit_x,
			is_hold_note: group_hold, 
			hit_dir: target_hit_dir, 
			boost_to_target, dash_to_target,
			hittable_time, passed_time };

		return target_info;
	}
	
	// add bricks to current_bricks
	fn add_to_current_bricks(&mut self) {
		while(self.upcoming_brick_idx < self.bricks.len()) {
			let idx = self.upcoming_brick_idx;
			if self.bricks[idx].appearance_y < self.end_y {
				let brick_type = self.bricks[idx].brick_type;
				let hold_segments = self.bricks[idx].hold_segments;

				let x = self.bricks[idx].x;
				let y = self.bricks[idx].appearance_y - self.scrolled_y;
				let appearance_y = self.bricks[idx].appearance_y;
				let end_y = self.bricks[idx].end_y;
				let bounds = ObjectBounds { left_x: x, top_y: y, right_x: x + BRICK_WIDTH as f32, bottom_y: y + BRICK_HEIGHT as f32 };
				
				self.current_bricks.push_back( Brick::new(brick_type, bounds, appearance_y, end_y, hold_segments) );
				self.upcoming_brick_idx += 1;
			} else {
				break;
			}
		}
	}

	// find the hold positions of the current target
	fn find_hold_positions(&self) -> Vec<f32> {
		let mut hold_positions = Vec::with_capacity(32);
		if let Some(target) = self.targets.get(self.target_idx) {
			let appearance_y = target.appearance_y;
			let mut bricks_iter = self.current_bricks.iter();

			loop {
				if let Some(brick) = bricks_iter.next() {
					if brick.appearance_y > appearance_y {
						break;
					} else if brick.appearance_y == appearance_y && brick.is_hold_note() {
						let hold_position = brick.bounds.left_x + (BRICK_WIDTH as f32 / 2.0) - (HOLD_HITBOX_WIDTH as f32 / 2.0);
						hold_positions.push(hold_position);
					}
				} else {
					break;
				}
			}
		}

		return hold_positions;
	}

	// destroy any bricks that overlap with player slash/hold hitboxes
	fn destroy_bricks(&mut self) {
		// TODO might be a little faster to do as bricks are updated
		// TODO more efficient way than checking all bricks, check only bricks that have reached a threshold height
		let score = &mut self.game_data.score;
		let bricks = &mut self.current_bricks;
		let bricks_broken = &mut self.bricks_broken;
		let hitboxes = self.player.hitboxes();
		let hold_hitboxes = self.player.hold_hitboxes();
		bricks.retain_mut(|brick| -> bool {
				let brick_type = brick.brick_type();

				// if not a hold segment check for intersection with regular hitboxes
				if ! brick.is_hold_segment() {
					let mut is_hold_note = false;
					for hitbox in &hitboxes {
						if hitbox.brick_type == brick_type && objects::intersect(&hitbox.bounds, &brick.bounds()) {
							*score += BRICK_SCORE;
							*bricks_broken += 1;
							if brick.attempt_break() {
								return false;
							}

							is_hold_note = true;
						}
					}

					// if it's a hold note, break any hold segments which also intersect with hitboxes
						// (when the top of a hold is broken, also break any hold segments which are hit 
						// so that the hold portion begins strictly after the initial hit)
					if is_hold_note {
						'allhitboxes: loop {
							for hitbox in &hitboxes {
								if hitbox.brick_type == brick_type && objects::intersect(&hitbox.bounds, &brick.bounds()) {
									*score += HOLD_SEGMENT_SCORE;
									*bricks_broken += 1;
									if brick.attempt_break() {
										return false;
									}

									continue 'allhitboxes;
								}
							}

							for hitbox in &hold_hitboxes {
								if hitbox.brick_type == brick_type && objects::intersect(&hitbox.bounds, &brick.bounds()) {
									*score += HOLD_SEGMENT_SCORE;
									*bricks_broken += 1;
									if brick.attempt_break() {
										return false;
									}

									continue 'allhitboxes;
								}
							}

							break 'allhitboxes;
						}
					}
				} else { // else check for intersection with hold hitboxes
					for hitbox in &hold_hitboxes {
						if hitbox.brick_type == brick_type && objects::intersect(&hitbox.bounds, &brick.bounds()) {
							*score += HOLD_SEGMENT_SCORE;
							*bricks_broken += 1;
							if brick.attempt_break() {
								return false;
							}
						}
					}
				}
					
			return true;
		});
	}
	
	// seeks (changes the song time) to the time specified. resets song
	pub fn seek(&mut self, time: f32) {
		self.player = Player::new((BRICK_WIDTH * 2) as f32 - objects::PLAYER_WIDTH as f32 / 2.0);
		self.scrolled_y = self.game_data.brick_speed * time;
		self.end_y = Game::end_y(self.scrolled_y, self.game_data.brick_speed);
		self.game_data.time_running = time;
		self.game_data.score = 0;
		self.bricks_broken = 0;
		
		// set current bricks and upcoming brick idx
		self.current_bricks = VecDeque::new();
		self.upcoming_brick_idx = 0;
		let mut i = 0;
		while(i < self.bricks.len()) {
			// if the end y is on screen
			if self.bricks[i].end_y > self.scrolled_y {
				self.add_to_current_bricks();
				break;
			}
			
			i += 1;
			self.upcoming_brick_idx = i;
		}

		// set target idx
		self.target_idx = 0;
		while(self.target_idx < self.targets.len()) {
			// if the target can still be hit, let it be the current target
			if self.targets[self.target_idx].passed_time > time {
				break;
			}
			
			self.target_idx += 1;
		}
	}
	
	fn end_y(scrolled_y: f32, brick_speed: f32) -> f32 {
		return scrolled_y + GAME_HEIGHT as f32 + brick_speed * 2.0; // 2 second window after bricks are off the screen
	}
}


// Equality and Order are determined only on the appearance y of bricks
impl PartialEq for UpcomingBrick {
	fn eq(&self, other: &UpcomingBrick) -> bool {
		return self.appearance_y == other.appearance_y;
	}
}
impl Eq for UpcomingBrick {}

impl PartialOrd for UpcomingBrick {
	fn partial_cmp(&self, other: &UpcomingBrick) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for UpcomingBrick {
	fn cmp(&self, other: &UpcomingBrick) -> Ordering {
		if self.appearance_y < other.appearance_y { Ordering::Less }
		else if self.appearance_y == other.appearance_y { Ordering::Equal }
		else { Ordering::Greater }
	}
}

// Equality and Order are determined only on the appearance y of bricks
impl PartialEq for TargetInfo {
	fn eq(&self, other: &TargetInfo) -> bool {
		return self.appearance_y == other.appearance_y;
	}
}
impl Eq for TargetInfo {}

impl PartialOrd for TargetInfo {
	fn partial_cmp(&self, other: &TargetInfo) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for TargetInfo {
	fn cmp(&self, other: &TargetInfo) -> Ordering {
		if self.appearance_y < other.appearance_y { Ordering::Less }
		else if self.appearance_y == other.appearance_y { Ordering::Equal }
		else { Ordering::Greater }
	}
}