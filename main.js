"use strict";

import * as graphics from "./graphics.js";
import * as myWasm from "./pkg/music_mercenary.js"; // !!! split into modules and change import name from myWasm


export async function run() {
	let resourceLocations;
	let gameInstance;
	
	
	await Promise.all( [ 
		myWasm.default(),
		fetch("./resources.json")
			.then(r => r.json())
			.then(r => { resourceLocations = r }) // !!! add error handling
	]);
	
	
	await graphics.loadImages(resourceLocations).then( 
		() => { 
			gameInstance = initGame(); // !!! add error handling
			graphics.renderAll(gameInstance.get_instructions()); },
		e => { alert("loadImages FAILED" + e); } // !!! add error handling
	);
	
	
	let last = (window.performance && window.performance.now) ? window.performance.now() : new Date().getTime();
	let now;
	const renderLoop = () => {
		// at a certain time threshold, get instruction
			// then render, asynchronously if possible to keep game ticking

		now = (window.performance && window.performance.now) ? window.performance.now() : new Date().getTime();
		
		
		// >:<
		// handle if there's too long a time between ticks
		gameInstance.tick((now - last) / 1000); // convert to seconds
		graphics.renderAll(gameInstance.get_instructions());
		// !!! get fps
		last = now;
		
		
		requestAnimationFrame(renderLoop);
	};
	
	requestAnimationFrame(renderLoop);
	
}


function initGame() {
	// !!! throw any errors
	let myGame = myWasm.Game.new();
	
	window.addEventListener("keydown", event => {
		// TODO faster handling of repeated key inputs from holding down a key?
		switch (event.keyCode) {
			case myWasm.InputKey.Space:
			case myWasm.InputKey.Comma:
			case myWasm.InputKey.Period:
			case myWasm.InputKey.Q:
				myGame.input_command(event.keyCode);
				break;
			case 27: // escape
				alert("Escape pressed"); //>:<
				break;
		}
	}, true);
	window.addEventListener("keyup", event => {
		switch (event.keyCode) {
			case myWasm.InputKey.Space:
			case myWasm.InputKey.Comma:
			case myWasm.InputKey.Period:
			case myWasm.InputKey.Q:
				myGame.stop_command(event.keyCode);
				break;
			
		}
	}, true);
	
	
	return myGame;
}