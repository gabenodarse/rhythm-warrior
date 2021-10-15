
import {Game, Editor} from "./Game.js";

// !!! resizing resizes both overlay and screen div, prompt "your screen has been resized. OK to adjust"
	// resizing retains aspect ratio, attempts to size sidebar to accommodate
	
export function EventPropagator(){
	this.game;
	this.overlay;
	
	this.controls;
	
	this.isPaused;
	this.isEditor;
	
	this.gameLoop;
	this.editorLoop;
	this.loop;
	this.resumeGame;
	this.handleKeyDown;
	this.handleKeyUp;
	this.resize;
}

// !!! moving functions to prototype means more difficult removal of listeners? or should some/all of this go to prototype?
EventPropagator.prototype.init = function(game, overlay, controls){
	this.game = game;
	this.overlay = overlay;
	this.controls = controls;
	
	this.gameLoop = () => {
		if(this.isPaused) {
			this.pause();
		} else {
			this.game.tick();
			this.overlay.updateScore(this.game.score());
		}
		requestAnimationFrame(this.loop);
	}
	
	this.editorLoop = () => {
		if(this.isPaused) {
			this.pause();
		} else {
			this.game.tick();
			this.overlay.updateSongData(this.game.songData());
		}
		requestAnimationFrame(this.loop);
	}
	
	this.pauseLoop = () => {
		if(!this.isPaused) {
			this.start(); // starts a new loop
		} else {
			requestAnimationFrame(this.loop);
		}
	}
	
	this.handleKeyDown = evt => {
		// TODO faster handling of repeated key inputs from holding down a key?
		if (evt.keyCode === 27){
			this.overlay.handleEscape();
			
			// if the game is not in editor mode, pause/unpause
			if(!this.isEditor){
				this.isPaused = !this.isPaused;
			}
		}
		else if(typeof(this.controls[evt.keyCode]) === "number" && !this.isPaused){
			this.game.startControl(this.controls[evt.keyCode]);
		}
	}
	
	this.handleKeyUp = evt => {
		if(typeof(this.controls[evt.keyCode]) === "number" && !this.isPaused){
			this.game.stopControl(this.controls[evt.keyCode]);
		}
	}
	
	let resizeRefresher = true;
	this.resize = () => {
		const delay = 50; // minimum delay between screen size refreshes
		if(resizeRefresher){
			resizeRefresher = false;
			let game = this.game;
			setTimeout(async function(){
				resizeRefresher = true;
				game.resize();
			},delay);
		}
	}
	
	this.loop = this.gameLoop;
	this.resize();
	
	window.addEventListener("keydown", this.handleKeyDown);
	window.addEventListener("keyup", this.handleKeyUp);
	window.addEventListener("resize", this.resize);
}

EventPropagator.prototype.togglePlay = function(){
	this.isPaused = !this.isPaused;
}

// only called from asynchronous game/editor/pause loop. Sends the loop to the game as a callback, starting a new loop.
EventPropagator.prototype.start = function(){
	this.isPaused = false;
	if(this.isEditor){
		this.loop = this.editorLoop;
	} else {
		this.loop = this.gameLoop;
	}
	this.game.start(this.loop);
}

// only called from within the asynchronous game/editor/pause loop
EventPropagator.prototype.pause = function(){
	this.isPaused = true;
	// !!! handle game key states on pause/unpause (as of now fires key up events on pause)
	for(const key in this.controls) {
		let evt = new KeyboardEvent("keyup", {
			keyCode: key,
		});
		this.handleKeyUp(evt);
	}
	this.loop = this.pauseLoop;
	this.game.pause();
}

EventPropagator.prototype.enableEditor = function(){
	if(!this.isEditor){
		this.overlay.showElement("editorOverlay");
		this.overlay.hideElement("score");
		
		this.game = this.game.toEditor();
		
		this.overlay.updateSongData(this.game.songData());
		
		this.isEditor = true;
	}
}

EventPropagator.prototype.disableEditor = function(){
	if(this.isEditor){
		this.overlay.hideElement("editorOverlay");
		this.overlay.showElement("score");
		
		this.game = this.game.toGame();
		
		this.overlay.updateSongData(this.game.songData());
		
		this.isEditor = false;
	}
}

EventPropagator.prototype.restartSong = function(){
	this.game.restart();
}

EventPropagator.prototype.runOnGame = function(functionToRun, updateEditor){
	this.isPaused = true;
	
	let ret = functionToRun(this.game);
	
	this.game.renderGame();
	if(updateEditor === true) {
		this.overlay.updateSongData(this.game.songData());
	}
	
	return ret;
}

