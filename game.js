"use strict";

// >:< can make editor and game separate types which inherit from Game Data type
import * as wasm from "./pkg/music_mercenary.js";
import * as load from "./load.js";

export function Game () {
	// !!! resizing based on screen size + options
	this.gameCanvas = document.createElement('canvas');
	this.gameContext = this.gameCanvas.getContext('2d');
	this.xFactor = 1;
	this.yFactor = 1;
	this.gameCanvas.width = 1100; 
	this.gameCanvas.height = 600;
	document.body.appendChild(this.gameCanvas); // !!! determine position

	this.lastTick;
	this.gameData;
	this.canvases;
	this.audioContext = new AudioContext();
	this.audioSource;
	this.audioBuffer;
	this.audioTimeSafetyBuffer = 0.15;
}

Game.prototype.load = async function () {
	let loader = new load.Loader();
	
	// TODO add error handling
	await wasm.default();
	
	await loader.init()
		.then( () => loader.loadGraphics(wasm))
		.then( res => this.canvases = res );
	
	// TODO add error handling
	await fetch("song.mp3")
		.then(res => res.arrayBuffer())
		.then(res => this.audioContext.decodeAudioData(res))
		.then(res => { this.audioBuffer = res; }
	);
	
	this.gameData = wasm.Game.new();
	
	let gameDim = wasm.game_dimensions();
	this.xFactor = this.gameCanvas.width / gameDim.x;
	this.yFactor = this.gameCanvas.height / gameDim.y;
	
	let songData = loader.getSong(2);
	songData[0]["values"].forEach( note => {
		this.gameData.load_brick(note[2], note[3], note[4]);
	});
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
	
	this.renderAll();
}

Game.prototype.startControl = function(cntrl){
	let now = new Date().getTime();
	this.gameData.input_command(cntrl, (now - this.lastTick) / 1000);
}

Game.prototype.stopControl = function(cntrl){
	let now = new Date().getTime();
	this.gameData.stop_command(cntrl, (now - this.lastTick) / 1000);
}

Game.prototype.showEditor = function(){
	let dims = wasm.game_dimensions();
	let screenWidth = dims.x;
	let screenHeight = dims.y;
	
	let beatInterval = this.gameData.beat_interval();
	let brickSpeed = this.gameData.brick_speed();
	let y = 0;
	let t = ( ((-this.gameData.song_time()) % beatInterval) + beatInterval ) % beatInterval; // positive modulus
	let yOffset = this.gameData.ground_pos();
	
	while(true){
		if(t * brickSpeed + yOffset < screenHeight){
			this.gameContext.fillRect(0, (t * brickSpeed + yOffset) * this.yFactor, screenWidth * this.xFactor, 3);
			t += beatInterval / 2;
			this.gameContext.fillRect(0, (t * brickSpeed + yOffset) * this.yFactor, screenWidth * this.xFactor, 1);
			t += beatInterval / 2;
			continue;
		}
		break;
	}
}

Game.prototype.renderAll = function(){
	let instructions = this.gameData.rendering_instructions();
	
	// TODO error handling: check if instructions is an array of PositionedGraphic objects
	instructions.forEach( instruction => {
		this.gameContext.drawImage(this.canvases[instruction.g],instruction.x * this.xFactor,instruction.y * this.yFactor); 
	});
}

Game.prototype.seek = function(time){
	this.gameData.seek(time);
}

