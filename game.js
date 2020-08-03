"use strict";

import * as wasm from "./pkg/music_mercenary.js";
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
	this.gameData;
	this.graphics; // !!! can be either canvases or webGL. Asdd way to choose between them.
	this.database;
	this.song;
	this.isLoaded = false;
	
	this.audioContext = new AudioContext();
	this.audioSource;
	this.audioBuffer;
	this.audioTimeSafetyBuffer = 0.15;
	
	this.brickBreakAudioContext = new AudioContext();
	this.brickBreakAudioSource;
	this.brickBreakAudioBuffer;
	
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

Game.prototype.load = async function () {
	if(this.isLoaded){ return; }
	
	let loader = new load.Loader();
	
	// TODO add error handling
	await loader.init()
		.then( () => loader.loadGraphics("webGL", this.screenDiv)) //canvases or webGL
		.then( res => this.graphics = res );
		
	// !!! can happen same time as graphics are loading
	this.database = await loader.loadDatabase();
	
	// !!! associate with Songs table in database and load as needed using loader
	// TODO add error handling
	await fetch("song.mp3")
		.then(res => res.arrayBuffer())
		.then(res => this.audioContext.decodeAudioData(res))
		.then(res => { this.audioBuffer = res; }
	);
	
	// !!! make part of loader
	await fetch("pop.wav")
		.then(res => res.arrayBuffer())
		.then(res => this.brickBreakAudioContext.decodeAudioData(res))
		.then(res => { 
			this.brickBreakAudioBuffer = res; 
			this.brickBreakAudioSource = this.brickBreakAudioContext.createBufferSource(); 
			this.brickBreakAudioSource.buffer = this.brickBreakAudioBuffer;
			this.brickBreakAudioSource.connect(this.brickBreakAudioContext.destination);
		}
	);
	
	this.gameData = wasm.Game.new();
	
	let gameDim = wasm.game_dimensions();
	this.width = gameDim.x;
	this.height = gameDim.y;
	this.xFactor = 1;
	this.yFactor = 1;
	
	this.loadSong(6);
	
	this.isLoaded = true;
}

Game.prototype.toEditor = async function(){
	if(!(this instanceof Editor)){
		// !!! make broken notes reappear
		Object.setPrototypeOf(this, Editor.prototype);
		Editor.call(this);
	}
	
	await this.load();
	return this;
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

Game.prototype.start = function (callback) {
	// !!! creating a new buffer source each time because I couldn't figure out how to resume audio precisely
		// make sure multiple buffer sources don't linger in memory
	this.audioSource = this.audioContext.createBufferSource(); 
	this.audioSource.buffer = this.audioBuffer;
	this.audioSource.connect(this.audioContext.destination);
	
	let switchTime = this.audioContext.currentTime + this.audioTimeSafetyBuffer;
	this.audioSource.start(switchTime, this.gameData.song_time());
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
	this.gameData.seek(0);
	this.renderGame();
}

Game.prototype.tick = function(){
	let now = new Date().getTime();
	// !!! render asynchronously to keep game ticking???
	// !!! handle if there's too long a time between ticks (pause game?)
	// !!! get fps, average, and log
	let timePassed = (now - this.lastTick) / 1000; // convert to seconds
	this.gameData.tick(timePassed); 
	this.lastTick = now;
	this.renderGame();
	if( this.gameData.bricks_broken() > 0 ){
		// play sound then create a new audio source in preparation for subsequent bricks breaking
		this.brickBreakAudioSource.start();
		this.brickBreakAudioSource = this.brickBreakAudioContext.createBufferSource(); 
		this.brickBreakAudioSource.buffer = this.brickBreakAudioBuffer;
		this.brickBreakAudioSource.connect(this.brickBreakAudioContext.destination);
	}
}

Game.prototype.startControl = function(cntrl){
	let now = new Date().getTime();
	this.gameData.input_command(cntrl, (now - this.lastTick) / 1000);
}

Game.prototype.stopControl = function(cntrl){
	let now = new Date().getTime();
	this.gameData.stop_command(cntrl, (now - this.lastTick) / 1000);
}

Game.prototype.renderGame = function(){
	let instructions = this.gameData.rendering_instructions();
	
	this.graphics.render(instructions, this.xFactor, this.yFactor);
}

Game.prototype.score = function(){
	return this.gameData.score();
}

Game.prototype.songData = function(){
	this.song.beatInterval = this.gameData.beat_interval();
	this.song.songTime = this.gameData.song_time();
	this.song.score = this.gameData.score()
	return this.song;
}

Game.prototype.songs = function(){
	let songs = this.database.searchSong();
	
	return songs;
}

Game.prototype.newSong = function(bpm, brickSpeed, duration){
	// !!! check if current song has been saved (modified flag?) 
		// No need to show a check for regular game usage where songs aren't edited
	// !!! creating a new game to load a new song? Or create a load_song method? wasm garbage collected?
	let createSong = true;
	if(createSong){
		this.gameData = wasm.Game.new(bpm, brickSpeed, duration);
		this.song = {};
		this.song.bpm = bpm;
		this.song.brickSpeed = brickSpeed;
		this.song.duration = duration;
	}
	
	return createSong;
}

Game.prototype.loadSong = function(songID){
	
	let {notes, song} = this.database.loadSong(songID);
	let songValues = {};
	
	song[0]["columns"].forEach( (columnName, idx) => {
		if(columnName.toUpperCase() === "SONGID"){
			songValues.songID = song[0]["values"][0][idx];
		}
		else if(columnName.toUpperCase() === "NAME"){
			songValues.name = song[0]["values"][0][idx];
		}
		else if(columnName.toUpperCase() === "ARTIST"){
			songValues.artist = song[0]["values"][0][idx];
		}
		else if(columnName.toUpperCase() === "DIFFICULTY"){
			songValues.difficulty = song[0]["values"][0][idx];
		}
		else if(columnName.toUpperCase() === "BPM"){
			songValues.bpm = song[0]["values"][0][idx];
		}
		else if(columnName.toUpperCase() === "BRICKSPEED"){
			songValues.brickSpeed = song[0]["values"][0][idx];
		}
		else if(columnName.toUpperCase() === "DURATION"){
			songValues.duration = song[0]["values"][0][idx];
		}
		else if(columnName.toUpperCase() === "TIMECREATED"){
			songValues.timeCreated = song[0]["values"][0][idx];
		}
	});
	
	if(this.newSong(songValues.bpm, songValues.brickSpeed, songValues.duration) == true){
		// TODO flimsy way of indexing into notes to retrieve correct values
		notes[0]["values"].forEach( note => {
			this.gameData.toggle_brick(note[2], note[3], note[4]); 
		});
	}
	
	this.song = songValues;
	// TODO doesn't show song immediately after load, and seek isn't a member of Game so can't seek to show
	this.renderGame();
}

Game.prototype.saveSong = function(songData, overwrite){
	let notes = JSON.parse(this.gameData.song_notes_json());
	if(overwrite === true){
		songData.songID = this.song.songID;
		this.database.overwriteSong(songData, notes);
	}
	else{
		this.database.saveSong(songData, notes);
	}
}

export function Editor () {
	this.editorLoaded;
	this.notes;
	this.notesIdx;
	
	this.playTickingSound = false;
	this.tickAudioContext;
	this.tickAudioBuffer;
	this.tickAudioSource;
	
	if(!(this instanceof Game)){
		Game.call(this);
	}
	
	this.editorLoaded = false;
	this.tickAudioContext = new AudioContext();
}

Object.setPrototypeOf(Editor.prototype, Game.prototype);

Editor.prototype.load = async function(){
	if(!this.isLoaded){
		await Game.prototype.load.call(this);
	}
	if(this.editorLoaded){
		return;
	}
	
	let json = JSON.parse(this.gameData.song_notes_json());
	this.notes = [];
	this.notesIdx = 0;
	json.forEach( note => {
		this.notes.push(note.time);
	});
	
	await fetch("tick.wav")
		.then(res => res.arrayBuffer())
		.then(res => this.tickAudioContext.decodeAudioData(res))
		.then(res => { 
			this.tickAudioBuffer = res; 
			this.tickAudioSource = this.tickAudioContext.createBufferSource(); 
			this.tickAudioSource.buffer = this.tickAudioBuffer;
			this.tickAudioSource.connect(this.tickAudioContext.destination);
		}
	);
	
	this.editorLoaded = true;
}

Editor.prototype.tick = function(){
	Game.prototype.tick.call(this);
	
	
	let notePassed = false;
	while(this.notes[this.notesIdx] < this.gameData.song_time()){
		++this.notesIdx;
		notePassed = true;
	}
	if(this.playTickingSound === true && notePassed){
		this.tickAudioSource.start();
		this.tickAudioSource = this.tickAudioContext.createBufferSource(); 
		this.tickAudioSource.buffer = this.tickAudioBuffer;
		this.tickAudioSource.connect(this.tickAudioContext.destination);
	}
}

Editor.prototype.toggleTickingSound = function(){
	if(this.playTickingSound === true){
		this.playTickingSound = false;
	}
	else{
		this.playTickingSound = true;
	}
}

Editor.prototype.seek = function(time){
	this.gameData.seek(time);
	this.renderGame();
	
	this.notesIdx = 0;
	for(let i = 0; i < this.notes.length; ++i){
		if(time > this.notes[i]){
			this.notesIdx = i;
		}
	}
}

Editor.prototype.createNote = function(x, y){
	// !!! support for third, sixth, twelfth notes
	let sixteenthNoteTime = this.gameData.beat_interval() / 4;
	let t = this.gameData.song_time();
	
	y -= wasm.ground_pos() * this.yFactor;
	let brickT = t + y / (this.gameData.brick_speed() * this.yFactor);
	brickT += sixteenthNoteTime - (brickT % sixteenthNoteTime + sixteenthNoteTime) % sixteenthNoteTime; //subtract positive modulus
	let brickWidth = wasm.graphic_size(wasm.GraphicGroup.Brick).x * this.xFactor
	
	x = Math.floor(x / brickWidth); // !!! do calculation in game.rs to ensure consistency
	
	this.gameData.toggle_brick(0, brickT, x);
	
	// to have the game add the note. 
	// TODO, more robust way would be to add the note to on screen notes in toggle_brick, then just rerender with renderGame()
	this.seek(t); 
	
	// TODO only add/remove new/removed notes, don't reload whole song
	let json = JSON.parse(this.gameData.song_notes_json());
	let songTime = this.gameData.song_time();
	this.notes = [];
	this.notesIdx = 0;
	json.forEach( note => {
		this.notes.push(note.time);
		if(songTime > note.time){
			++this.notesIdx;
		}
	});
}

Editor.prototype.toGame = function(){
	if(this.isLoaded == false) {
		throw Error("game object has not been loaded");
	}
	
	// !!! rethink the relationship between editor and game. Seems flimsy to switch to game while retaining all game data.
		// Clear hack right now, switch to editor and back to rewind song
	Object.setPrototypeOf(this, Game.prototype);
	
	return this;
}
	