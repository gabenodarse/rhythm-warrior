"use strict";

import {GameCore} from "./GameCore.js";

export function Game () {
	this.lastTick; // time since the game last ticked
}

Object.setPrototypeOf(Game.prototype, GameCore.prototype);

Game.prototype.start = function (callback) {
	if(!this.isSongLoaded){
		throw Error("Attempting to start a song before load");
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

Game.prototype.stopAudio = function(){
	this.audioSource.stop();
}

Game.prototype.restart = function(){
	this.stopAudio();
	this.gameObject.seek(0);
}

Game.prototype.tick = function(){
	let now = performance.now();
	// !!! handle if there's too long a time between ticks (pause game?)
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
}

Game.prototype.startControl = function(cntrl){
	let now = performance.now();
	this.gameObject.input_command(cntrl, (now - this.lastTick) / 1000);
}

Game.prototype.stopControl = function(cntrl){
	let now = performance.now();
	this.gameObject.stop_command(cntrl, (now - this.lastTick) / 1000);
}
