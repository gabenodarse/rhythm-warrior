
import {Game, Editor} from "./Game.js";

let g_menuKeyPresses = []
// !!! resizing resizes both overlay and screen div, prompt "your screen has been resized. OK to adjust"
	// resizing retains aspect ratio, attempts to size sidebar to accommodate
	
export function EventPropagator(){
	this.game;
	this.overlay;
	this.controls;
	
	this.resumeEvents; // events to fire once the game resumes
	
	this.isRunning;
	this.isEditor;
	this.stopFlag;
	
	this.loop;
	this.resize;
}

// !!! moving functions to prototype means more difficult removal of listeners? or should some/all of this go to prototype?
EventPropagator.prototype.init = function(game, overlay, controls){
	this.game = game;
	this.overlay = overlay;
	this.controls = controls;
	
	this.resumeEvents = [];
	
	let resizeRefresher = true;
	let resize = () => {
		if(resizeRefresher){
			resizeRefresher = false;
			requestAnimationFrame(() => {
				resizeRefresher = true;
				this.game.resize();
			});
		}
	}
	
	this.loop = () => this.gameLoop();
	resize();
	
	this.stopFlag = false;
	this.isRunning = false;
	this.isEditor = false;
	
	document.addEventListener("keydown", evt => {	this.handleKeyDown(evt) });
	document.addEventListener("keyup", evt => { this.handleKeyUp(evt) });
	document.addEventListener("resize", resize);
}

EventPropagator.prototype.togglePlay = function(){
	if(this.isRunning){
		this.stopFlag = true;
	} else {
		this.stopFlag = false;
		this.start();
	}
}

EventPropagator.prototype.start = function(){
	this.overlay.hideElement("menu");
	this.overlay.hideElement("homeScreen");
	this.overlay.showElement("score");
	
	if(this.isRunning){
		throw Error("Attempting to start game when the game is already running");
	}
	
	this.startLoop();
}

EventPropagator.prototype.pause = function(){
	this.overlay.showElement("menu");
	this.overlay.populateMenu("gameMenu");
	this.overlay.hideElement("score");
	this.stopFlag = true;
}

EventPropagator.prototype.restartSong = function(){
	this.game.restart(); // !!! should synchronize with game loop or no? Probably
}

EventPropagator.prototype.exitToHomeScreen = function(){
	this.stopFlag = true;
	this.overlay.hideElement("menu");
	this.overlay.hideElement("score");
	this.overlay.hideElement("editorOverlay");
	this.overlay.hideElement("endGameScreen");
	this.overlay.showElement("homeScreen");
}

EventPropagator.prototype.enableEditor = function(){
	if(!this.isEditor){
		this.overlay.showElement("editorOverlay");
		this.overlay.hideElement("score");
		
		this.game = this.game.toEditor();
		
		this.overlay.updateSongData(this.game.getSongData());
		
		this.loop = () => this.editorLoop();
		this.isEditor = true;
	}
}

EventPropagator.prototype.disableEditor = function(){
	if(this.isEditor){
		this.overlay.hideElement("editorOverlay");
		this.overlay.showElement("score");
		
		this.game = this.game.toGame();
		
		this.overlay.updateSongData(this.game.getSongData());
		
		this.loop = () => this.gameLoop();
		this.isEditor = false;
	}
}

EventPropagator.prototype.runOnGame = function(functionToRun, updateEditor){
	this.stopFlag = true;
	
	let ret = functionToRun(this.game);
	
	this.game.renderGame();
	if(updateEditor === true) {
		this.overlay.updateSongData(this.game.getSongData());
	}
	
	return ret;
}

EventPropagator.prototype.handleKeyUp = function(evt){
	if(typeof(this.controls[evt.keyCode]) === "number"){
		if(this.isRunning){
			this.game.stopControl(this.controls[evt.keyCode]);
		} else {
			let controlID = this.controls[evt.keyCode];
			this.resumeEvents[ controlID ] = new KeyboardEvent("keyup", { keyCode: evt.keyCode, });
		}
	}
}

EventPropagator.prototype.handleKeyDown = function(evt){
	// branching if statements to handle events based on state of game and menus
	if(typeof(this.controls[evt.keyCode]) === "number"){
		if(this.isRunning){
			this.game.startControl(this.controls[evt.keyCode]);
		} else {
			let controlID = this.controls[evt.keyCode];
			this.resumeEvents[ controlID ] = null;			
		}
	}
	
	if(evt.keyCode === 27){ // escape key
		// if the game is not in editor mode, pause/unpause
		if(!this.overlay.isElementShowing("homeScreen") && this.overlay.isElementShowing("menu")){
			this.overlay.hideElement("menu");
			if(!this.isRunning && !this.isEditor){
				this.start();
			}
		} else if(!this.overlay.isElementShowing("homeScreen") && !this.overlay.isElementShowing("menu")){
			this.overlay.showElement("menu");
			this.overlay.populateMenu("gameMenu");
			if(this.isRunning && !this.isEditor){
				this.pause();
			}
		} else if(this.overlay.isElementShowing("homeScreen") && this.overlay.isElementShowing("menu")){
			this.overlay.hideElement("menu");
		} else if(this.overlay.isElementShowing("homeScreen") && !this.overlay.isElementShowing("menu")){
			this.overlay.showElement("menu");
			this.overlay.populateMenu("homeMenu");
		}
	} else if(this.overlay.isElementShowing("menu")){
		this.overlay.passEvent("menu", evt);
		
		// master menu
		g_menuKeyPresses.unshift(evt);
		if(g_menuKeyPresses.length > 10){
			g_menuKeyPresses.pop();
		}
		if( g_menuKeyPresses.length >= 6 && !this.overlay.isElementShowing("homeScreen")
		&& g_menuKeyPresses[5].keyCode == 77 && g_menuKeyPresses[4].keyCode == 65  && g_menuKeyPresses[3].keyCode == 83 
		&& g_menuKeyPresses[2].keyCode == 84 && g_menuKeyPresses[1].keyCode == 69 && g_menuKeyPresses[0].keyCode == 82 ){
			this.overlay.populateMenu("masterGameMenu");
		}
	} else if(this.overlay.isElementShowing("homeScreen")){
		this.overlay.passEvent("homeScreen", evt);
	} else if(this.overlay.isElementShowing("endGameScreen")){
		this.overlay.passEvent("endGameScreen", evt);
	} else if(this.overlay.isElementShowing("editorOverlay")){
		this.overlay.passEvent("editorOverlay", evt);
	}
}

EventPropagator.prototype.gameLoop = function(){
	if(this.stopFlag){
		this.stopLoop();
	} else {
		this.game.tick();
		
		let songData = this.game.getSongData();
		
		this.overlay.updateScore(songData.gameData.score);
		
		if(songData.gameData.time_running > 60){
			this.overlay.showElement("endGameScreen");
			this.stopLoop();
		}
		
		requestAnimationFrame(this.loop);
	}
}

EventPropagator.prototype.editorLoop = function(){
	if(this.stopFlag) {
		this.stopLoop();
	} else {
		this.game.tick();
		this.overlay.updateSongData(this.game.getSongData());
		requestAnimationFrame(this.loop);
	}
}

// sends the loop to the game as a callback, starting a new loop.
EventPropagator.prototype.startLoop = function(){
	this.isRunning = true;
	this.stopFlag = false;
	
	for(let evt of this.resumeEvents) {
		if(evt != null){
			document.dispatchEvent(evt);
		}
	}
	
	this.game.start(this.loop);
}

// only called from within the asynchronous game/editor loop
EventPropagator.prototype.stopLoop = function(){
	this.isRunning = false;
	this.stopFlag = false;
	
	this.game.pause();
}