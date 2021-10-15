
import * as wasm from "../pkg/music_mercenary.js";
import {Game, Editor} from "./Game.js";
import {EventPropagator} from "./EventPropagator.js";
import {Overlay} from "./overlay.js";

export async function run(){
	let game = new Game();
	let propagator = new EventPropagator();
	let overlay;
	let controls = [];
	
	await game.init();
	
	if(Object.keys(controls).length == 0){
		controls[32] = wasm.Input.Dash; // space
		controls[81] = wasm.Input.Slash1; // q
		controls[87] = wasm.Input.Slash2; // w
		controls[69] = wasm.Input.Slash3; // e
	}
	
	// !!! order kinda janky because keys have to be set before overlay creation
		// and overlay creation needs to happen before propagator init
	overlay = new Overlay(game.songData(), propagator, controls); 
	overlay.showElement("score");
	propagator.init(game, overlay, controls);
	
	propagator.start();
}