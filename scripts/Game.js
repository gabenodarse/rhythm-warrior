"use strict";

import * as wasm from "../pkg/music_mercenary.js";
import * as load from "./load.js";

export function Game () {
	//members
	this.width;
	this.height;
	this.xFactor;
	this.yFactor;
	
	this.div;
	this.gameObject;
	this.graphics;
	this.database; 
	this.songData;
	this.isLoaded;
	
	this.lastTick; // time since the game last ticked

	// tick timer
	this.numTicksPerTickCalc; // number of ticks for each average calculation
	this.tickCounter; // counts number of ticks since last average calculation
	this.tickTotalTime; // total tick time for the average calculation
	this.minTickTracker; // minimum tick time in this average time calculation
	this.maxTickTracker; // maximum tick time in this average time calculation
	this.minTickTime;
	this.maxTickTime;
	this.averageTickTime;

	this.preRenderTime; // array of average pre-render time, minimum pre-render time, and maximum pre-render time
	
	// audio
	this.audioContext;
	this.popAudioBuffer;
	this.audioSource;
	this.audioBuffer;
	this.audioTimeSafetyBuffer;

	// set defaults
	this.isLoaded = false;

	this.numTicksPerTickCalc = 30;
	this.tickCounter = 0;
	this.tickTotalTime = 0;
	this.minTickTracker = 1000;
	this.maxTickTracker = 0;
	this.minTickTime = 0;
	this.maxTickTime = 0;
	this.averageTickTime = 0;

	// TODO move to init function?
	this.audioContext = new AudioContext();
	this.audioTimeSafetyBuffer = 0.15;
	
	// initialize screen div
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
		.then( () => loader.loadGraphics("canvases", this.div)) // !!! !!! !!! canvases or webGL. Make just webGL
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
	this.preRender();
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
	// set the last tick time to when the moment the game is set to start
	this.lastTick = performance.now() + this.audioTimeSafetyBuffer * 1000; 
	
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
	this.preRender();
}

Game.prototype.tick = function(){
	let now = performance.now();
	// !!! handle if there's too long a time between ticks (pause game?)
	// !!! log fps and tick times and preRender times
	let timePassed = (now - this.lastTick) / 1000; // convert to seconds
	this.lastTick = now;
	
	// tick game state
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

	// tick time tracking
	let endTickTime = performance.now();
	let thisTickTime = endTickTime - now;
	this.tickTotalTime += thisTickTime;
	if(thisTickTime < this.minTickTracker){
		this.minTickTracker = thisTickTime;
	}
	if(thisTickTime > this.maxTickTracker){
		this.maxTickTracker = thisTickTime;
	}
	this.tickCounter += 1;
	if(this.tickCounter == this.numTicksPerTickCalc){
		let averageTickTime = this.tickTotalTime / this.numTicksPerTickCalc;
		this.averageTickTime = averageTickTime / 1000; // convert to seconds
		this.minTickTime = this.minTickTracker;
		this.maxTickTime = this.maxTickTracker;
		
		this.tickTotalTime = 0;
		this.minTickTracker = 1000;
		this.maxTickTracker = 0;
		this.tickCounter = 0;
	}
}

Game.prototype.startControl = function(cntrl){
	let now = performance.now();
	this.gameObject.input_command(cntrl, (now - this.lastTick) / 1000);
}

Game.prototype.stopControl = function(cntrl){
	let now = performance.now();
	this.gameObject.stop_command(cntrl, (now - this.lastTick) / 1000);
}

Game.prototype.preRender = function(){
	let instructions = this.gameObject.rendering_instructions();
	
	this.graphics.preRender(instructions, this.xFactor, this.yFactor);
	this.preRenderTime = this.graphics.getPreRenderTime();
}

Game.prototype.getScore = function(){
	return this.songData.gameData.score;
}

Game.prototype.getTickTime = function(){
	return {average: this.averageTickTime,min: this.minTickTime,max: this.maxTickTime};
}

Game.prototype.getPreRenderTime = function(){
	return this.preRenderTime;
}

Game.prototype.dimensionFactors = function(){
	return {xFactor: this.xFactor, yFactor: this.yFactor};
}

// TODO songData should contain score?
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
	
	if(bricks[0]){
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
	
	this.preRender();
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
		note.approx_time = wasm.BrickData.approx_time(note.beat_pos, this.songData.bpm);
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
	this.songData.gameData = this.gameObject.game_data()
	this.preRender();
}

Editor.prototype.createDefaultBrick = function(beatPos, xPos){
	this.gameObject.add_brick(wasm.BrickData.new(0, beatPos, beatPos, xPos, false, false, false, false));
}

Editor.prototype.createBrick = function(brickType, beatPos, endBeatPos, xPos, isTriplet, isTrailing, isLeading, isHoldNote){
	this.gameObject.add_brick(wasm.BrickData.new(brickType, beatPos, endBeatPos, xPos, isTriplet, isTrailing, isLeading, isHoldNote));
}

Editor.prototype.removeBrick = function(brickType, beatPos, endBeatPos, xPos, isTriplet, isTrailing, isLeading, isHoldNote){
	this.gameObject.remove_brick(wasm.BrickData.new(brickType, beatPos, endBeatPos, xPos, isTriplet, isTrailing, isLeading, isHoldNote));
}

// TODO does not accound for is_trailing is_leading or is_triplet. Ambiguities in the brick selected can lead to bugs.
Editor.prototype.selectBrick = function(beatPos, xPos){
	return this.gameObject.select_brick(beatPos, xPos);
}

Editor.prototype.toGame = function(){
	if(this.isLoaded == false) {
		throw Error("game object has not been loaded");
	}
	
	Object.setPrototypeOf(this, Game.prototype);
	
	return this;
}