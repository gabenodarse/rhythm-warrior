"use strict";

import * as graphics from "./graphics.js";
import * as wasm from "./pkg/music_mercenary.js";

let g_controls = {};
let g_game;
let g_gamePaused = false;

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
		rej => { alert("loadImages FAILED" + rej); }
	);
	
	// !!! make loading a song its own function
	// TODO add error handling
	await fetch("./Here-we-go.json")
		.then(res => res.json())
		.then(res => { 
			res.forEach( entry => {
				g_game.load_brick(entry[0], entry[1], entry[2]);
			});
		}) 
	;
	
	let last = (window.performance && window.performance.now) ? window.performance.now() : new Date().getTime();
	let now;
	const renderLoop = () => {
		now = (window.performance && window.performance.now) ? window.performance.now() : new Date().getTime();
		
		if(!g_gamePaused) {
			// at a certain time threshold, get instruction
			// then render, asynchronously if possible to keep game ticking
		
			// !!! handle if there's too long a time between ticks (pause game?)
			// TODO get fps
			g_game.tick((now - last) / 1000); // convert to seconds
			graphics.renderAll(g_game.get_instructions());
		}
		
		last = now;
		
		requestAnimationFrame(renderLoop);
	};
	
	requestAnimationFrame(renderLoop);
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
	for(const key in g_controls) {
		let event = new KeyboardEvent("keyup", {
			keyCode: key,
		});
		g_handleKeyUp(event);
	}
	
	g_gamePaused = true;
	controls(); //>:< create a pause menu and get to controls from there from pause menu
}

function unpause() {
	//>:< put down all keys that are down
	g_gamePaused = false;
}

// !!! add error handling
function controls() {
	let newControls = Object.assign({}, g_controls);
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
		// remove the old key from the controls
		let oldKey = inputKeys[input];
		if(oldKey){
			delete newControls[oldKey];
		}
		
		// check if the input's new associated key is already a control.
			// If so, set the input associated with that key to false because the control will be changed
		if(newControls[newKey]){
			inputKeys[ newControls[newKey] ] = false;
		}
		
		// add the new key to the controls, mark the input as having a new associated key, and display update in the menu
		newControls[newKey] = input;
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
	
	//>:< move so event listeners are only added once
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
			g_controls = newControls;
			window.removeEventListener("keydown", acceptControls);
		}
	}
	window.addEventListener("keydown", acceptControls);
	
}