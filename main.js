
import * as wasm from "./pkg/music_mercenary.js";
import {Game, Editor} from "./Game.js";
import {EventPropagator} from "./EventPropagator.js";
import {Overlay} from "./overlay.js";

// >:< update score
// !!! resizing resizes both overlay and screen div, prompt "your screen has been resized. OK to adjust"
	// resizing retains aspect ratio, attempts to size sidebar to accommodate

export async function run(){
	let game = new Game();
	let propagator = new EventPropagator();
	let overlay;
	let controls = [];
	
	await game.load();
	
	if(Object.keys(controls).length == 0){
		controls[32] = wasm.Input.Jump; // space
		controls[188] = wasm.Input.Left; // comma
		controls[190] = wasm.Input.Right; // period
		controls[81] = wasm.Input.Ability1; // q
		controls[87] = wasm.Input.Ability2; // w
		controls[69] = wasm.Input.Ability3; // e
		controls[82] = wasm.Input.Ability4; // r
	}
	
	// !!! order kinda janky because keys have to be set before overlay creation
		// and overlay creation needs to happen before propagator init
	overlay = new Overlay(game, propagator, controls); 
	overlay.toggleElement("score");
	propagator.init(game, overlay, controls);
	
	propagator.start();
}
