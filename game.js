"use strict";

import * as wasm from "./pkg/music_mercenary.js";
import * as load from "./load.js";

export function Game () {
	this.width;
	this.height;
	this.xFactor;
	this.yFactor;

	this.lastTick;
	this.gameData;
	this.graphics; // !!! can be either canvases or webGL. Add way to choose between them.
	this.audioContext = new AudioContext();
	this.audioSource;
	this.audioBuffer;
	this.audioTimeSafetyBuffer = 0.15;
}

Game.prototype.load = async function () {
	let loader = new load.Loader(); // >:< loader as member? (so different songs can be loaded without creating new loaders)
	
	// TODO add error handling
	await loader.init()
		.then( () => loader.loadGraphics("canvases"))
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
}

Game.prototype.resize = function(width, height){
	let gameDim = wasm.game_dimensions();
	this.width = width;
	this.height = height;
	this.xFactor = width / gameDim.x;
	this.yFactor = height / gameDim.y;
	this.graphics.resize(this.xFactor, this.yFactor);
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
	let now = new Date().getTime();
	let switchTime = this.audioContext.currentTime + this.audioTimeSafetyBuffer;
	this.audioSource.stop(switchTime);
	let timePassed = (now - this.lastTick) / 1000; // convert to seconds
	this.gameData.tick(timePassed + this.audioTimeSafetyBuffer);
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



export function Editor () {
	Game.call(this);
	
	this.scroller = document.querySelector("#scroller input");
	this.editorOverlay = document.querySelector("#editor-overlay");

	if(!this.scroller){
		throw Error("no scroller found");
	}
	if(!this.editorOverlay){
		throw Error("no editor overlay found");
	}
}

Object.setPrototypeOf(Editor.prototype, Game.prototype);

Editor.prototype.renderEditor = function(){
	// render game components
	this.renderGame();
	
	let dims = wasm.game_dimensions();
	let screenWidth = dims.x;
	let screenHeight = dims.y;
	let ctx = this.editorOverlay.getContext("2d");
	ctx.clearRect(0, 0, screenWidth * this.xFactor, screenHeight * this.yFactor);
	
	let beatInterval = this.gameData.beat_interval();
	let brickSpeed = this.gameData.brick_speed();
	let y = 0;
	let t = ( ((-this.gameData.song_time()) % beatInterval) + beatInterval ) % beatInterval; // positive modulus
	let yOffset = this.gameData.ground_pos();
	
	while(true){
		if(t * brickSpeed + yOffset < screenHeight){
			ctx.fillRect(0, (t * brickSpeed + yOffset) * this.yFactor, screenWidth * this.xFactor, 3);
			t += beatInterval / 2;
			ctx.fillRect(0, (t * brickSpeed + yOffset) * this.yFactor, screenWidth * this.xFactor, 1);
			t += beatInterval / 2;
			continue;
		}
		break;
	}
}

Editor.prototype.seek = function(time){
	this.gameData.seek(time);
}

Editor.prototype.load = async function(){
	await Game.prototype.load.call(this);
	
	this.scroller.value = this.gameData.song_time();
	this.scroller.max = this.gameData.song_duration();
	this.scroller.step = this.gameData.beat_interval() * 2;
	this.scroller.addEventListener("input", evt => {
		let t = parseInt(this.scroller.value);
		this.seek(t);
		this.renderEditor();
	});
	
	this.editorOverlay.width = this.width;
	this.editorOverlay.height = this.height;
	this.editorOverlay.lastClickTime = 0;
	this.editorOverlay.addEventListener("click", evt => {
		let now = new Date().getTime();
		if(now - this.editorOverlay.lastClickTime < 500){
			let x = evt.clientX - this.editorOverlay.offsetLeft;
			let y = evt.clientY - this.editorOverlay.offsetTop;
			let t = this.gameData.song_time();
			this.createNote(x, y, t);
			this.gameData.seek(t);
			this.renderEditor();
		}
		
		this.editorOverlay.lastClickTime = now;
	});
	
}

Editor.prototype.resize = function(width, height){
	Game.prototype.resize.call(this, width, height);
	this.renderEditor();
}

Editor.prototype.createNote = function(x, y, t){
	// !!! support for third, sixth, twelfth notes
	let sixteenthNoteTime = this.gameData.beat_interval() / 4;
	
	y -= this.gameData.ground_pos() * this.yFactor;
	t += y / (this.gameData.brick_speed() * this.yFactor);
	t += sixteenthNoteTime - (t % sixteenthNoteTime + sixteenthNoteTime) % sixteenthNoteTime; //subtract positive modulus
	let brickWidth = wasm.graphic_size(wasm.GraphicGroup.Brick).x * this.xFactor
	
	// >:< Because there are 32 notes, each x non overlapping. Magic number should be removed.
		// probably want to narrow it down to 24 notes. Store note positions as 0-23? (modify midi conversion)
	x = Math.floor(x / brickWidth) - 16;
	
	this.gameData.toggle_brick(0, t, x);	
}
	
	
	