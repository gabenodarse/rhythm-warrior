import * as wasm from "../pkg/music_mercenary.js";
import {Game} from "./Game.js";

export function Editor () {
	Game.call(this);
}

Object.setPrototypeOf(Editor.prototype, Game.prototype);

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

Editor.prototype.toGame = function(){
	if(this.isLoaded == false) {
		throw Error("game object has not been loaded");
	}
	
	Object.setPrototypeOf(this, Game.prototype);
	
	return this;
}