"use strict";

import * as load from "./load.js";
import * as wasm from "../pkg/music_mercenary.js";

// !!! resizing resizes both overlay and screen div, prompt "your screen has been resized. OK to adjust"
	// resizing retains aspect ratio, attempts to size sidebar to accommodate
	
export function EventPropagator(){
	this.game;
	this.overlay;
	this.controls;
	this.graphics;
	
	// screen where everything is added
	this.screenDiv;
	this.width;
	this.height;
	this.xFactor;
	this.yFactor;

	this.resumeEvents; // events to fire once the game resumes
	
	this.isRunning;
	this.stopFlag;
	this.isPreRendered; // boolean describing whether the game state is ready to be re-rendered
	this.isBlocking; // boolean describing whether events are being blocked
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

	// tick timer
	this.numTicksPerTickCalc; // number of ticks for each average calculation
	this.tickCounter; // counts number of ticks since last average calculation
	this.tickTotalTime; // total tick time for the average calculation
	this.minTickTracker; // minimum tick time in this average time calculation
	this.maxTickTracker; // maximum tick time in this average time calculation
	this.minTickTime;
	this.maxTickTime;
	this.averageTickTime;

	// pre-render timer
	this.numPreRenders; // number of pre-renders for each average calculation
	this.preRenderCounter; // counts number of pre-renders since last average calculation
	this.preRenderTotalTime; // total pre-render time for the average calculation
	this.minPreRenderTracker; // minimum pre-render time in this average time calculation
	this.maxPreRenderTracker; // maximum pre-render time in this average time calculation
	this.minPreRenderTime;
	this.maxPreRenderTime;
	this.averagePreRenderTime;

	this.loop;
}

EventPropagator.prototype.init = async function(game, overlay, controls){
	this.game = game;
	this.overlay = overlay;
	this.controls = controls;
	
	this.loop = () => this.gameLoop();
	this.resumeEvents = [];
	this.stopFlag = false;
	this.isRunning = false;
	this.isBlocking = false;
	this.resizeRefresher = true;

	// fps
	this.numFramesPerFPS = 30; 
	this.frameTimes = new Array(this.numFramesPerFPS); 
	this.frameCounter = 0; 
	this.minFrameTracker = 1000;
	this.maxFrameTracker = 0;
	this.minFrameTime = 0;
	this.maxFrameTime = 0;
	this.fps = 0;

	// tick timer
	this.numTicksPerTickCalc = 30;
	this.tickCounter = 0;
	this.tickTotalTime = 0;
	this.minTickTracker = 1000;
	this.maxTickTracker = 0;
	this.minTickTime = 0;
	this.maxTickTime = 0;
	this.averageTickTime = 0;

	// pre-render timer
	this.numPreRenders = 30;
	this.preRenderCounter = 0;
	this.preRenderTotalTime = 0;
	this.minPreRenderTracker = 1000;
	this.maxPreRenderTracker = 0;
	this.minPreRenderTime = 0;
	this.maxPreRenderTime = 0;
	this.averagePreRenderTime = 0;

	// screen div
	let gameDim = wasm.game_dimensions();
	this.screenDiv = document.getElementById("screen");
	this.width = this.screenDiv.clientWidth;
	this.height = this.screenDiv.clientHeight;
	this.xFactor = this.width / gameDim.x;
	this.yFactor = this.height / gameDim.y;

	// initialize loader and load graphics
	// TODO error handling when it takes too long
	let loader = new load.Loader();
	await loader.init()
		.then( () => loader.loadGraphics("canvases", this.screenDiv)) // !!! !!! !!! canvases or webGL. Make just webGL
		.then( res => this.graphics = res )
		.catch( rej => { throw Error("Error initializing loader / loading assets: " + rej ) });
	
	window.addEventListener("keydown", evt => { this.handleEvent(evt) });
	window.addEventListener("keyup", evt => { this.handleEvent(evt) });
	window.addEventListener("mousedown", evt => {this.handleEvent(evt)});
	window.addEventListener("mousemove", evt => {this.handleEvent(evt)});
	window.addEventListener("mouseup", evt => {this.handleEvent(evt)});
	window.addEventListener("click", evt => {this.handleEvent(evt)});
	window.addEventListener("wheel", evt => {this.handleEvent(evt)});
	window.addEventListener("input", evt => { this.handleEvent(evt) });
	window.addEventListener("resize", () => {this.handleResize()});
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
		let startTickTime = performance.now();
		this.game.tick();
		let endTickTime = performance.now();
		this.trackTickTime(endTickTime - startTickTime);
		
		this.overlay.update({fps: this.fps, xFactor: this.xFactor, yFactor: this.yFactor});

		requestAnimationFrame(() => {
			// fps tracking
			this.trackFPS();

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
	
	// !!! !!! !!! reset fps tracker variables
	this.lastFrame = performance.now();
	this.frameTimes = new Array(this.numFramesPerFPS);
	this.frameCounter = 0;
	this.minFrameTracker = 1000;
	this.maxFrameTracker = 0;

	this.preRender();
	this.isPreRendered = true;

	this.game.start(this.loop);
}

// only called from within the asynchronous game/editor loop
EventPropagator.prototype.stopLoop = function(){
	this.isRunning = false;
	this.stopFlag = false;
	
	this.game.stopAudio();
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
	} else if(instruction == "start-from-homescreen"){
		this.startFromHomescreen();
	} else if (instruction == "wait-song-load"){
		this.waitSongLoad();
	} else if (instruction == "pre-render"){
		this.overlay.update({fps: this.fps, xFactor: this.xFactor, yFactor: this.yFactor});
	}
	else {
		throw Error("EventPropagator.runInstruction, no instruction: " + instruction);
	}

	this.preRender();
	this.isPreRendered = true;
}

EventPropagator.prototype.startFromHomescreen = function(){
	this.isBlocking = true;
	
	if(this.game.getSongLoaded()){
		this.isBlocking = false;
		this.startLoop();
		return;
	}

	else{
		requestAnimationFrame(() => {this.startFromHomescreen()});
		return;
	}
}

EventPropagator.prototype.waitSongLoad = function(){
	this.isBlocking = true;

	if(this.game.getSongLoaded()){
		this.isBlocking = false;
		return;
	}

	else{
		requestAnimationFrame(() => {this.waitSongLoad()});
		return;
	}
}

EventPropagator.prototype.preRender = function(){
	let instructions = this.game.getRenderingInstructions();
	
	let startTime = performance.now();
	this.graphics.preRender(instructions, this.xFactor, this.yFactor);
	let endTime = performance.now();
	this.trackPreRenderTime(endTime - startTime);
}

EventPropagator.prototype.handleResize = function(evt){
	if(this.resizeRefresher){
		this.resizeRefresher = false;
		requestAnimationFrame(() => {
			let width = this.screenDiv.clientWidth;
			let height = this.screenDiv.clientHeight;
			let gameDim = wasm.game_dimensions();
			
			this.width = width;
			this.height = height;
			this.xFactor = width / gameDim.x;
			this.yFactor = height / gameDim.y;
			this.graphics.resize(this.xFactor, this.yFactor);
			
			this.overlay.update({fps: this.fps, xFactor: this.xFactor, yFactor: this.yFactor});

			this.preRender();
			this.resizeRefresher = true;
		});
	}
}

EventPropagator.prototype.handleEvent = function(evt){
	if(this.isBlocking) return;

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
		this.gameKeyDown(evt);
		return;
	}

	if(evt.type === "keyup"){
		this.gameKeyUp(evt);
		return;
	}
	
}

EventPropagator.prototype.handleGameRender = function(evt){
	if(this.isPreRendered == false){
		this.preRender();
		this.isPreRendered = true;
	}
}

EventPropagator.prototype.gameKeyDown = function(evt){
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

EventPropagator.prototype.gameKeyUp = function(evt){
	if(typeof(this.controls[evt.keyCode]) === "number" && this.isRunning){
		this.game.stopControl(this.controls[evt.keyCode]);
	}
}

// track how long the most recent frame lasted
EventPropagator.prototype.trackFPS = function(){
	let now = performance.now();
	let timePassed = now - this.lastFrame;

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

	this.lastFrame = now;
}

// track how long ticks take
EventPropagator.prototype.trackTickTime = function(timePassed){
	this.tickTotalTime += timePassed;
	if(timePassed < this.minTickTracker){
		this.minTickTracker = timePassed;
	}
	if(timePassed > this.maxTickTracker){
		this.maxTickTracker = timePassed;
	}
	this.tickCounter += 1;
	if(this.tickCounter == this.numTicksPerTickCalc){
		let averageTickTime = this.tickTotalTime / this.numTicksPerTickCalc;
		this.averageTickTime = averageTickTime / 1000; // convert to seconds
		this.minTickTime = this.minTickTracker;
		this.maxTickTime = this.maxTickTracker;
		
		this.tickTotalTime = 0;
		this.minTickTracker = 1000;
		this.maxTickTracker = 0;
		this.tickCounter = 0;
	}
}

// track how long pre-renders take
EventPropagator.prototype.trackPreRenderTime = function(timePassed){
	this.preRenderTotalTime += timePassed;
	if(timePassed < this.minPreRenderTracker){
		this.minPreRenderTracker = timePassed;
	}
	if(timePassed > this.maxPreRenderTracker){
		this.maxPreRenderTracker = timePassed;
	}
	this.preRenderCounter += 1;
	if(this.preRenderCounter == this.numPreRenders){
		let averagePreRenderTime = this.preRenderTotalTime / this.numPreRenders;
		this.averagePreRenderTime = averagePreRenderTime;
		this.minPreRenderTime = this.minPreRenderTracker;
		this.maxPreRenderTime = this.maxPreRenderTracker;
		
		this.preRenderTotalTime = 0;
		this.minPreRenderTracker = 1000;
		this.maxPreRenderTracker = 0;
		this.preRenderCounter = 0;
	}
}

