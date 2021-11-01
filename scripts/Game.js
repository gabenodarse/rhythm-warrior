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
	
	this.div;
	this.lastTick;
	this.gameObject;
	this.graphics; // !!! can be either canvases or webGL. Add way to choose between them.
	this.database; 
	this.songData;
	this.isLoaded = false;
	
	this.audioContext = new AudioContext();
	this.popAudioBuffer;
	this.audioSource;
	this.audioBuffer;
	this.audioTimeSafetyBuffer = 0.15;
	
	//initialize screen div
	this.div = document.createElement("div");
	this.div.id = "game";
	document.getElementById("screen").appendChild(this.div);
}

Game.prototype.init = async function () {
	if(this.isLoaded){ return; }
	
	let loader = new load.Loader();
	
	// initialize loader and load graphics
	// TODO error handling when it takes too long
	await loader.init()
		.then( () => loader.loadGraphics("canvases", this.div)) // >:< canvases or webGL. Make just webGL
		.then( res => this.graphics = res )
		.catch( rej => { throw Error("Error initializing loader / loading assets: " + rej ) });
		
	// load sounds (not songs)
		// !!! once there are more sound effects move to loader
		// TODO error handling when it takes too long
	await fetch("./assets/sounds/pop.wav")
		.then(res => res.arrayBuffer())
		.then(res => this.audioContext.decodeAudioData(res))
		.then(res => this.popAudioBuffer = res)
		.catch(rej => { throw Error("Error loading audio: " + rej) });

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
	let width = this.div.clientWidth;
	let height = this.div.clientHeight;
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
	// set the last tick time to when the moment the game is set to restart
	this.lastTick = new Date().getTime() + this.audioTimeSafetyBuffer * 1000; 
	
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
	
	let bricks_broken = this.gameObject.bricks_broken();
	if(bricks_broken > 0){
		// "An AudioBufferSourceNode can only be played once; after each call to start(),
			// you have to create a new node if you want to play the same sound again ...
			// you can use these nodes in a "fire and forget" manner" - MDN
		let audioSource = new AudioBufferSourceNode(this.audioContext, {buffer: this.popAudioBuffer}); 
		audioSource.connect(this.audioContext.destination);
		audioSource.start();
	}
	
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
	let {bricks, song} = this.database.loadSong(songID);
	
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
	
	let brickTypeIDX, beatPosIDX, endBeatPosIDX, xPosIDX, isTripletIDX, isTrailingIDX, isLeadingIDX, isHoldNoteIDX;
	bricks[0]["columns"].forEach( (columnName, idx) => {
		if(columnName.toUpperCase() === "BRICKTYPE"){
			brickTypeIDX = idx;
		} else if(columnName.toUpperCase() === "BEATPOS"){
			beatPosIDX = idx;
		} else if(columnName.toUpperCase() === "ENDBEATPOS"){
			endBeatPosIDX = idx;
		} else if(columnName.toUpperCase() === "XPOS"){
			xPosIDX = idx;
		} else if(columnName.toUpperCase() === "ISTRIPLET"){
			isTripletIDX = idx;
		} else if(columnName.toUpperCase() === "ISTRAILING"){
			isTrailingIDX = idx;
		} else if(columnName.toUpperCase() === "ISLEADING"){
			isLeadingIDX = idx;
		} else if(columnName.toUpperCase() === "ISHOLDNOTE"){
			isHoldNoteIDX = idx;
		}
	});
	if(bricks[0]){
		bricks[0]["values"].forEach( brick => {
			this.gameObject.add_brick(wasm.BrickData.new( 
				brick[brickTypeIDX], brick[beatPosIDX], brick[endBeatPosIDX], brick[xPosIDX],
				brick[isTripletIDX], brick[isTrailingIDX], brick[isLeadingIDX], brick[isHoldNoteIDX])); 
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
		filename: filename,
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
	let notes = this.gameObject.bricks();
	notes.forEach( note => {
		note.approx_time = note.approx_time(this.songData.bpm);
		console.log(note.approx_time);
	});
	
	
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

// >:<
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