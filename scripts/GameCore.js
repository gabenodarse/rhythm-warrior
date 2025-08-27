"use strict";

import * as wasm from "../pkg/rhythm_warrior.js";
import * as load from "./load.js";
import {Editor} from "./Editor.js";
import {Game} from "./Game.js";

export function GameCore () {
	// members
	this.gameObject;
	this.wasmMemoryObj;
	this.database; 
	this.songData;
	this.isLoaded;
	this.isSongLoaded;
	
	// audio
	this.audioContext;
	this.soundKeys;
	this.soundBuffers;
	this.audioSource;
	this.songBuffer;
	this.audioTimeSafetyBuffer;

	// set defaults
	this.isLoaded = false;
	this.isSongLoaded = false;

	this.audioTimeSafetyBuffer = 0.15;
}

GameCore.prototype.init = async function (wasmMemoryObj) {
	if(this.isLoaded){ return; }
	
	this.wasmMemoryObj = wasmMemoryObj;
	
	let loader = new load.Loader();
	
	this.audioContext = new AudioContext();
	
	let audioResourceLocations;
	await fetch("./audio-resources.json")
		.then(res => res.json())
		.then(res => { audioResourceLocations = res });
	
	// load sounds (not songs)
		// !!! move to loader
		// TODO error handling when it takes too long
	this.soundKeys = Object.keys(audioResourceLocations);
	this.soundLocations = Object.values(audioResourceLocations);
	let numAudioResources = this.soundKeys.length;
	this.soundBuffers = Object();
	for(let i = 0; i < numAudioResources; ++i){
		await fetch("./assets/sounds/" + this.soundLocations[i])
			.then(res => res.arrayBuffer())
			.then(res => this.audioContext.decodeAudioData(res))
			.then(res => this.soundBuffers[this.soundKeys[i]] = res)
			.catch(rej => { throw Error("Error loading audio: " + rej) });
	}
		
	// !!! can happen same time as graphics are loading
	this.database = await loader.loadDatabase();
	
	this.gameObject = wasm.Game.new();
    this.songData = {};
	
	this.isLoaded = true;
}

GameCore.prototype.getSongLoaded = function(){
	return this.isSongLoaded;
}

GameCore.prototype.getRenderingInstructions = function(){
	return this.gameObject.rendering_instructions();
}

GameCore.prototype.getScore = function(){
	return this.gameObject.game_data().score;
}

GameCore.prototype.getSongData = function(){
	return this.songData;
}

GameCore.prototype.getGameData = function(){
	return this.gameObject.game_data();
}

GameCore.prototype.getSongBuffer = function(){
	return this.songBuffer;
}

GameCore.prototype.songs = function(){
	let songs = this.database.searchSong();
	
	return songs;
}

// !!! confirmation on deleted data, this and other song modification methods
GameCore.prototype.newSong = function(name, artist, difficulty, bpm, brickSpeed, duration, songStartOffset, songFileName, jsonFileName){
	this.gameObject = wasm.Game.new(bpm, brickSpeed, duration);
	this.songData = {
		name: name,
		artist: artist,
		difficulty: difficulty,
		bpm: bpm, // !!! bpm brick speed and duration all also occur in the wasm Game object, consistency not guaranteed
		brickSpeed: brickSpeed,
		duration: duration,
		startOffset: songStartOffset,
		timeCreated: 0,
		timeModified: 0,
		filename: songFileName,
		jsonname: jsonFileName
	}
}

GameCore.prototype.modifySong = function(name, artist, difficulty, bpm, brickSpeed, duration, songStartOffset, songFileName, jsonFileName){
	let notes = this.gameObject.bricks();

	// !!! no need to create a whole new game object when modifying some metadata
	this.gameObject = wasm.Game.new(bpm, brickSpeed, duration);

	notes.forEach( note =>{
		let brickType = note.brick_type;
		let beatPos = note.beat_pos;
		let endBeatPos = note.end_beat_pos;
		let xPos = note.x_pos;
		let isTriplet = note.is_triplet;
		let isTrailing = note.is_trailing;
		let isLeading = note.is_leading;
		let isHoldNote = note.is_hold_note;
		this.gameObject.initial_load_add_brick(wasm.BrickData.new( 
			brickType, beatPos, endBeatPos, xPos, isTriplet, isTrailing, isLeading, isHoldNote)); 
	});
	
	this.gameObject.seek(0);
	
	if(!songFileName){
		songFileName = this.songData.filename;
	}
	
	this.songData = {
		name: name,
		artist: artist,
		difficulty: difficulty,
		bpm: bpm,
		brickSpeed: brickSpeed,
		duration: duration,
		startOffset: songStartOffset,
		timeCreated: 0,
		timeModified: 0,
		filename: songFileName,
		jsonname: jsonFileName
	}
}

GameCore.prototype.loadSong = async function(songData){
	// !!! creating a new game to load a new song? Or create a load_song method? wasm garbage collected?
	this.isSongLoaded = false;

	let songObject = this.database.loadSong(songData);
	
	this.gameObject = wasm.Game.new(songObject.bpm, songObject.brickSpeed, songObject.duration);
	
	songObject.notes.forEach( note => {
		let brickType = note[0];
		let beatPos = note[1];
		let endBeatPos = note[2];
		let xPos = note[3];
		let isTriplet = note[4];
		let isTrailing = note[5];
		let isLeading = note[6];
		let isHoldNote = note[7];
		this.gameObject.initial_load_add_brick(wasm.BrickData.new( 
			brickType, beatPos, endBeatPos, xPos, isTriplet, isTrailing, isLeading, isHoldNote)); 
	});
	
	this.gameObject.seek(0);
	
	this.songData = {
		name: songObject.name,
		artist: songObject.artist,
		difficulty: songObject.difficulty,
		bpm: songObject.bpm,
		brickSpeed: songObject.brickSpeed,
		duration: songObject.duration,
		startOffset: songObject.startOffset,
		timeCreated: songObject.timeCreated,
		timeModified: songObject.timeModified,
		filename: songObject.filename,
		jsonname: songObject.jsonname
	}

	// fetch the mp3
		// !!! add error handling 
	await fetch("./assets/music/" + this.songData.filename)
		.then(res => res.arrayBuffer())
		.then(res => this.audioContext.decodeAudioData(res))
		.then(res => { this.songBuffer = res; }
	);
	
	this.isSongLoaded = true;
}

GameCore.prototype.loadMP3 = async function(file){
	this.isSongLoaded = false;

	// !!! add error handling
	await file.arrayBuffer()
		.then(res => this.audioContext.decodeAudioData(res))
		.then(res => { this.songBuffer = res; }
	);
		
    this.songData.filename = file.name;
	this.isSongLoaded = true;
}

GameCore.prototype.userLoadSong = async function(songAudioFile, songJsonFile){
	this.isSongLoaded = false;

	let songObject;
	await songJsonFile.text()
		.then(res => songObject = JSON.parse(res));
	
	this.gameObject = wasm.Game.new(songObject.bpm, songObject.brickSpeed, songObject.duration);
	
	songObject.notes.forEach( note => {
		let brickType = note[0];
		let beatPos = note[1];
		let endBeatPos = note[2];
		let xPos = note[3];
		let isTriplet = note[4];
		let isTrailing = note[5];
		let isLeading = note[6];
		let isHoldNote = note[7];
		this.gameObject.initial_load_add_brick(wasm.BrickData.new( 
			brickType, beatPos, endBeatPos, xPos, isTriplet, isTrailing, isLeading, isHoldNote)); 
	});
	
	this.gameObject.seek(0);
	
	this.songData = {
		name: songObject.name,
		artist: songObject.artist,
		difficulty: songObject.difficulty,
		bpm: songObject.bpm,
		brickSpeed: songObject.brickSpeed,
		duration: songObject.duration,
		startOffset: songObject.startOffset,
		timeCreated: songObject.timeCreated,
		timeModified: songObject.timeModified,
		filename: songAudioFile.name,
		jsonname: songJsonFile.name
	}
	
	// !!! add error handling
	await songAudioFile.arrayBuffer()
		.then(res => this.audioContext.decodeAudioData(res))
		.then(res => { this.songBuffer = res; }
	);
	
	this.isSongLoaded = true;
}

GameCore.prototype.saveSong = function(songData){
	let notes = this.gameObject.bricks();
	
	notes.forEach( note => {
		note.approx_time = wasm.BrickData.approx_time(note.beat_pos, this.songData.bpm);
	});
	
	this.database.saveSong(songData, notes);
}

GameCore.prototype.toEditor = function(){
	if(this.isLoaded == false) {
		throw Error("game core has not been loaded");
	}
	if(this instanceof Editor){
		return this;
	}
	
	this.gameObject.seek(this.gameObject.game_data().time_running); 
	Object.setPrototypeOf(this, Editor.prototype);
	
	return this;
}

GameCore.prototype.toGame = function(){
	if(this.isLoaded == false) {
		throw Error("game core has not been loaded");
	}
	if(this instanceof Game){
		return this;
	}

    this.gameObject.seek(this.gameObject.game_data().time_running); 
	Object.setPrototypeOf(this, Game.prototype);
	
	return this;
}