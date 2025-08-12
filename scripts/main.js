
import * as wasm from "../pkg/rhythm_warrior.js";
import {GameCore} from "./GameCore.js";
import {EventPropagator} from "./EventPropagator.js";
import {Overlay} from "./Overlay.js";

export async function run(){
	let controls;
	let game;
	let overlay;
	let propagator;
	
	controls = [];
	controls[13] = wasm.Input.Dash; // space
	controls[81] = wasm.Input.Slash1; // q
	controls[87] = wasm.Input.Slash2; // w
	controls[69] = wasm.Input.Slash3; // e
	
	game = new GameCore();
	await game.init();
	game = game.toGame();
	
	overlay = new Overlay(game, controls);
	propagator = new EventPropagator();	

	await propagator.init(game, overlay, controls);
}