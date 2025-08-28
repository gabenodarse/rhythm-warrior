"use strict";

import * as load from "./load.js";
import * as wasm from "../pkg/rhythm_warrior.js";
import {TimeTracker} from "./TimeTracker.js";

// !!! resizing resizes both overlay and screen div, prompt "your screen has been resized. OK to adjust"
	// resizing retains aspect ratio, attempts to size sidebar to accommodate
	
export function EventPropagator(){
	this.wasmMemoryObj;
	this.game;
	this.overlay;
	this.controls;
	this.graphics;
	
	// screen where everything is added
	this.screenDiv;
	this.xFactor;
	this.yFactor;

	this.resumeEvents; // events to fire once the game resumes
	
	this.isRunning;
	this.stopFlag;
	this.isPreRendered; // boolean describing whether the game state is ready to be re-rendered
	this.isBlocking; // boolean describing whether events are being blocked
	this.resizeRefresher; // boolean to ensure a maximum of one resize event is handled per frame

	this.resize; // function to fire on resize events
	
	// timers
	this.tickTimeTracker;
	this.preRenderTimeTracker;
	this.frameTimeTracker;
	this.fps;

	this.loop;
}

EventPropagator.prototype.init = async function(wasmMemoryObj, game, overlay, controls){
	this.wasmMemoryObj = wasmMemoryObj;
	this.game = game;
	this.overlay = overlay;
	this.controls = controls;
	
	this.loop = () => this.gameLoop();
	this.resumeEvents = [];
	this.stopFlag = false;
	this.isRunning = false;
	this.isBlocking = false;
	this.resizeRefresher = true;

	// timers
	this.tickTimeTracker = new TimeTracker(30);
	this.preRenderTimeTracker = new TimeTracker(30);
	this.frameTimeTracker = new TimeTracker(30);
	this.fps = "fps: ";

	// screen div
	let gameDim = wasm.game_dimensions();
	this.screenDiv = document.getElementById("screen");
	this.xFactor = this.screenDiv.clientWidth / gameDim.x;
	this.yFactor = this.screenDiv.clientHeight / gameDim.y;

	// initialize loader and load graphics
	// TODO error handling when it takes too long
	let loader = new load.Loader();
	await loader.init()
		.then( () => loader.loadGraphics(this.screenDiv))
		.then( res => this.graphics = res )
		.catch( rej => { throw Error("Error initializing loader / loading assets: " + rej ) });
}

EventPropagator.prototype.addListeners = function(){
	this.handleResize(); // to account for any resizes that happened before the listener was added
	
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
	if(this.stopFlag){
		this.stopLoop();
		return;
	} else {
		this.tickTimeTracker.startTime(performance.now());
		this.game.tick();
		this.tickTimeTracker.endTime(performance.now());
		
		let songData = this.game.getSongData();
		let gameData = this.game.getGameData();
		if(gameData.time_running >= gameData.duration){
			this.stopLoop();
			if (!gameData.is_modified) {	
				this.overlay.goToEndGameScreen();
				return;
			}
			
			alert("The song data has been modified. \nAny changes made are not saved automatically, please open the master menu and navigate to " 
				+ "\"save song\" or \"overwrite song\" if you wish to save and download the changes.\n"
				+ "To go back to the home screen, select \"Quit Song\" from the menu");
			return;
		}

		requestAnimationFrame(() => {
			// fps
			let fpsVal = 1000 / this.frameTimeTracker.getMostRecent().pop().average;
			if(Number.isFinite(fpsVal)){
				this.fps = "fps: " + Math.round(fpsVal);
			} 
			else {
				this.fps = "fps: ";
			}
			this.frameTimeTracker.endTime(performance.now()); // record the end time for last frame before recording the start time of this one
			this.frameTimeTracker.startTime(performance.now());

			// if the game is pre-rendered, dispatch an event saying that the render occurred (triggering another pre-render)
			// (for performance, instead of prerendering before the repaint, prerender when the gameRender event or some other event triggers one)
			if(this.isPreRendered){
				this.overlay.update({fps: this.fps, xFactor: this.xFactor, yFactor: this.yFactor});
				
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
	
	this.preRender();
	this.isPreRendered = true;

	this.game.start(() => {
		// start fps tracker
		this.frameTimeTracker.startTime(performance.now());

		this.loop();
	});
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
	} 
	
	else if(instruction == "restart-song"){
		if(this.isRunning){
			this.togglePlay();
		}
		this.tickTimeTracker.reset();
		this.preRenderTimeTracker.reset();
		this.frameTimeTracker.reset();
		this.game.restart();
	} 
	
	else if(instruction == "stop-loop"){
		this.stopFlag = true;
	} 
	
	else if(instruction == "start-loop"){
		this.startLoop();
	} 
	
	else if(instruction == "start-from-homescreen"){
		this.tickTimeTracker.reset();
		this.preRenderTimeTracker.reset();
		this.frameTimeTracker.reset();
		
		this.startFromHomescreen();
	} 
	
	else if(instruction == "wait-song-load"){
		this.waitSongLoad();
	} 
	
	else if(instruction == "pre-render"){
		this.overlay.update({fps: this.fps, xFactor: this.xFactor, yFactor: this.yFactor});
	} 
	
	else if(instruction == "download-log"){
		this.downloadLog();
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
	
	this.preRenderTimeTracker.startTime(performance.now());
	this.graphics.preRender(instructions, this.wasmMemoryObj);
	this.preRenderTimeTracker.endTime(performance.now());
}

EventPropagator.prototype.handleResize = function(evt){
	if(this.resizeRefresher){
		this.resizeRefresher = false;
		requestAnimationFrame(() => {
			let width = this.screenDiv.clientWidth;
			let height = this.screenDiv.clientHeight;
			let gameDim = wasm.game_dimensions();
			
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

EventPropagator.prototype.downloadLog = function(){
	let fileObj = {};
	fileObj.tickAggregates = this.tickTimeTracker.getAggregates();
	fileObj.preRenderAggregates = this.preRenderTimeTracker.getAggregates();
	fileObj.frameAggregates = this.frameTimeTracker.getAggregates();
	fileObj.ticksComplete = this.tickTimeTracker.getRecord();
	fileObj.preRendersComplete = this.preRenderTimeTracker.getRecord();
	fileObj.framesComplete = this.frameTimeTracker.getRecord();

	let fileStr = JSON.stringify(fileObj, null, 4);
	let file = new Blob([fileStr], { type: 'text/plain' });

	let a = document.createElement('a')
	a.href = URL.createObjectURL(file);
	a.download = "log.txt";
	
	document.body.appendChild(a)
	a.click()
	document.body.removeChild(a)
}

