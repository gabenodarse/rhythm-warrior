
import {Game, Editor} from "./Game.js";

// !!! resizing resizes both overlay and screen div, prompt "your screen has been resized. OK to adjust"
	// resizing retains aspect ratio, attempts to size sidebar to accommodate
	
export function EventPropagator(){
	this.game;
	this.overlay;
	this.controls;
	
	this.resumeEvents; // events to fire once the game resumes
	
	this.isRunning;
	this.stopFlag;
	this.isPreRendered; // boolean describing whether the game state is ready to be re-rendered
	this.resizeRefresher; // boolean to ensure a maximum of one resize event is handled per frame

	this.resize; // function to fire on resize events
	
	// fps timer
	this.lastFrame; // time of the last frame
	this.numFramesPerFPS; // number of frames to draw between each fps calculation
	this.frameTimes; // array of frame times for fps calculation
	this.frameCounter; // counts number of frames since last fps calculation
	this.minFrameTracker; // minimum frame time in this fps calculation
	this.maxFrameTracker; // maximum frame time in this fps calculation
	this.minFrameTime;
	this.maxFrameTime;
	this.fps;

	this.loop;
}

EventPropagator.prototype.init = function(game, overlay, controls){
	this.game = game;
	this.overlay = overlay;
	this.controls = controls;
	
	this.loop = () => this.gameLoop();
	this.resumeEvents = [];
	this.stopFlag = false;
	this.isRunning = false;

	this.numFramesPerFPS = 30; 
	this.frameTimes = new Array(this.numFramesPerFPS); 
	this.frameCounter = 0; 
	this.minFrameTracker = 1000;
	this.maxFrameTracker = 0;
	this.minFrameTime = 0;
	this.maxFrameTime = 0;
	this.fps = 0;

	this.resizeRefresher = true;
	
	window.addEventListener("keydown", evt => { this.handleEvent(evt) });
	window.addEventListener("keyup", evt => { this.handleEvent(evt) });
	window.addEventListener("mousedown", evt => {this.handleEvent(evt)});
	window.addEventListener("mousemove", evt => {this.handleEvent(evt)});
	window.addEventListener("mouseup", evt => {this.handleEvent(evt)});
	window.addEventListener("click", evt => {this.handleEvent(evt)});
	window.addEventListener("wheel", evt => {this.handleEvent(evt)});
	window.addEventListener("input", evt => { this.handleEvent(evt) });
	window.addEventListener("resize", this.handleResize );
	window.addEventListener("gameRender", evt => { this.handleGameRender(evt) });

	this.handleResize();
}

EventPropagator.prototype.togglePlay = function(){
	if(this.isRunning){
		this.stopFlag = true;
	} else {
		this.stopFlag = false;
		this.startLoop();
	}
}

EventPropagator.prototype.handleResize = function(evt){
	if(this.resizeRefresher){
		this.resizeRefresher = false;
		requestAnimationFrame(() => {
			this.game.resize();
			this.resizeRefresher = true;
		});
	}
}

EventPropagator.prototype.handleEvent = function(evt){
	if(this.overlay.isCapturing()){
		let instruction = this.overlay.passEvent(evt);
		this.runInstruction(instruction);

		if(evt.type == "keydown" && typeof(this.controls[evt.keyCode]) === "number"){
			let controlID = this.controls[evt.keyCode];
			this.resumeEvents[ controlID ] = null;			
		}
		else if(evt.type == "keyup" && typeof(this.controls[evt.keyCode]) === "number") {
			let controlID = this.controls[evt.keyCode];
			this.resumeEvents[ controlID ] = new KeyboardEvent("keyup", { keyCode: evt.keyCode, });
		}


		return;
	}
	
	if(evt.type == "keydown"){
		this.handleGameKeyDown(evt);
		return;
	}

	if(evt.type === "keyup"){
		this.handleGameKeyUp(evt);
		return;
	}
	
}

EventPropagator.prototype.handleGameKeyDown = function(evt){
	if(evt.keyCode === 27){ // escape key
		if(this.isRunning){
			this.stopFlag = true;
		}
		this.overlay.openGameMenu();
	} 

	if(typeof(this.controls[evt.keyCode]) === "number" && this.isRunning){
		this.game.startControl(this.controls[evt.keyCode]);
	}
}

EventPropagator.prototype.handleGameKeyUp = function(evt){
	if(typeof(this.controls[evt.keyCode]) === "number" && this.isRunning){
		this.game.stopControl(this.controls[evt.keyCode]);
	}
}

EventPropagator.prototype.runInstruction = function(instruction){
	if(!instruction) return;
	if(instruction == "toggle-play"){
		this.togglePlay();
	} else if(instruction == "restart-song"){
		this.game.restart(); // !!! should synchronize with game loop and pause game
	} else if(instruction == "stop-loop"){
		this.stopFlag = true;
	} else if(instruction == "start-loop"){
		this.startLoop();
	} else {
		throw Error("EventPropagator.runInstruction, no instruction: " + instruction);
	}
}

EventPropagator.prototype.handleGameRender = function(evt){
	if(this.isPreRendered == false){
		this.game.preRender();
		this.isPreRendered = true;
	}
}

EventPropagator.prototype.gameLoop = function(){
	let songData = this.game.getSongData();
	if(this.stopFlag){
		this.stopLoop();
		return;
	} else if(songData.gameData.time_running > 60) {
		this.stopLoop();
		this.overlay.goToEndGameScreen();
		return;
	} else {
		this.game.tick();
		
		// TODO reduce fps info to just fps
		let score = this.game.getScore();
		let tickTime = this.game.getTickTime();
		let preRenderTime = this.game.getPreRenderTime();
		let fps = "FPS: " + this.fps + "<br>MIN FRAME TIME (ms): " + this.minFrameTime + "<br>MAX FRAME TIME(ms): " + this.maxFrameTime
			+ "<br>TICK TIME AVERAGE(ms): " + tickTime.average + "<br>MIN: " + tickTime.min + "<br>MAX: " + tickTime.max
			+ "<br>PRE-RENDER TIME AVERAGE(ms): " + preRenderTime.average + "<br>MIN: " + preRenderTime.min + "<br>MAX: " + preRenderTime.max;
		
		this.overlay.update(fps);

		requestAnimationFrame(() => {
			// fps tracking
			let now = performance.now();
			let timePassed = now - this.lastFrame;
			this.lastFrame = now;
			this.frameTimes[this.frameCounter] = now; 
			this.frameCounter += 1;
			if(timePassed < this.minFrameTracker){
				this.minFrameTracker = timePassed;
			}
			if(timePassed > this.maxFrameTracker){
				this.maxFrameTracker = timePassed;
			}
			if(this.frameCounter == this.numFramesPerFPS){
				let averageFrameTime = (this.frameTimes[this.numFramesPerFPS - 1] - this.frameTimes[0]) / this.numFramesPerFPS;
				averageFrameTime = averageFrameTime / 1000; // convert to seconds
				this.fps = 1 / averageFrameTime;
				this.minFrameTime = this.minFrameTracker;
				this.maxFrameTime = this.maxFrameTracker;

				this.minFrameTracker = 1000;
				this.maxFrameTracker = 0;
				this.frameCounter = 0;
			}

			// if the game is pre-rendered, dispatch an event saying that the render occurred (triggering another pre-render)
			// (instead of prerendering before the repaint, prerender when the gameRender event or some other event triggers one)
			if(this.isPreRendered){
				this.isPreRendered = false;
				let evt = new Event("gameRender");
				window.dispatchEvent( evt );
			}
			this.loop();
		});
	}
}

// sends the loop to the game as a callback, starting a new loop.
EventPropagator.prototype.startLoop = function(){
	if(this.isRunning){
		throw Error("Attempting to start game when the game is already running");
	}

	this.isRunning = true;
	this.stopFlag = false;
	
	for(let evt of this.resumeEvents) {
		if(evt != null){
			document.dispatchEvent(evt);
		}
	}
	
	// reset fps tracker variables
	this.lastFrame = performance.now();
	this.frameTimes = new Array(this.numFramesPerFPS);
	this.frameCounter = 0;
	this.minFrameTracker = 1000;
	this.maxFrameTracker = 0;

	this.game.preRender();
	this.isPreRendered = true;

	this.game.start(this.loop);
}

// only called from within the asynchronous game/editor loop
EventPropagator.prototype.stopLoop = function(){
	this.isRunning = false;
	this.stopFlag = false;
	
	this.game.pause();
}