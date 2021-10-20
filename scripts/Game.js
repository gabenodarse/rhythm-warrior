"use strict";

import * as wasm from "../pkg/music_mercenary.js";
import * as load from "./load.js";

//TODO searching through game to its prototype to find the tick function every tick is technically suboptimal?
export function Game () {
	//members
	this.width;
	this.height;
	this.xFactor;
	this.yFactor;
	
	this.screenDiv;
	this.lastTick;
	this.gameObject;
	this.graphics; // !!! can be either canvases or webGL. Add way to choose between them.
	this.database; 
	this.songData;
	this.isLoaded = false;
	
	this.audioContext = new AudioContext();
	this.audioSource;
	this.audioBuffer;
	this.audioTimeSafetyBuffer = 0.15;
	
	//initialize screen div
	this.screenDiv = document.createElement("div");
	this.screenDiv.style.position = "absolute";
	this.screenDiv.style.top = "0";
	this.screenDiv.style.left = "0";
	this.screenDiv.style.margin = "0";
	this.screenDiv.style.width = "100vw";
	this.screenDiv.style.height = "100vh";
	document.body.appendChild(this.screenDiv);
}

Game.prototype.init = async function () {
	if(this.isLoaded){ return; }
	
	let loader = new load.Loader();
	
	// TODO add error handling
	await loader.init()
		.then( () => loader.loadGraphics("canvases", this.screenDiv)) // >:< canvases or webGL. Make just webGL
		.then( res => this.graphics = res );
		
	// !!! can happen same time as graphics are loading
	this.database = await loader.loadDatabase();
	
	this.gameObject = wasm.Game.new();
	
	let gameDim = wasm.game_dimensions();
	this.width = gameDim.x;
	this.height = gameDim.y;
	this.xFactor = 1;
	this.yFactor = 1;
	
	// !!! don't load song (mp3 and notes from database) on initialization
	// !!! loading arbitrary song... instead should query and load first song, or allow no song to be loaded
	await this.loadSong(7); 
	
	this.isLoaded = true;
}

Game.prototype.resize = function(){
	let width = this.screenDiv.clientWidth;
	let height = this.screenDiv.clientHeight;
	let gameDim = wasm.game_dimensions();
	
	this.width = width;
	this.height = height;
	this.xFactor = width / gameDim.x;
	this.yFactor = height / gameDim.y;
	this.graphics.resize(this.xFactor, this.yFactor);
	this.renderGame();
}

Game.prototype.start = async function (callback) {
	if(!this.audioBuffer){
		let {notes, song} = this.database.loadSong(this.songData.songID);
		
		let filename;
		song[0]["columns"].forEach( (columnName, idx) => {
			if(columnName.toUpperCase() === "FILENAME"){
				filename = song[0]["values"][0][idx];
			}
		});
		
		// fetch the mp3
		// TODO add error handling
		// !!! if file is not found?
		await fetch(filename)
			.then(res => res.arrayBuffer())
			.then(res => this.audioContext.decodeAudioData(res))
			.then(res => { this.audioBuffer = res; }
		);
	}
	
	// "An AudioBufferSourceNode can only be played once; after each call to start(),
		// you have to create a new node if you want to play the same sound again ...
		// you can use these nodes in a "fire and forget" manner" - MDN
	this.audioSource = new AudioBufferSourceNode(this.audioContext, {buffer: this.audioBuffer}); 
	this.audioSource.connect(this.audioContext.destination);
	
	let switchTime = this.audioContext.currentTime + this.audioTimeSafetyBuffer;
	this.audioSource.start(switchTime, this.gameObject.game_data().time_running + this.songData.startOffset); 
	this.lastTick = new Date().getTime() + this.audioTimeSafetyBuffer * 1000; // set the last tick time to after the buffer
	
	// timeout to prevent negative ticks
	setTimeout( () => {
		requestAnimationFrame(callback);
	}, this.audioTimeSafetyBuffer * 1000);
	
}

Game.prototype.pause = function(){
	this.audioSource.stop();
}

Game.prototype.restart = function(){
	this.gameObject.seek(0);
	this.renderGame();
}

Game.prototype.tick = function(){
	let now = new Date().getTime();
	// !!! render asynchronously to keep game ticking???
	// !!! handle if there's too long a time between ticks (pause game?)
	// !!! get fps, average, and log
	let timePassed = (now - this.lastTick) / 1000; // convert to seconds
	this.lastTick = now;
	
	this.gameObject.tick(timePassed); 
	this.songData.gameData = this.gameObject.game_data();
	
	this.renderGame();
}

Game.prototype.startControl = function(cntrl){
	let now = new Date().getTime();
	this.gameObject.input_command(cntrl, (now - this.lastTick) / 1000);
}

Game.prototype.stopControl = function(cntrl){
	let now = new Date().getTime();
	this.gameObject.stop_command(cntrl, (now - this.lastTick) / 1000);
}

Game.prototype.renderGame = function(){
	let instructions = this.gameObject.rendering_instructions();
	
	this.graphics.render(instructions, this.xFactor, this.yFactor);
}

Game.prototype.score = function(){
	return this.gameObject.score();
}

Game.prototype.getSongData = function(){
	return this.songData;
}

Game.prototype.songs = function(){
	let songs = this.database.searchSong();
	
	return songs;
}

// TODO confirmation on deleted data, this and load song
Game.prototype.newSong = function(bpm, brickSpeed, duration, songStartOffset){
	this.gameObject = wasm.Game.new(bpm, brickSpeed, duration);
	this.songData = {
		songID: null,
		name: "",
		artist: "",
		difficulty: "",
		bpm: bpm, // !!! bpm brick speed and duration all also occur in the wasm Game object, consistency not guaranteed
		brickSpeed: brickSpeed,
		duration: duration,
		startOffset: songStartOffset,
		timeCreated: 0,
		timeModified: 0,
		filename: "",
		gameData: this.gameObject.game_data()
	}
}

Game.prototype.loadSong = function(songID){
	// !!! check if current song has been saved (modified flag?) 
		// No need to show a check for regular game usage where songs aren't edited
	// !!! creating a new game to load a new song? Or create a load_song method? wasm garbage collected?
	let {notes, song} = this.database.loadSong(songID);
	
	let name, artist, difficulty, bpm, brickSpeed, duration, startOffset, timeCreated, timeModified, filename;
	song[0]["columns"].forEach( (columnName, idx) => {
		if(columnName.toUpperCase() === "NAME"){
			name = song[0]["values"][0][idx];
		} else if(columnName.toUpperCase() === "ARTIST"){
			artist = song[0]["values"][0][idx];
		} else if(columnName.toUpperCase() === "DIFFICULTY"){
			difficulty = song[0]["values"][0][idx];
		} else if(columnName.toUpperCase() === "BPM"){
			bpm = song[0]["values"][0][idx];
		} else if(columnName.toUpperCase() === "BRICKSPEED"){
			brickSpeed = song[0]["values"][0][idx];
		} else if(columnName.toUpperCase() === "DURATION"){
			duration = song[0]["values"][0][idx];
		} else if(columnName.toUpperCase() === "STARTOFFSET"){
			startOffset = song[0]["values"][0][idx];
		} else if(columnName.toUpperCase() === "TIMECREATED"){
			timeCreated = song[0]["values"][0][idx];
		} else if(columnName.toUpperCase() === "TIMEMODIFIED"){
			timeModified = song[0]["values"][0][idx];
		} else if(columnName.toUpperCase() === "FILENAME"){
			filename = song[0]["values"][0][idx];
		}
	});
	
	this.gameObject = wasm.Game.new(bpm, brickSpeed, duration);
	
	// TODO flimsy way of indexing into notes to retrieve correct values
	if(notes[0]){
		notes[0]["values"].forEach( note => {
			this.gameObject.toggle_brick(note[2], note[3], note[4]); 
		});
	}
	
	this.songData = {
		songID: songID,
		name: name,
		artist: artist,
		difficulty: difficulty,
		bpm: bpm,
		brickSpeed: brickSpeed,
		duration: duration,
		startOffset: startOffset,
		timeCreated: timeCreated,
		timeModified: timeModified,
		filename: "",
		gameData: this.gameObject.game_data()
	}
	
	this.audioBuffer = null;
	
	this.renderGame();
}

Game.prototype.loadMP3 = async function(file){
	// !!! add error handling
	await file.arrayBuffer()
		.then(res => this.audioContext.decodeAudioData(res))
		.then(res => { this.audioBuffer = res; }
	);
}

Game.prototype.saveSong = function(songData, overwrite){
	let notes = JSON.parse(this.gameObject.song_notes_json());
	if(overwrite === true && this.songData.songID !== null){
		this.database.overwriteSong(songData, notes);
	}
	else{
		this.database.saveSong(songData, notes);
	}
}

Game.prototype.toEditor = function(){
	if(this.isLoaded == false) {
		throw Error("game object has not been loaded");
	}
	if(this instanceof Editor){
		return this;
	}
	
	// !!! make broken notes reappear
	Object.setPrototypeOf(this, Editor.prototype);
	
	return this;
}

export function Editor () {
	Game.call(this);
}

Object.setPrototypeOf(Editor.prototype, Game.prototype);

Editor.prototype.seek = function(time){
	this.gameObject.seek(time);
	this.renderGame();
}

Editor.prototype.createNote = function(x, y){
	// !!! support for third, sixth, twelfth notes
	let sixteenthNoteTime = this.gameObject.beat_interval() / 4;
	let t = this.gameObject.song_time();
	
	// set brickT to the time of the song plus an offset accounting for the y position
	y -= wasm.ground_pos() * this.yFactor;
	let brickT = t + y / (this.gameObject.brick_speed() * this.yFactor);
	// round to a sixteenth note 
		// (by adding the difference to the next sixteenthNoteTime, i.e. adding sixteenthNoteTime - positive modulus)
	brickT += sixteenthNoteTime - (brickT % sixteenthNoteTime + sixteenthNoteTime) % sixteenthNoteTime; 
	
	let brickWidth = wasm.graphic_size(wasm.GraphicGroup.Brick1).x * this.xFactor
	let pos = Math.floor(x / brickWidth);
	
	this.gameObject.toggle_brick(0, brickT, pos);
	
	// to have the game add the note. 
	// TODO, more robust way would be to add the note to on screen notes in toggle_brick, then just rerender with renderGame()
	this.seek(t); 
}

Editor.prototype.toGame = function(){
	if(this.isLoaded == false) {
		throw Error("game object has not been loaded");
	}
	
	Object.setPrototypeOf(this, Game.prototype);
	
	return this;
}
	