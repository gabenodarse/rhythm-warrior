"use strict";

import * as graphics from "./graphics.js";
import * as wasm from "./pkg/music_mercenary.js";
import * as loader from "./load.js";

let g_controls = {};
let g_game;
let g_gamePaused = false;
let g_startGame = () => {}

const g_handleKeyDown = event => {
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
		g_game.input_command(g_controls[event.keyCode]);
	}
	
	event.preventDefault();
}
const g_handleKeyUp = event => {
	if(typeof(g_controls[event.keyCode]) === "number" && !g_gamePaused){
		g_game.stop_command(g_controls[event.keyCode]);
	}
};

export async function run() {
	let resourceLocations;
	let db;
	const audioContext = new AudioContext();
	let audioSource;
	let audioBuffer;
	let songTime = 0;
	const audioTimeSafetyBuffer = 0.15; // longer gives the hardware more time to prepare the sound to end at a precise time
	
	// !!! move the loading to load.js
	// TODO add error handling
	await Promise.all( [ 
		wasm.default(),
		fetch("./resources.json")
			.then(res => res.json())
			.then(res => { resourceLocations = res })
	]);
	
	// TODO add error handling
	await graphics.loadImages(resourceLocations).then( 
		() => { 
			initGame();
			graphics.renderAll(g_game.get_instructions()); },
		rej => { console.log("loadImages FAILED" + rej); }
	);
	
	// TODO add error handling
	await loader.loadDefaultDB()
		.then(res => {
			db = res;
		})
		.catch( rej => {
			console.log("could not load the the database. Err: " + rej);
		}
	);
	
	// TODO add error handling
	await fetch("song.mp3")
		.then(res => res.arrayBuffer())
		.then(res => audioContext.decodeAudioData(res))
		.then(res => { audioBuffer = res; }
	);
	
	let songData = loader.loadSong(1, db);
	songData[0]["values"].forEach( note => {
		g_game.load_brick(note[2], note[3], note[4]);
	});
	
	
	
	let last; // last tick time
	let now;
	const renderLoop = () => {
		now = (window.performance && window.performance.now) ? window.performance.now() : new Date().getTime();
		
		if(g_gamePaused) {
			let switchTime = audioContext.currentTime + audioTimeSafetyBuffer;
			audioSource.stop(switchTime);
			let timePassed = (now - last) / 1000; // convert to seconds
			g_game.tick(timePassed + audioTimeSafetyBuffer);
			songTime += timePassed + audioTimeSafetyBuffer;
			
			return;
		}
		
		// !!! render asynchronously to keep game ticking???
		// !!! handle if there's too long a time between ticks (pause game?)
		// TODO get fps
		let timePassed = (now - last) / 1000; // convert to seconds
		songTime += timePassed;
		g_game.tick(timePassed); 
		last = now;
		
		graphics.renderAll(g_game.get_instructions());
		
		requestAnimationFrame(renderLoop);
	};
	
	const start = () => {
		// !!! creating a new buffer source each time because I couldn't figure out how to resume audio precisely
			// make sure multiple buffer sources don't linger in memory
		audioSource = audioContext.createBufferSource(); 
		audioSource.buffer = audioBuffer;
		audioSource.connect(audioContext.destination);
		
		let switchTime = audioContext.currentTime + audioTimeSafetyBuffer;
		audioSource.start(switchTime, songTime);
		last = (window.performance && window.performance.now) ? window.performance.now() : new Date().getTime() 
		last += audioTimeSafetyBuffer * 1000;
		
		requestAnimationFrame(renderLoop);
	}
	
	g_startGame = start;
	start();
}


function initGame() {
	// TODO throw or handle any errors
	g_game = wasm.Game.new();
	
	g_controls[32] = wasm.Input.Jump; // space
	g_controls[188] = wasm.Input.Left; // comma
	g_controls[190] = wasm.Input.Right; // period
	g_controls[81] = wasm.Input.Ability1; // q
	g_controls[87] = wasm.Input.Ability2; // w
	g_controls[69] = wasm.Input.Ability3; // e
	g_controls[82] = wasm.Input.Ability4; // r
	
	window.addEventListener("keydown", g_handleKeyDown);
	window.addEventListener("keyup", g_handleKeyUp);
}

function pause() {
	//>:< handle key states on pause/unpause
	for(const key in g_controls) {
		let event = new KeyboardEvent("keyup", {
			keyCode: key,
		});
		g_handleKeyUp(event);
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
	
	//>:< move so event listeners are only added once (not on every call of the function)
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
	// >:< accept controls with a different button
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


