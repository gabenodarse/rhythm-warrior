
import * as wasm from "../pkg/music_mercenary.js";
import {Game, Editor} from "./Game.js";
import {EventPropagator} from "./EventPropagator.js";
import {Overlay} from "./overlay.js";

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
	
	game = new Game();
	await game.init();

	overlay = new Overlay(game, controls);

	propagator = new EventPropagator();
	propagator.init(game, overlay, controls);
}