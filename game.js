"use strict";

import * as graphics from "./graphics.js";
import * as wasm from "./pkg/music_mercenary.js";
import * as load from "./load.js";

let g_controls = {};
let g_game;
let g_gamePaused = false;
let g_startGame = () => {}
let g_gameStartControl = (cntrl) => {}
let g_gameStopControl = (cntrl) => {}

const g_handleGameKeyDown = event => {
	// TODO faster handling of repeated key inputs from holding down a key?
	if (event.keyCode === 27){
		if(g_gamePaused){
			unpause();
		}
		else{
			pause();
		}
	}
	else if(typeof(g_controls[event.keyCode]) === "number" && !g_gamePaused){
		g_gameStartControl(g_controls[event.keyCode]);
	}
	
	event.preventDefault();
}

const g_handleGameKeyUp = event => {
	if(typeof(g_controls[event.keyCode]) === "number" && !g_gamePaused){
		g_gameStopControl(g_controls[event.keyCode]);
	}
};

export async function run() {
	let game = new Game();
	await game.load();
	
	const loop = () => {
		if(g_gamePaused) {
			game.pause();
			return;
		}
		
		game.tick();
		requestAnimationFrame(loop);
	}
	
	const start = () => {
		game.start(loop);
	}
	
	g_startGame = start;
	g_gameStartControl = (cntrl) => {
		game.startControl(cntrl);
	}
	g_gameStopControl = (cntrl) => {
		game.stopControl(cntrl);
	}
	
	start();
}

function Game () {
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
	this.songTime = 0;
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
	
	g_controls = [];
	g_controls[32] = wasm.Input.Jump; // space
	g_controls[188] = wasm.Input.Left; // comma
	g_controls[190] = wasm.Input.Right; // period
	g_controls[81] = wasm.Input.Ability1; // q
	g_controls[87] = wasm.Input.Ability2; // w
	g_controls[69] = wasm.Input.Ability3; // e
	g_controls[82] = wasm.Input.Ability4; // r
	
	// remove event listeners that aren't game // >:<
	window.addEventListener("keydown", g_handleGameKeyDown);
	window.addEventListener("keyup", g_handleGameKeyUp);
	
	let gameDim = wasm.game_dimensions();
	this.xFactor = this.gameCanvas.width / gameDim.x;
	this.yFactor = this.gameCanvas.height / gameDim.y;
	
	let songData = loader.getSong(1);
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
	this.audioSource.start(switchTime, this.songTime);
	this.lastTick = new Date().getTime() + this.audioTimeSafetyBuffer * 1000;
	
	requestAnimationFrame(callback);
}

Game.prototype.pause = function(){
	let now = new Date().getTime();
	let switchTime = this.audioContext.currentTime + this.audioTimeSafetyBuffer;
	this.audioSource.stop(switchTime);
	let timePassed = (now - this.lastTick) / 1000; // convert to seconds
	this.gameData.tick(timePassed + this.audioTimeSafetyBuffer);
	this.songTime += timePassed + this.audioTimeSafetyBuffer;
}

Game.prototype.tick = function(){
	let now = new Date().getTime();
	// !!! render asynchronously to keep game ticking???
	// !!! handle if there's too long a time between ticks (pause game?)
	// !!! get fps, average, and log
	let timePassed = (now - this.lastTick) / 1000; // convert to seconds
	this.songTime += timePassed;
	this.gameData.tick(timePassed); 
	this.lastTick = now;
	
	graphics.renderAll(this.gameData.rendering_instructions(), this.canvases, this.xFactor, this.yFactor, this.gameContext);
}

Game.prototype.startControl = function(cntrl){
	let now = new Date().getTime();
	this.gameData.input_command(cntrl, (now - this.lastTick) / 1000);
}

Game.prototype.stopControl = function(cntrl){
	let now = new Date().getTime();
	this.gameData.stop_command(cntrl, (now - this.lastTick) / 1000);
}

function pause() {
	//>:< handle key states on pause/unpause
	for(const key in g_controls) {
		let evt = new KeyboardEvent("keyup", {
			keyCode: key,
		});
		g_handleGameKeyUp(evt);
	}
	
	g_gamePaused = true;
	controls(); // !!! create a pause menu and get to controls from there
}

function unpause() {
	g_gamePaused = false;
	g_startGame();
}

// !!! add error handling
function controls() {
	let inputKeys = [];
	
	for (const key in g_controls) {
		inputKeys[g_controls[key]] = key;
	}
	
	// assuming the inputs are numbered 0..number-of-different-inputs
	// add all buttons to change controls as children
	let changeControlsMenu = document.querySelector("#control-change-menu");
	let controlChangeDivs = changeControlsMenu.children;
	for(let i = 0; i < controlChangeDivs.length; ++i){
		let children = controlChangeDivs[i].children;
		controlChangeDivs[i].label = children[0];
		controlChangeDivs[i].button = children[1];
	}
	
	controlChangeDivs[wasm.Input.Jump].label.prefixText = "Jump - ";
	controlChangeDivs[wasm.Input.Left].label.prefixText = "Move left - ";
	controlChangeDivs[wasm.Input.Right].label.prefixText = "Move right - ";
	controlChangeDivs[wasm.Input.Ability1].label.prefixText = "Ability 1 - ";
	controlChangeDivs[wasm.Input.Ability2].label.prefixText = "Ability 2 - ";
	controlChangeDivs[wasm.Input.Ability3].label.prefixText = "Ability 3 - ";
	controlChangeDivs[wasm.Input.Ability4].label.prefixText = "Ability 4 - ";
	for(let i = 0; i < controlChangeDivs.length; ++i){
		controlChangeDivs[i].label.innerHTML = controlChangeDivs[i].label.prefixText + inputKeys[i];
	}
	
	let changeInputKey = (input, newKey) => {
		// mark the input as having a new associated key, and display update in the menu
		inputKeys[input] = newKey;
		controlChangeDivs[input].label.innerHTML = controlChangeDivs[input].label.prefixText + newKey;
	}
		
	let awaitNewKey = (input) => {
		let screenBlocker = document.querySelector("#screen-blocker");
		screenBlocker.style.display = "initial";
		
		let handleKeyPress = event => {
			changeInputKey(input, event.keyCode);
			screenBlocker.style.display = "none";
		}
			
		window.addEventListener("keydown", handleKeyPress, {once: true});
	}
	
	// !!! move so event listeners are only added once (not on every call of the function)
	let eventHandlers = [];
	eventHandlers[wasm.Input.Jump] = () => { awaitNewKey(wasm.Input.Jump); };
	eventHandlers[wasm.Input.Left] = () => { awaitNewKey(wasm.Input.Left); };
	eventHandlers[wasm.Input.Right] = () => { awaitNewKey(wasm.Input.Right); };
	eventHandlers[wasm.Input.Ability1] = () => { awaitNewKey(wasm.Input.Ability1); };
	eventHandlers[wasm.Input.Ability2] = () => { awaitNewKey(wasm.Input.Ability2); };
	eventHandlers[wasm.Input.Ability3] = () => { awaitNewKey(wasm.Input.Ability3); };
	eventHandlers[wasm.Input.Ability4] = () => { awaitNewKey(wasm.Input.Ability4); };
	eventHandlers.forEach( (eventHandler, input) => {
		controlChangeDivs[input].button.addEventListener("click", eventHandler);
	});
	
	changeControlsMenu.style.display = "block";
	
	// !!! add ability to accept or cancel changed controls
	// !!! accept controls with a different button
	let acceptControls = event => {
		if (event.keyCode === 192){
			changeControlsMenu.style.display = "none";
			g_controls = {};
			for (const control in inputKeys) {
				g_controls[inputKeys[control]] = parseInt(control);
			}
			
			window.removeEventListener("keydown", acceptControls);
		}
	}
	window.addEventListener("keydown", acceptControls);
	
}

// >:< editor
function initEditor() {
	let game = new Game();
	await game.load();
	
	window.removeEventListener("keydown", g_handleGameKeyDown);
	window.removeEventListener("keyup", g_handleGameKeyUp);
}
