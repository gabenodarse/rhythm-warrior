
import {Game, Editor} from "./Game.js";

// !!! resizing resizes both overlay and screen div, prompt "your screen has been resized. OK to adjust"
	// resizing retains aspect ratio, attempts to size sidebar to accommodate
	
export function EventPropagator(){
	this.game;
	this.overlay;
	
	this.controls;
	
	this.paused;
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
		if(this.paused) {
			return;
		}
		
		this.game.tick();
		this.overlay.updateScore(this.game.score());
		requestAnimationFrame(this.loop); // !!! set timeout or request animation frame better?
	}
	
	this.editorLoop = () => {
		if(this.paused) {
			return;
		}
		
		this.game.tick();
		this.overlay.updateSongData(this.game.songData());
		requestAnimationFrame(this.loop);
	}
	
	this.handleKeyDown = evt => {
		// TODO faster handling of repeated key inputs from holding down a key?
		if (evt.keyCode === 27){
			this.overlay.toggleElement("menu");
			
			// !!! doesn't work as intended if menu is open and song is playing (via editor before switching back to game)
			if(!(this.game instanceof Editor)){
				if(this.paused){
					this.start();
				}
				else{
					this.pause();
				}
			}
		}
		else if(typeof(this.controls[event.keyCode]) === "number" && !this.paused){
			this.game.startControl(this.controls[event.keyCode]);
		}
	}
	
	this.handleKeyUp = evt => {
		if(typeof(this.controls[event.keyCode]) === "number" && !this.paused){
			this.game.stopControl(this.controls[event.keyCode]);
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
	if(this.paused){
		this.start();
	}
	else{
		this.pause();
	}
}

EventPropagator.prototype.start = function(){
	this.paused = false;
	this.game.start(this.loop);
}

EventPropagator.prototype.pause = function(){
	// !!! handle game key states on pause/unpause (as of now fires key up events on pause)
	for(const key in this.controls) {
		let evt = new KeyboardEvent("keyup", {
			keyCode: key,
		});
		this.handleKeyUp(evt);
	}
	this.paused = true;
	this.game.pause();
}

EventPropagator.prototype.enableEditor = function(){
	this.overlay.toggleElement("editorOverlay");
	this.overlay.toggleElement("score");
	this.game = this.game.toEditor();
	this.loop = this.editorLoop;
	this.overlay.updateSongData(this.game.songData());
}
// !!! make screwing with game through UI impossible. Through hacking IDC. 
	// Distinction from game and editor (going to game from editor starts at 0?)
// >:< pausing game with esc, enabling editor, playing then pausing, then disabling editor resumes the game
EventPropagator.prototype.disableEditor = function(){
	this.overlay.toggleElement("editorOverlay");
	this.overlay.toggleElement("score");
	this.game = this.game.toGame();
	this.loop = this.gameLoop;
}

EventPropagator.prototype.restartSong = function(){
	this.game.restart();
}

EventPropagator.prototype.runOnGame = function(functionToRun, updateEditor){
	if(this.paused == false){
		this.pause();
	}
	
	let ret = functionToRun(this.game);
	if(updateEditor === true) this.overlay.updateSongData(this.game.songData());
	return ret;
}

