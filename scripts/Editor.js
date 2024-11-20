import * as wasm from "../pkg/music_mercenary.js";
import {GameCore} from "./GameCore.js";

export function Editor () {
	this.lastTick; // time since the game last ticked
}

Object.setPrototypeOf(Editor.prototype, GameCore.prototype);

Editor.prototype.start = function (callback) {
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

Editor.prototype.stopAudio = function(){
	this.audioSource.stop();
}

Editor.prototype.restart = function(){
	this.stopAudio();
	this.gameObject.seek(0);
}

Editor.prototype.tick = function(){
	let now = performance.now();
    
	let timePassed = (now - this.lastTick) / 1000; // convert to seconds
	this.lastTick = now;
	
	this.gameObject.seek(this.songData.gameData.time_running + timePassed); 
	this.songData.gameData = this.gameObject.game_data();
}

Editor.prototype.seek = function(time){
	this.gameObject.seek(time);
	this.songData.gameData = this.gameObject.game_data();
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

// !!! does not account for is_trailing is_leading or is_triplet. Ambiguities in the brick selected can lead to bugs.
Editor.prototype.selectBrick = function(beatPos, xPos){
	return this.gameObject.select_brick(beatPos, xPos);
}