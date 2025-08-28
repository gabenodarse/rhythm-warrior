import * as wasm from "../pkg/rhythm_warrior.js";
import {GameCore} from "./GameCore.js";
import {EventPropagator} from "./EventPropagator.js";
import {Overlay} from "./Overlay.js";

let g_wasmMemoryObj;
let g_controls;
let g_game;
let g_overlay;
let g_propagator;

// entry point. The overlay displays a pre initialized "welcome" screen where the user is prompted to load and initialize the game.
export function run(){
	g_overlay = new Overlay();
	g_overlay.preInit(initFunction);
}

async function initFunction(){
	g_overlay.updatePreInit(0);
	
	// initialize wasm memory object
	let wasmObj = await wasm.default();
	g_wasmMemoryObj = wasmObj.memory;
	
	g_overlay.updatePreInit(0.1);
	
	// initialize controls
	g_controls = [];
	g_controls[13] = wasm.Input.Dash; // space
	g_controls[81] = wasm.Input.Slash1; // q
	g_controls[87] = wasm.Input.Slash2; // w
	g_controls[69] = wasm.Input.Slash3; // e
	
	// initialize game
	g_game = new GameCore();
	await g_game.init(g_wasmMemoryObj);
	g_game = g_game.toGame();
	
	g_overlay.updatePreInit(0.2);
	
	// initialize event propagator
	g_propagator = new EventPropagator();
	await g_propagator.init(g_wasmMemoryObj, g_game, g_overlay, g_controls);
	
	g_overlay.updatePreInit(0.4);
	
	let songs = g_game.songs();
	for(let i = 0; i < songs.length; ++i){
		await g_game.loadSongAudio(songs[i]);
		g_overlay.updatePreInit(0.4 + 0.6*(i+1)/songs.length);
	}
	
	// initialize overlay
	g_overlay.initGame(g_game, g_controls);
	
	// add the event listeners
	g_propagator.addListeners();
};