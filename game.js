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
	this.graphics; // !!! can be either canvases or webGL. Add way to choose between them.
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

Game.prototype.load = async function () {
	if(this.isLoaded){ return; }
	
	// >:< loader as member? (so different songs can be loaded without creating new loaders)
		// load songs separately from graphics
	let loader = new load.Loader();
	
	// TODO add error handling
	await loader.init()
		.then( () => loader.loadGraphics("webGL", this.screenDiv)) //canvases or webGL
		.then( res => this.graphics = res );
	
	// TODO add error handling
	await fetch("song.mp3")
		.then(res => res.arrayBuffer())
		.then(res => this.audioContext.decodeAudioData(res))
		.then(res => { this.audioBuffer = res; }
	);
	
	this.gameData = wasm.Game.new();
	
	let gameDim = wasm.game_dimensions();
	this.width = gameDim.x;
	this.height = gameDim.y;
	this.xFactor = 1;
	this.yFactor = 1;
	
	let songData = loader.getSong(2);
	songData[0]["values"].forEach( note => {
		this.gameData.toggle_brick(note[2], note[3], note[4]);
	});
	
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

Game.prototype.tick = function(){
	let now = new Date().getTime();
	// !!! render asynchronously to keep game ticking???
	// !!! handle if there's too long a time between ticks (pause game?)
	// !!! get fps, average, and log
	let timePassed = (now - this.lastTick) / 1000; // convert to seconds
	this.gameData.tick(timePassed); 
	this.lastTick = now;
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

Game.prototype.songData = function(){
	return {
		beatInterval: this.gameData.beat_interval(),
		brickSpeed: this.gameData.brick_speed(),
		songTime: this.gameData.song_time(),
		songDuration: this.gameData.song_duration()
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
	this.loadEditorComponent();
	
	return this;
}

export function Editor () {
	Game.call(this);
	
	this.onScreenClick;
}

Object.setPrototypeOf(Editor.prototype, Game.prototype);

Editor.prototype.seek = function(time){
	this.gameData.seek(time);
}

Editor.prototype.load = async function(){
	if(!this.isLoaded){
		await Game.prototype.load.call(this);
	}
	this.loadEditorComponent();
}

// >:< shall this be removed, store event listener in the overlay?
Editor.prototype.loadEditorComponent = function(){
	if(!this.onScreenClick){
		this.onScreenClick = evt => {
			let x = evt.clientX - this.screenDiv.offsetLeft;
			let y = evt.clientY - this.screenDiv.offsetTop;
			let t = this.gameData.song_time();
			this.createNote(x, y, t);
			this.gameData.seek(t); 
			this.renderGame();
		}
		
		this.screenDiv.addEventListener("click", this.onScreenClick); // TODO smarter way to add/remove this listener?
	}
}

Editor.prototype.createNote = function(x, y, t){
	// !!! support for third, sixth, twelfth notes
	let sixteenthNoteTime = this.gameData.beat_interval() / 4;
	
	y -= wasm.ground_pos() * this.yFactor;
	let brickT = t + y / (this.gameData.brick_speed() * this.yFactor);
	brickT += sixteenthNoteTime - (brickT % sixteenthNoteTime + sixteenthNoteTime) % sixteenthNoteTime; //subtract positive modulus
	let brickWidth = wasm.graphic_size(wasm.GraphicGroup.Brick).x * this.xFactor
	
	// >:< Because there are 32 notes, each x non overlapping. Magic number should be removed.
		// probably want to narrow it down to 24 notes. Store note positions as 0-23? (modify midi conversion)
	x = Math.floor(x / brickWidth) - 16;
	
	this.gameData.toggle_brick(0, brickT, x);
	this.seek(t); 
	this.renderGame();	
}

Editor.prototype.toGame = function(){
	if(this.isLoaded == false) {
		throw Error("game object has not been loaded");
	}
	
	this.screenDiv.removeEventListener("click", this.onScreenClick);
	this.onScreenClick = undefined;
	this.gameData.seek(0); // !!! rethink the relationship between editor and game. Seems flimsy.
	Object.setPrototypeOf(this, Game.prototype);
	
	return this;
}
	