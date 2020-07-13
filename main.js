
import * as wasm from "./pkg/music_mercenary.js";
import * as load from "./load.js";
import * as game from "./game.js";

const Game = game.Game;
const Editor = game.Editor;

let g_resizeRefresher = true;
let g_controls = {};
let g_gamePaused = false;
let g_startGame = () => {}
let g_gameStartControl = (cntrl) => {}
let g_gameStopControl = (cntrl) => {}
let g_resizeGame = (width, height) => {}
let g_resize = () => {}

const g_resizeScreen = () => {
	const delay = 50; // minimum delay between screen size refreshes
	if(g_resizeRefresher){
		g_resizeRefresher = false;
		setTimeout(async function(){
			g_resizeRefresher = true;
			let screenDiv = document.querySelector("#screen-div");
			g_resizeGame(screenDiv.clientWidth, screenDiv.clientHeight);
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

export async function run() {
	let game = new Game();
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
	
	const loop = () => {
		if(g_gamePaused) {
			game.pause();
			return;
		}
		
		game.tick();
		game.renderGame();
		requestAnimationFrame(loop); // !!! set timeout or request animation frame better?
	}
	
	g_startGame = () => {
		game.start(loop);
	}
	g_gameStartControl = (cntrl) => {
		game.startControl(cntrl);
	}
	g_gameStopControl = (cntrl) => {
		game.stopControl(cntrl);
	}
	g_resizeGame = (width, height) => {
		game.resize(width, height);
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
	g_resizeGame = (width, height) => {
		editor.resize(width, height);
	}
	
	let start = () => {
		editor.seek(0);
		editor.renderEditor();
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
	controls(); // !!! create a pause menu and get to controls from there
}

function unpause() {
	g_gamePaused = false;
	g_startGame();
}

// !!! add error handling
function controls() {
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
		// mark the input as having a new associated key, and display update in the menu
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
	
	// !!! move so event listeners are only added once (not on every call of the function)
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
	// !!! accept controls with a different button
	let acceptControls = event => {
		if (event.keyCode === 192){
			changeControlsMenu.style.display = "none";
			g_controls = {};
			for (const control in inputKeys) {
				g_controls[inputKeys[control]] = parseInt(control);
			}
			
			window.removeEventListener("keydown", acceptControls);
		}
	}
	window.addEventListener("keydown", acceptControls);
	
}
