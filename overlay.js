
import * as wasm from "./pkg/music_mercenary.js";

// !!! finish map
let g_keyCodeNames = [];
g_keyCodeNames[32] = "space";
g_keyCodeNames[188] = ", (<)";
g_keyCodeNames[190] = ". (>)";
g_keyCodeNames[81] = "q";
g_keyCodeNames[87] = "w";
g_keyCodeNames[69] = "e";
g_keyCodeNames[82] = "r";

export function Overlay(game, eventPropagater, controlsMap){
	this.overlay;
	this.score;
	this.menu;
	this.editorOverlay;
	
	let overlay = document.createElement("div");
	let score = new Score();
	let menu = new Menu(eventPropagater, controlsMap);
	let editorOverlay = new EditorOverlay(game); // >:< single editor object
	
	overlay.style.width = "100vw";
	overlay.style.height = "100vh";
	overlay.style.position = "absolute";
	overlay.style.left = "0";
	overlay.style.top = "0";
	overlay.style.pointerEvents = "none";
	
	overlay.appendChild(score.domElement());
	overlay.appendChild(menu.domElement());
	overlay.appendChild(editorOverlay.domElement());
	document.body.appendChild(overlay);
	
	this.overlay = overlay;
	this.score = score;
	this.menu = menu;
	this.editorOverlay = editorOverlay;
}

// >:< for when a game object isn't tied to 1 specific song
// Overlay.prototype.createNewEditor = function(bpm, pixelsPerSecond){
	// this.editorGuidingLines.updateState(bpm, pixelsPerSecond);
// }

Overlay.prototype.updateEditor = function(time){
	this.editorOverlay.updateTime(time);
}

function EditorOverlay(game){
	this.div;
	this.guidingLines;
	this.scroller;
	
	this.div = document.createElement("div");
	this.div.style.width = "100vw";
	this.div.style.height = "100vh";
	this.div.style.display = "none";
	
	this.guidingLines = new EditorGuidingLines(game);
	
	this.div.appendChild(this.guidingLines.domElement());
}

EditorOverlay.prototype.toggle = function(){
	if(this.div.style.display != "none"){
		this.div.style.display = "none";
	}
	else{
		this.div.style.display = "block";
	}
}

EditorOverlay.prototype.updateTime = function(time){
	this.guidingLines.updateTime(time);
}

EditorOverlay.prototype.domElement = function(){
	return this.div;
}

function EditorGuidingLines(game){
	this.canvas;
	this.beatInterval; // how long between beats in seconds
	this.beatPixelInterval; // how many pixels between beats
	this.groundPosOffset = wasm.ground_pos();
	
	this.canvas = document.createElement("canvas");
	this.canvas.width;
	this.canvas.height;
	this.canvas.style.width = "100%";
	this.canvas.style.height = "100%";
	
	let dims = wasm.game_dimensions();
	this.canvas.width = dims.x;
	this.canvas.height = dims.y;
	
	let songData = game.songData();
	this.beatInterval = songData.beatInterval;
	this.beatPixelInterval = this.beatInterval * songData.brickSpeed;
}

EditorGuidingLines.prototype.domElement = function(){
	return this.canvas;
}

// TODO faster if the canvas stays the same and is just repositioned on time changes. 
	// However, if the game height is not the full screen height, lines would show outside the game's boundaries
EditorGuidingLines.prototype.updateTime = function(time){
	if(time < 0){
		console.log("can't update to a negative time");
		return;
	}
	
	let ctx = this.canvas.getContext("2d");
	ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
	
	let quarterBeatInterval = this.beatInterval / 4;
	let quarterBeatPixelInterval = this.beatPixelInterval / 4;
	let timeIntoBeat = time % this.beatInterval;
	let timeIntoQuarterBeat = time % quarterBeatInterval;
	// round to next quarter beat interval
	let posOffset = this.groundPosOffset;
	posOffset -= timeIntoQuarterBeat / quarterBeatInterval * quarterBeatPixelInterval;
	let quarterBeatCounter = Math.floor(timeIntoBeat / quarterBeatInterval);
	for(let y = posOffset; y < this.canvas.height; y += quarterBeatPixelInterval){
		if(quarterBeatCounter % 4 == 0){ // beat line
			ctx.fillRect(0, y, this.canvas.width, 3);
		}
		else if(quarterBeatCounter % 4 == 2){ // half beat line
			ctx.fillRect(0, y, this.canvas.width, 1);
		}
		else{ // quarter beat line
			
		}
		++quarterBeatCounter;
	}
}

function Score(){
	this.scoreDiv;
	this.score;
	this.scoreInner;
	
	this.scoreDiv = document.createElement("div");
	this.score = 0;
	this.scoreInner = document.createElement("p");
	
	this.scoreDiv.style.display = "none";
	this.scoreDiv.style.position = "absolute";
	this.scoreDiv.style.top = "0";
	this.scoreDiv.style.right = "0";
	this.scoreDiv.style.height = "3vw";
	this.scoreDiv.style.width = "9vw";
	this.scoreDiv.style.fontSize = "2em";
	this.scoreDiv.style.zIndex = "200";
	this.scoreInner.style.margin = "0";
	this.scoreInner.style.userSelect = "none";
	this.scoreInner.innerHTML = this.score;
	
	this.scoreDiv.appendChild(this.scoreInner);
	document.body.appendChild(this.scoreDiv);
}

function Menu(eventPropagater, controlsMap){
	this.menuDiv;
	this.currentDisplayed;
	this.mainMenu; // each sub div contains an array of selections, each selection contains a select function
	this.controlsMenu;
	
	this.menuDiv = document.createElement("div");
	this.menuDiv.style.display = "none";
	this.menuDiv.style.position = "absolute";
	this.menuDiv.style.top = "10vh";
	this.menuDiv.style.left = "40vw";
	this.menuDiv.style.height = "70vh";
	this.menuDiv.style.width = "20vw";
	this.menuDiv.style.fontSize = "3em";
	this.menuDiv.style.zIndex = "200";
	this.menuDiv.style.backgroundColor = "#9999ff";
	
	this.mainMenu = new MenuPanel();
	this.controlsMenu = new ControlsMenu(controlsMap);
	
	this.mainMenu.addSelection(() => { 
		this.mainMenu.deactivate();
		this.controlsMenu.activate();
		this.currentDisplayed = this.controlsMenu;
	}, "Controls");
	this.mainMenu.addSelection(() => {
		eventPropagater.enableEditor();
	}, "Enable Editor");
	
	this.menuDiv.appendChild(this.mainMenu.domElement());
	this.menuDiv.appendChild(this.controlsMenu.domElement());
}

function MenuPanel(){
	this.div;
	this.eventListener;
	this.selections;
	this.selectionIdx;
	
	this.div = document.createElement("div");
	this.div.style.width = "100%";
	this.div.style.display = "none";
	this.selections = [];
	this.selectionIdx = 0;
	
	this.eventListener = evt => {
		if(evt.keyCode == 38 && this.selectionIdx > 0){ // up arrow
			this.selections[this.selectionIdx].toggleHighlight();
			--this.selectionIdx;
			this.selections[this.selectionIdx].toggleHighlight();
		}
		else if(evt.keyCode == 40 && this.selectionIdx + 1 < this.selections.length){ // down arrow
			this.selections[this.selectionIdx].toggleHighlight();
			++this.selectionIdx;
			this.selections[this.selectionIdx].toggleHighlight();
		}
		else if(evt.keyCode == 13){
			if(this.selections[this.selectionIdx]){
				this.selections[this.selectionIdx].select();
			}
		}
	};
}

function ControlsMenu(controlsMap){
	MenuPanel.call(this);
	this.selectionsNames = [];
	
	let possible_inputs = wasm.Input;
	let num_inputs = Object.keys(possible_inputs).length
	for(let i = 0; i < num_inputs; ++i){
		let name = "";
		let currentKey = "";
		
		let controlSelection;
		
		// find the control name mapping to i
		for (const key in possible_inputs){
			if(possible_inputs[key] == i){
				name = key;
				break;
			}
		}
		
		// find the current key mapping to i
		for (const key in controlsMap){
			if(controlsMap[key] == i){
				currentKey = key;
			}
		}
		
		// create callback for when the selection is selected
		let callback = (keyCode) => {
			let prevKey = null;
			let newKeyName = g_keyCodeNames[keyCode] ? g_keyCodeNames[keyCode] : keyCode;
			for (const key in controlsMap){
				if(controlsMap[key] == i){
					controlsMap[key] = undefined;
					break;
				}
			}
			
			// reset name if a key is rebound
			if(controlsMap[keyCode] != undefined){
				let controlID = controlsMap[keyCode];
				this.setSelectionName(controlID, "");
			}
			controlsMap[keyCode] = i;
			this.setSelectionName(i, newKeyName);
		}
		
		this.selectionsNames.push(name);
		this.addSelection(null, "");
		this.selections[i].setSelectionFunction(() => { changeControlDialog(callback); });
		let currentKeyName = g_keyCodeNames[currentKey] ? g_keyCodeNames[currentKey] : currentKey;
		this.setSelectionName(i, currentKeyName);
	}
}

Object.setPrototypeOf(ControlsMenu.prototype, MenuPanel.prototype);

function MenuSelection(onSelect, selectionText, parentPanelDiv){
	this.div;
	this.selectionText;
	this.onSelect;
	this.highlighted;
	
	this.div = document.createElement("div");
	this.div.style.width = "100%";
	this.div.style.height = "20%";
	
	this.selectionText = document.createElement("p");
	this.selectionText.innerHTML = selectionText;
	this.selectionText.style.margin = "0";
	this.selectionText.style.userSelect = "none";
	this.onSelect = onSelect;
	this.highlighted = false;
	
	this.div.appendChild(this.selectionText);
	parentPanelDiv.appendChild(this.div);
}

MenuPanel.prototype.activate = function(){
	this.selectionIdx = 0;
	document.addEventListener("keydown", this.eventListener);
	this.div.style.display = "block";
}

MenuPanel.prototype.deactivate = function(){
	document.removeEventListener("keydown", this.eventListener);
	this.div.style.display = "none";
}

MenuPanel.prototype.addSelection = function(onSelect, selectionText){
	this.selections.push( new MenuSelection(onSelect, selectionText, this.div) );
	if(this.selections.length == 1){
		this.selections[0].toggleHighlight();
	}
}

MenuPanel.prototype.domElement = function(){
	return this.div;
}

Overlay.prototype.toggleElement = function(elementName){
	if(this[elementName] != undefined){
		this[elementName].toggle();
	}
	else{
		console.log("the overlay does not have member \"" + elementName + "\"");
	}
}

Overlay.prototype.updateScore = function(newScore){
	this.score.update(newScore);
}

Overlay.prototype.updateBeatLines = function(firstBeatY, beatDY){
	
}

Score.prototype.domElement = function(){
	return this.scoreDiv;
}

Score.prototype.toggle = function(){
	if(this.scoreDiv.style.display != "none"){
		this.scoreDiv.style.display = "none";
	}
	else{
		this.scoreDiv.style.display = "block";
	}
}

Score.prototype.update = function(newScore){
	if(newScore != this.score){
		this.score = newScore;
		this.scoreInner.innerHTML = newScore;
	}
}

Menu.prototype.domElement = function(){
	return this.menuDiv;
}

Menu.prototype.toggle = function(){
	if(this.menuDiv.style.display != "none"){
		this.menuDiv.style.display = "none";
		if(this.currentDisplayed){
			this.currentDisplayed.deactivate();
			this.currentDisplayed = null;
		}
	}
	else{
		this.menuDiv.style.display = "block";
		if(this.currentDisplayed){
			this.currentDisplayed.deactivate();
		}
		this.currentDisplayed = this.mainMenu;
		this.mainMenu.activate();
	}
}

ControlsMenu.prototype.setSelectionName = function(selectionID, controlName){
	this.selections[selectionID].setText( this.selectionsNames[selectionID] + " - " + controlName);
}

MenuSelection.prototype.select = function(){
	if(typeof(this.onSelect) == "function"){
		this.onSelect();
	}
}

MenuSelection.prototype.toggleHighlight = function(){
	if(this.highlighted){
		this.div.style.outline = "";
		this.highlighted = false;
	}
	else{
		this.div.style.outline = "4px solid cyan";
		this.highlighted = true;
	}
}

MenuSelection.prototype.setSelectionFunction = function(fn){
	this.onSelect = fn;
}

MenuSelection.prototype.setText = function(txt){
	this.selectionText.innerHTML = txt;
}

MenuSelection.prototype.domElement = function(){
	return this.div;
}

function changeControlDialog(callback){
	let enterKeyDiv = document.createElement("div");
	let enterKeyText = document.createElement("p");
	
	enterKeyDiv.style.position = "absolute";
	enterKeyDiv.style.width = "20vw";
	enterKeyDiv.style.height = "12vw";
	enterKeyDiv.style.left = "40vw";
	enterKeyDiv.style.top = "20vh";
	enterKeyDiv.style.zIndex = "300";
	
	enterKeyText.innerHTML = "Enter Key";
	enterKeyText.fontSize = "3.5em";
	
	enterKeyDiv.appendChild(enterKeyText);
	document.body.appendChild(enterKeyDiv);
	
	document.addEventListener("keydown", evt => {
		evt.preventDefault();
		callback(evt.keyCode);
		enterKeyDiv.remove();
	},{once: true, capture: true});
}