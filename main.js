"use strict";

import * as graphics from "./graphics.js";
import * as wasm from "./pkg/music_mercenary.js";


export async function run() {
	let resourceLocations;
	let gameInstance;
	
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
			gameInstance = initGame();
			graphics.renderAll(gameInstance.get_instructions()); },
		rej => { alert("loadImages FAILED" + rej); }
	);
	
	
	// !!! make loading a song its own function
	// TODO add error handling
	await fetch("./Here-we-go.json")
		.then(res => res.json())
		.then(res => { 
			res.forEach( entry => {
				gameInstance.load_brick(entry[0], entry[1], entry[2]);
			});
		}) 
	;
	
	
	let last = (window.performance && window.performance.now) ? window.performance.now() : new Date().getTime();
	let now;
	const renderLoop = () => {
		// at a certain time threshold, get instruction
			// then render, asynchronously if possible to keep game ticking

		now = (window.performance && window.performance.now) ? window.performance.now() : new Date().getTime();
		
		
		// !!!
		// handle if there's too long a time between ticks (pause game?)
		gameInstance.tick((now - last) / 1000); // convert to seconds
		graphics.renderAll(gameInstance.get_instructions());
		// !!! get fps
		last = now;
		
		
		requestAnimationFrame(renderLoop);
	};
	
	requestAnimationFrame(renderLoop);
	
}


function initGame() {
	// TODO throw or handle any errors
	let myGame = wasm.Game.new();
	
	window.addEventListener("keydown", event => {
		// TODO faster handling of repeated key inputs from holding down a key?
		switch (event.keyCode) {
			case wasm.InputKey.Space:
			case wasm.InputKey.Comma:
			case wasm.InputKey.Period:
			case wasm.InputKey.Q:
				myGame.input_command(event.keyCode);
				break;
			case 27: // escape
				alert("Escape pressed"); // !!! Pause game
				break;
		}
	}, true);
	window.addEventListener("keyup", event => {
		switch (event.keyCode) {
			case wasm.InputKey.Space:
			case wasm.InputKey.Comma:
			case wasm.InputKey.Period:
			case wasm.InputKey.Q:
				myGame.stop_command(event.keyCode);
				break;
			
		}
	}, true);
	
	
	return myGame;
}