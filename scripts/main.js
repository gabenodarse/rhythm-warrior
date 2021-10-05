
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
		controls[219] = wasm.Input.Left; // left bracket
		controls[221] = wasm.Input.Right; // right bracket
		controls[81] = wasm.Input.Ability1; // q
		controls[87] = wasm.Input.Ability2; // w
		controls[69] = wasm.Input.Ability3; // e
		controls[82] = wasm.Input.Ability4; // r
	}
	
	// !!! order kinda janky because keys have to be set before overlay creation
		// and overlay creation needs to happen before propagator init
	overlay = new Overlay(game.songData(), propagator, controls); 
	overlay.toggleElement("score");
	propagator.init(game, overlay, controls);
	
	propagator.start();
}
