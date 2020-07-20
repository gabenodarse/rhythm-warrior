
import * as wasm from "./pkg/music_mercenary.js";
import {Game, Editor, createEditorFromGame} from "./game.js";
import * as overlay from "./overlay.js";

// >:< eliminate globals by creating functions to set up event handlers ?? EventPropagator
	// then combine game, event propagator, and overlay into one master object?
// !!! make screwing with game through editor or otherwise more difficult
// >:< changing from editor to game and back
// >:< arrange overlay.js, Controller
// >:< single editor object within overlay
// !!! disable player graphic in editor
	// make song work in editor
// resizing resizes both overlay and screen div, prompt "your screen has been resized. OK to adjust"
	// resizing retains aspect ratio, attempts to size sidebar to accommodate
// click through div or dispatch events from overlay to screen/body?
// >:< 
	// this.scroller.value = this.gameData.song_time();
	// this.scroller.max = this.gameData.song_duration();
	// this.scroller.step = this.gameData.beat_interval() * 2;
	// this.scroller.addEventListener("input", evt => {
		// let t = parseInt(this.scroller.value);
		// this.seek(t);
		// this.renderGame();
	// });
// editor doesn't update on pause. Fix when consolidating to EventPropagator
// !!! toggles in propagator and overlay to display/hide
function EventPropagator(){
	this.connectedGame;
	this.connectedOverlay;
}

EventPropagator.prototype.init = function(game, overlay){
	this.connectedGame = game;
	this.connectedOverlay = overlay;
}

EventPropagator.prototype.enableEditor = function(){
	this.connectedOverlay.toggleElement("editorOverlay");
	this.connectedGame = createEditorFromGame(this.connectedGame);
	// >:< set editor overlay to update on ticks
}

let g_resizeRefresher = true;
let g_controls = {}; // maps key codes to game controls
let g_gamePaused = false;
let g_startGame = () => {}
let g_gameStartControl = (cntrl) => {}
let g_gameStopControl = (cntrl) => {}
let g_resizeGame = () => {}
let g_resize = () => {}

const g_resizeScreen = () => {
	const delay = 50; // minimum delay between screen size refreshes
	if(g_resizeRefresher){
		g_resizeRefresher = false;
		setTimeout(async function(){
			g_resizeRefresher = true;
			g_resizeGame();
		},delay);
	}
}
	
const g_handleGameKeyDown = event => {
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
		g_gameStartControl(g_controls[event.keyCode]);
	}
	
	event.preventDefault();
}

const g_handleGameKeyUp = event => {
	if(typeof(g_controls[event.keyCode]) === "number" && !g_gamePaused){
		g_gameStopControl(g_controls[event.keyCode]);
	}
};

export async function run(){
	let game = new Game();
	let propagator = new EventPropagator();
	let overlayObj;
	
	await game.load();
	
	window.addEventListener("keydown", g_handleGameKeyDown);
	window.addEventListener("keyup", g_handleGameKeyUp);
	window.addEventListener("resize", g_resizeScreen); 
	
	if(Object.keys(g_controls).length == 0){
		g_controls[32] = wasm.Input.Jump; // space
		g_controls[188] = wasm.Input.Left; // comma
		g_controls[190] = wasm.Input.Right; // period
		g_controls[81] = wasm.Input.Ability1; // q
		g_controls[87] = wasm.Input.Ability2; // w
		g_controls[69] = wasm.Input.Ability3; // e
		g_controls[82] = wasm.Input.Ability4; // r
	}
	
	// !!! order kinda janky because keys have to be set before overlay creation
		// and overlay creation needs to happen before propagator init
	overlayObj = new overlay.Overlay(game, propagator, g_controls); 
	overlayObj.toggleElement("score");
	propagator.init(game, overlayObj);
	
	const loop = () => {
		if(g_gamePaused) {
			overlayObj.toggleElement("menu");
			game.pause();
			return;
		}
		
		game.tick();
		// >:< 
		overlayObj.updateEditor(game.gameData.song_time());
		game.renderGame();
		requestAnimationFrame(loop); // !!! set timeout or request animation frame better?
	}
	
	g_startGame = () => {
		overlayObj.toggleElement("menu");
		game.start(loop);
	}
	g_gameStartControl = (cntrl) => {
		game.startControl(cntrl);
	}
	g_gameStopControl = (cntrl) => {
		game.stopControl(cntrl);
	}
	g_resizeGame = () => {
		game.resize();
	}
	
	// !!! how to end a game/editor
	g_resizeScreen();
	game.start(loop);
}

export async function runEditor() {
	window.addEventListener("resize", g_resizeScreen);
	let editor = new Editor();
	await editor.load();
	
	// !!! way to work with editor and song together
	g_resizeGame = () => {
		editor.resize();
	}
	
	let start = () => {
		editor.seek(0);
		editor.renderGame();
	}
	
	g_resizeScreen();
	editor.start(start);
}

function pause() {
	// !!! handle key states on pause/unpause
	for(const key in g_controls) {
		let evt = new KeyboardEvent("keyup", {
			keyCode: key,
		});
		g_handleGameKeyUp(evt);
	}
	
	g_gamePaused = true;
}

function unpause() {
	g_gamePaused = false;
	g_startGame();
}
