
extern crate console_error_panic_hook; // !!! set when a new game is created. Move to its own initialization

use std::collections::btree_set::BTreeSet; 
use std::collections::VecDeque;
use std::cmp::Ordering;

use crate::objects;
use crate::player;

use crate::player::Player;
use crate::brick::HittableBrick;
use crate::BrickData;
use crate::GameData;
use crate::Input;
use crate::GraphicGroup;
use crate::Graphic;
use crate::GraphicFlags;
use crate::PositionedGraphic;
use crate::LingeringGraphic;
use crate::RenderingInstructions;
use crate::AudioInstructions;
use crate::SoundEffect;
use objects::Object;
use objects::HitBox;
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
use objects::BRICK_SEGMENT_GAP;
use objects::BRICK_WIDTH;
use objects::HOLD_HITBOX_WIDTH;
use player::RUN_SPEED;

const MAX_TIME_BETWEEN_TICKS: f32 = 0.025;
const MAX_BOOST_DISTANCE: f32 = 4.0 * PLAYER_WIDTH as f32;
const BRICK_SCORE: i32 = 100;
const HOLD_SEGMENT_SCORE: i32 = 10;
pub const DASH_INDICATOR_WIDTH: i32 = 85;
pub const DASH_INDICATOR_HEIGHT: i32 = 60;
const MISS_EFFECT_TIME: f32 = 0.2;

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
	pub brick_group: VecDeque<HittableBrick>,
	pub appearance_y: f32,
	pub end_y: f32,
	pub dest_x: f32, // where the player left_x should be to hit the target
	pub post_hit_x: f32, // where the player left_x should be after hitting the target
	pub is_hold_note: bool,
	pub hit_dir: Direction, // direction player must slash in order to hit targets once he arrives at dest_x
	
	pub dash_to_target: bool, // whether the player should dash in order to reach dest_x on time
	pub hittable_time: f32,
	pub passed_time: f32
}

#[wasm_bindgen]
pub struct Game {
	player: Player,
	// !!! better data structures than VecDeques. Indexable BTrees
	bricks: VecDeque<UpcomingBrick>, // all bricks of the song, ordered
	// uses a vec instead of a btree because std lib btreeset is unindexable
	targets: VecDeque<TargetInfo>, // all targets of the song, in increasing order based on appearance y and with only one target for any appearance y
	// TODO combine target_idx, last_target_missed, scrolled_y, and end_y into some sort of current state struct
		// current state struck may want to store an updated target (dash or no dash due to last_target_missed boolean)
	target_idx: usize,
	last_target_missed: bool, // true when the last target was missed
	scrolled_y: f32, // how much y has been scrolled (time 0 has a scrolled y of 0)
	end_y: f32, // y value of the bottom of the screen plus a 2 second window of bricks scrolling (so bricks offscreen may be loaded early)
	game_data: GameData, 
	// TODO only store bricks, not the notes
	notes: BTreeSet<BrickData>, // all notes of the song before conversion into bricks
	rendering_instructions_buf: Vec<PositionedGraphic>,
	audio_instructions_flags: [bool; 128], // !!! size must be greater than the number of SoundEffects
	audio_instructions_buf: Vec<u8>,
	game_graphics: Vec<LingeringGraphic>, // graphic effects on the game (passed to graphics when javascript requests rendering data)
}

#[wasm_bindgen]
impl Game {
	pub fn new(bpm: f32, brick_speed: f32, duration: f32) -> Game {
		console_error_panic_hook::set_once();
		
		return Game {
			player: Player::new((BRICK_WIDTH * 2) as f32 - objects::PLAYER_WIDTH as f32 / 2.0),
			bricks: VecDeque::new(), // all bricks of the song, ordered by time they are meant to be played
			targets: VecDeque::new(),
			target_idx: 0,
			last_target_missed: false,
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
				is_modified: false
			},
			notes: BTreeSet::new(),
			rendering_instructions_buf: Vec::with_capacity(512), // TODO what should the upper limit be? Make it a hard limit
			audio_instructions_flags: [false; 128],
			audio_instructions_buf: Vec::with_capacity(32),
			game_graphics: Vec::with_capacity(12)
		};
	}
			
	// tick the game state by the given amount of time
	pub fn tick(&mut self, mut seconds_passed: f32) {
		self.audio_instructions_flags = [false; 128];
		
		// prevent disproportionally long ticks
		if seconds_passed > MAX_TIME_BETWEEN_TICKS { 
			self.tick(seconds_passed - MAX_TIME_BETWEEN_TICKS);
			seconds_passed = MAX_TIME_BETWEEN_TICKS;
		}
		
		// check for any actions that happened mid tick. Either action_tick (if the player is slashing) or regular tick
		let end_tick_time = self.game_data.time_running + seconds_passed;
		if let Some(action_time) = self.player.check_action(self.game_data.time_running, end_tick_time) {
			let pre_action_time = action_time - self.game_data.time_running;
			let post_action_time = seconds_passed - pre_action_time;
			
			// scroll screen to the time of action
			let delta_y = pre_action_time * self.game_data.brick_speed;
			self.scrolled_y += delta_y;
			self.end_y += delta_y;
			self.game_data.time_running += pre_action_time;
			
			// action tick the player, check for brick destruction
			let hitbox = self.player.action_tick(&self.game_data);
			self.destroy_bricks(&hitbox);
			
			// scroll screen the rest of the tick time
			let delta_y = post_action_time * self.game_data.brick_speed;
			self.scrolled_y += delta_y;
			self.end_y += delta_y;
			self.game_data.time_running += post_action_time;
			
			// second, forgiveness check for brick destruction (if action occurs in the same tick that notes become hittable, always destroy the bricks)
			self.destroy_bricks(&hitbox);
		} 
		else {
			// tick the player
			self.player.tick(seconds_passed, &self.game_data);
						
			// check for hold brick destruction (first of two times, to account for hold notes which may sneak past hitbox)
			self.destroy_holds();
			
			// scroll screen
			let delta_y = seconds_passed * self.game_data.brick_speed;
			self.scrolled_y += delta_y;
			self.end_y += delta_y;
			self.game_data.time_running += seconds_passed;
		}
		
		// check for hold brick destruction (second of two times)
		self.destroy_holds();
			
		// update target if all its bricks are destroyed
		let mut all_destroyed = true;
		if let Some(ti) = self.targets.get(self.target_idx) {
			for brick in &ti.brick_group {
				if !brick.is_broken() {
					all_destroyed = false;
				}
			}
		}
		if all_destroyed && self.target_idx < self.targets.len() {
			self.target_idx += 1;
			self.last_target_missed = false;
		}
		
		// stun player if a brick has hit the top of the screen
		if let Some(ti) = self.targets.get(self.target_idx) {
			if ti.appearance_y < self.scrolled_y {
				let top_y = ti.appearance_y - self.scrolled_y;
				let mut illegal_bricks = false;
				
				for brick in &ti.brick_group {
					if let Some(brick_bounds) = brick.bounds(top_y) {
						if brick_bounds.top_y < 0.0 {
							illegal_bricks = true;
							break;
						}
					}
				}
				
				if illegal_bricks {
					// set new target beyond all targets that are in the game section (which are forgotten)
					loop {
						if self.target_idx >= self.targets.len() {
							break;
						} else if self.targets[self.target_idx].appearance_y > self.end_y {
							break;
						} else {
							self.target_idx += 1;
						}
					}
					
					// visual effect
					let graphic = Graphic{ g: GraphicGroup::MissEffect, frame: 0, flags: 0, arg: 0 };
					let positioned_graphic = PositionedGraphic::new(graphic, 0.0, 0.0);
					self.game_graphics.push(LingeringGraphic {
						positioned_graphic,
						start_t: self.game_data.time_running,
						end_t: self.game_data.time_running + MISS_EFFECT_TIME
					});
					
					self.player.stun(self.game_data.time_running);
					self.last_target_missed = true;
				}
			}
		}
		
		// update player's target
		let mut target = None; 
		if let Some(ti) = self.targets.get(self.target_idx) {
			if ti.appearance_y > self.end_y {
				target = None;
			}
			else {
				if self.last_target_missed {
					let mut ti = ti.clone();
					ti.dash_to_target = false;
					target = Some(ti.clone());
				}
				else {
					target = Some(ti.clone());
				}
			}
		}
		self.player.update_target(target);
	}
	
	// updates the audio instructions and returns a pointer to access them
	pub fn audio_instructions(&mut self) -> AudioInstructions {
		let audio_instructions_buf = &mut self.audio_instructions_buf;
		audio_instructions_buf.clear();
		
		let mut audioKey = 0;
		for val in self.audio_instructions_flags {
			if val {
				audio_instructions_buf.push(audioKey);
			}
			
			audioKey += 1;
		}
		
		return AudioInstructions {
			num_instructions: audio_instructions_buf.len(),
			instructions_ptr: audio_instructions_buf.as_ptr()
		};
	}
	
	// updates the rendering instructions and returns a pointer to access them
	pub fn rendering_instructions(&mut self) -> RenderingInstructions {
		let rendering_instructions_buf = &mut self.rendering_instructions_buf;
		let game_graphics = &mut self.game_graphics;
		
		rendering_instructions_buf.clear();
		
		// push background
		rendering_instructions_buf.push(
			PositionedGraphic::new(Graphic{ g: GraphicGroup::Background, frame: 0, flags: 0, arg: 0}, 0.0, 0.0)
		);
		
		// push game effects
		let time_running = self.game_data.time_running;
		game_graphics.retain_mut(|lg| -> bool {
			if lg.end_t > time_running {
				let mut pg = lg.positioned_graphic.clone();
				let proportion_time_passed = (time_running - lg.start_t) / (lg.end_t - lg.start_t);
				
				pg.g.flags |= GraphicFlags::Opacity as u8;
				pg.g.arg = 255 - (proportion_time_passed * 255.0) as u8;
				rendering_instructions_buf.push(pg);
				return true;
			} else {
				return false;
			}
		});
		
		// push a dash indicator for the current target
		if let Some(ti) = self.targets.get(self.target_idx) {
			if ti.appearance_y <= self.end_y && ti.dash_to_target && !self.last_target_missed && !self.player.check_in_range() {
				let pg;
				match ti.hit_dir {
					Direction::Left => {
						let graphic_x = ti.dest_x;
						let graphic_y = ti.appearance_y - self.scrolled_y + (BRICK_HEIGHT as f32 - DASH_INDICATOR_HEIGHT as f32) / 2.0;
						pg = PositionedGraphic::new(Graphic{ g: GraphicGroup::DashIndicator , frame: 0, flags: GraphicFlags::HorizontalFlip as u8, arg: 0}, graphic_x, graphic_y);
					},
					Direction::Right => {
						let graphic_x = ti.dest_x + PLAYER_WIDTH as f32 - DASH_INDICATOR_WIDTH as f32;
						let graphic_y = ti.appearance_y - self.scrolled_y + (BRICK_HEIGHT as f32 - DASH_INDICATOR_HEIGHT as f32) / 2.0;
						pg = PositionedGraphic::new(Graphic{ g: GraphicGroup::DashIndicator , frame: 0, flags: 0, arg: 0}, graphic_x, graphic_y);
					}
				}
				
				rendering_instructions_buf.push( pg );
			}
		}
		// push dash indicators for targets after the current target
		let mut idx = self.target_idx + 1;
		loop {
			if let Some(ti) = self.targets.get(idx) {
				if ti.appearance_y <= self.end_y && ti.dash_to_target {
					let pg;
					match ti.hit_dir {
						Direction::Left => {
							let graphic_x = ti.dest_x;
							let graphic_y = ti.appearance_y - self.scrolled_y + (BRICK_HEIGHT as f32 - DASH_INDICATOR_HEIGHT as f32) / 2.0;
							pg = PositionedGraphic::new(Graphic{ g: GraphicGroup::DashIndicator , frame: 0, flags: GraphicFlags::HorizontalFlip as u8, arg: 0}, graphic_x, graphic_y);
						},
						Direction::Right => {
							let graphic_x = ti.dest_x + PLAYER_WIDTH as f32 - DASH_INDICATOR_WIDTH as f32;
							let graphic_y = ti.appearance_y - self.scrolled_y + (BRICK_HEIGHT as f32 - DASH_INDICATOR_HEIGHT as f32) / 2.0;
							pg = PositionedGraphic::new(Graphic{ g: GraphicGroup::DashIndicator , frame: 0, flags: 0, arg: 0}, graphic_x, graphic_y);
						}
					}
					
					rendering_instructions_buf.push( pg );
				}
				
				if ti.appearance_y > self.end_y {
					break;
				}
				
				idx += 1;
			} else {
				break;
			}
		}
		
		// push player graphics
		rendering_instructions_buf.append(&mut self.player.rendering_instructions(self.game_data.time_running));
		rendering_instructions_buf.append(&mut self.player.lg_rendering_instructions(self.game_data.time_running));
		
		// push bricks' graphics
		let mut idx = self.target_idx;
		loop {
			if idx >= self.targets.len() {
				break;
			}
			
			let ti = &self.targets[idx];
			
			if ti.appearance_y > self.end_y {
				break;
			}
			
			let target_y = ti.appearance_y - self.scrolled_y;
			for brick in &ti.brick_group {
				let mut brick_graphics = brick.rendering_instructions(target_y);
				rendering_instructions_buf.append(&mut brick_graphics);
			}
			
			idx += 1;
		}
		
		return RenderingInstructions {
			num_graphics: rendering_instructions_buf.len(),
			graphics_ptr: rendering_instructions_buf.as_ptr()
		}
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
	pub fn input_command (&mut self, input: Input, time_since_tick: f32) {
		let input_time = self.game_data.time_running + time_since_tick;
		
		self.player.input(input, input_time);
	}
	
	// takes key release command and passes it forward to be handled
	pub fn stop_command (&mut self, input: Input) {
		self.player.end_input(input);
	}
	
	// adds a brick to the song without setting the is_modified flag to true or calling seek()
	pub fn initial_load_add_brick(&mut self, brick_data: BrickData) {
		self.notes.insert( brick_data );
	}
	
	// TODO return true/false on success/failure add_brick and remove_brick
	// adds a brick according to the brick's brick data
	pub fn add_brick(&mut self, brick_data: BrickData) {
		self.notes.insert( brick_data );
		self.game_data.is_modified = true;
		
		self.seek(self.game_data.time_running);
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
	
	// removes the brick equal to brick_data
	pub fn remove_brick(&mut self, brick_data: BrickData) {
		self.notes.remove( &brick_data ); // TODO alert/log when a value was already there and the brick wasn't updated
		self.game_data.is_modified = true;
		
		self.seek(self.game_data.time_running);
	}
	
	fn prepare_song(&mut self) {
		self.bricks = VecDeque::new();
		self.targets = VecDeque::new();

		// populate self.bricks
		for brick_data in &self.notes {
			let appearance_y = brick_data.appearance_y(self.game_data.bpm, self.game_data.brick_speed);
			let end_appearance_y = brick_data.end_appearance_y(self.game_data.bpm, self.game_data.brick_speed);
			let hold_segments;
			
			if brick_data.is_hold_note {
				// hold length from the bottom of the starting note 
					// to the bottom of the ending note, if there was a full note (rather than just a hold segment) at the end beat pos
				let hold_length_pixels = end_appearance_y - appearance_y;
				// how many brick segments fit into the hold length, floored to a u8
				hold_segments = (hold_length_pixels / (objects::BRICK_SEGMENT_HEIGHT + objects::BRICK_SEGMENT_GAP) as f32) as u8;
			} 
			else {
				hold_segments = 0;
			}
			
			let end_y = appearance_y + BRICK_HEIGHT as f32 + (BRICK_SEGMENT_HEIGHT + BRICK_SEGMENT_GAP) as f32 * hold_segments as f32;
			self.bricks.push_back( UpcomingBrick {
				brick_type: brick_data.brick_type, 
				x: brick_data.x(),
				appearance_y,
				end_y,
				hold_segments
			});
		}
		
		// populate self.targets
		let mut player_start_x = 0.0;
		let mut player_start_time = 0.0;
		let mut brick_group: VecDeque<HittableBrick> = VecDeque::new();
		let mut group_appearance_y: f32 = 0.0;
		
		for upcoming_brick in &self.bricks {
			if upcoming_brick.appearance_y == group_appearance_y {
				brick_group.push_back( HittableBrick::new(upcoming_brick.brick_type, upcoming_brick.x, upcoming_brick.hold_segments));
			}
			else {
				if brick_group.len() > 0 {
					let new_target = self.create_target_info(player_start_x, player_start_time, brick_group, group_appearance_y);
					
					self.targets.push_back(new_target.clone());
					brick_group = VecDeque::new();
					player_start_x = new_target.post_hit_x;
					player_start_time = new_target.passed_time;
				}
				
				group_appearance_y = upcoming_brick.appearance_y;
				brick_group.push_back( HittableBrick::new(upcoming_brick.brick_type, upcoming_brick.x, upcoming_brick.hold_segments));
			}
		}
		if brick_group.len() > 0 {
			let last_target = self.create_target_info(player_start_x, player_start_time, brick_group, group_appearance_y);
			self.targets.push_back(last_target.clone());
		}
		
		for i in 1 .. self.bricks.len() {
			assert!(self.bricks[i-1] <= self.bricks[i]);
		}

		// assert targets are in increasing order (based on appearance y) and there is only one target for a given appearance y
		for i in 1 .. self.targets.len() {
			assert!(self.targets[i-1].appearance_y < self.targets[i].appearance_y);
		}

		let mut max_score = 0;
		for brick in &self.bricks {
			max_score += BRICK_SCORE;
			max_score += brick.hold_segments as i32 * HOLD_SEGMENT_SCORE;
		}
		self.game_data.max_score = max_score;
	}

	fn create_target_info(&self, player_start_x: f32, player_start_time: f32, brick_group: VecDeque<HittableBrick>, group_appearance_y: f32) -> TargetInfo {
		if brick_group.len() < 1 {
			panic!();
		}
		
		// get data from brick_group
		let mut group_left_x = brick_group[0].x();
		let mut group_right_x = brick_group[0].x() + BRICK_WIDTH as f32;
		let mut max_hold_segments = brick_group[0].hold_segments();
		for brick in &brick_group {
			if brick.x() < group_left_x {
				group_left_x = brick.x();
			}
			if brick.x() + BRICK_WIDTH as f32 > group_right_x {
				group_right_x = brick.x() + BRICK_WIDTH as f32;
			}
			if brick.hold_segments() > max_hold_segments {
				max_hold_segments = brick.hold_segments();
			}
		}
		
		// variables to fill for target info
		let target_hit_dir;
		let dash_to_target;
		let left_target_x = group_left_x - PLAYER_WIDTH as f32;
		let right_target_x = group_right_x;
		let dest_x;
		let post_hit_x;
		let group_end_y = group_appearance_y + BRICK_HEIGHT as f32 + (BRICK_SEGMENT_HEIGHT + BRICK_SEGMENT_GAP) as f32 * max_hold_segments as f32;
		let hittable_time;
		let passed_time;
		let group_hold = if max_hold_segments > 0 { true } else { false };

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
			target_hit_dir = Direction::Right;
			dest_x = left_target_x;
			distance_to_target = player_start_x - left_target_x;
		} else {
			target_hit_dir = Direction::Left;
			dest_x = right_target_x;
			distance_to_target = right_target_x - player_start_x;
		}

		let player_height_time = PLAYER_HEIGHT as f32 / self.game_data.brick_speed;
		let ground_pos_time = GROUND_POS as f32 / self.game_data.brick_speed;
		hittable_time = group_appearance_y / self.game_data.brick_speed - ground_pos_time; // brick rises above ground
		passed_time = group_end_y / self.game_data.brick_speed - ground_pos_time + player_height_time; // brick rises above player head
		let time_until_target = hittable_time - player_start_time;
		let max_run_distance = time_until_target * RUN_SPEED;

		if distance_to_target > max_run_distance + MAX_BOOST_DISTANCE {
			dash_to_target = true;
		}
		else if distance_to_target > max_run_distance {
			dash_to_target = false;
		}
		else {
			dash_to_target = false;
		}

		match target_hit_dir {
			Direction::Left => { 
				post_hit_x = if brick_group.len() > 1 { left_target_x } else { right_target_x };
			},
			Direction::Right => { 
				post_hit_x = if brick_group.len() > 1 { right_target_x } else { left_target_x };
			}
		};

		let target_info = TargetInfo { 
			brick_group,
			appearance_y: group_appearance_y, 
			end_y: group_end_y, 
			dest_x, 
			post_hit_x,
			is_hold_note: group_hold, 
			hit_dir: target_hit_dir, 
			dash_to_target,
			hittable_time, 
			passed_time };

		return target_info;
	}

	// destroy any bricks that overlap with passed hitboxes
	fn destroy_bricks(&mut self, hitbox: &HitBox) {
		let score = &mut self.game_data.score;
		let mut new_hold_positions = Vec::new();
		
		if let Some(ti) = self.targets.get_mut(self.target_idx) {
			let target_y = ti.appearance_y - self.scrolled_y;
			for brick in &mut ti.brick_group {
				if brick.is_broken() || brick.is_hold_segment() {
					continue;
				}
				let brick_type = brick.brick_type();
				let brick_bounds = match brick.bounds(target_y) {
					None => { continue; },
					Some(b) => { b }
				};
				
				if hitbox.brick_type == brick_type && objects::intersect(&hitbox.bounds, &brick_bounds) {
					*score += BRICK_SCORE;
					self.audio_instructions_flags[SoundEffect::NoteHit as usize] = true;
					
					if brick.attempt_break() {
						continue;
					}
					
					// any broken hold notes are added to the Game's hold positions
					let hold_position = brick_bounds.left_x + (BRICK_WIDTH as f32 / 2.0) - (HOLD_HITBOX_WIDTH as f32 / 2.0);
					new_hold_positions.push(hold_position);
				}
				else {
					continue;
				}

				// it's a hold note, break any hold segments which also intersect with the hitbox
					// (when the top of a hold is broken, also break any hold segments which are hit 
					// so that the hold portion begins strictly after the initial hit)
				loop {
					let segment_brick_bounds = match brick.bounds(target_y) {
						None => { break; },
						Some(b) => { b }
					};;
					if objects::intersect(&hitbox.bounds, &segment_brick_bounds) {
						*score += HOLD_SEGMENT_SCORE;
						if brick.attempt_break() {
							break;
						}

						continue;
					}

					break;
				}
			}
		}
		
		if new_hold_positions.len() > 0 {
			self.player.update_hold_positions(new_hold_positions);
		}
	}
	
	// destroy any hold segments that overlap with player hold hitboxes
	fn destroy_holds(&mut self) {
		let score = &mut self.game_data.score;
		let hold_hitboxes = self.player.hold_hitboxes();
		
		if let Some(ti) = self.targets.get_mut(self.target_idx) {
			let target_y = ti.appearance_y - self.scrolled_y;
			for brick in &mut ti.brick_group {
				if brick.is_broken() || !brick.is_hold_segment() {
					continue;
				}
				
				let brick_type = brick.brick_type();
				if let Some(segment_brick_bounds) = brick.bounds(target_y) {
					for hitbox in &hold_hitboxes {
						if hitbox.brick_type == brick_type && objects::intersect(&hitbox.bounds, &segment_brick_bounds) {
							self.audio_instructions_flags[SoundEffect::NoteSegmentHit as usize] = true;
							*score += HOLD_SEGMENT_SCORE;
							brick.attempt_break();
							break;
						}
					}
				}
			}
		}
	}
	
	// seeks (changes the song time) to the time specified. resets song
	pub fn seek(&mut self, time: f32) {
		// !!! find way to avoid flushing and repopulating bricks/targets on every seek
			// each target's brick_group needs to be reset somehow
			// duplicate targets (one unbroken and one destroyable?)
			// or (probably best) only include targets that are on the gamespace (saves memory, need to create targets during game)
		self.prepare_song();
		
		let time = if time < 0.0 { 0.0 } else { time };
		
		self.player = Player::new((BRICK_WIDTH * 2) as f32 - objects::PLAYER_WIDTH as f32 / 2.0);
		self.last_target_missed = false;
		self.scrolled_y = self.game_data.brick_speed * time;
		self.end_y = Game::end_y(self.scrolled_y, self.game_data.brick_speed);
		self.game_data.time_running = time;
		self.game_data.score = 0;

		// set target idx
		self.target_idx = 0;
		while(self.target_idx < self.targets.len()) {
			// if the target can still be hit, let it be the current target
			if self.targets[self.target_idx].end_y > self.scrolled_y {
				break;
			}
			
			self.target_idx += 1;
		}
		
		// update player's target
		let mut target = None; 
		if let Some(ti) = self.targets.get(self.target_idx) {
			if ti.appearance_y > self.end_y {
				target = None;
			}
			else {
				target = Some(ti.clone());
			}
		}
		
		// last_target_missed is false, so just pass the target without checking
		self.player.update_target(target);
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