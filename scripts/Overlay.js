
import * as wasm from "../pkg/music_mercenary.js";

let g_keyCodeNames = [];
g_keyCodeNames[32] = "Space";
g_keyCodeNames[13] = "Enter";

// !!! does overlay ever have to be resized?
// TODO having the editor overlay included in game.js Editor class might make more sense
	// would allow game and overlay elements to synchronize more easily
// TODO space key for playing/pausing editor?
// TODO clicking pauses editor?

// main overlay class. all overlay elements are children, directly or nested
export function Overlay(songData, eventPropagator, controlsMap){
	this.overlayDiv;
	this.score;
	this.menu;
	this.editorOverlay;
	this.homeScreen;
	this.endGameScreen;
	
	this.overlayDiv = document.createElement("div");
	this.homeScreen = new HomeScreen(eventPropagator);
	this.endGameScreen = new EndGameScreen(eventPropagator);
	this.score = new Score();
	this.menu = new Menu(eventPropagator, controlsMap);
	this.editorOverlay = new EditorOverlay(songData, eventPropagator);
	
	this.overlayDiv.id = "overlay";
	
	this.overlayDiv.appendChild(this.score.domElement());
	this.overlayDiv.appendChild(this.menu.domElement());
	this.overlayDiv.appendChild(this.editorOverlay.domElement());
	this.overlayDiv.appendChild(this.homeScreen.domElement());
	this.overlayDiv.appendChild(this.endGameScreen.domElement());
	document.getElementById("screen").appendChild(this.overlayDiv);
	
}

// class for the homescreen which holds song selections
function HomeScreen(eventPropagator){
	this.eventPropagator = eventPropagator;
	this.homeScreenDiv;
	this.songSelections = [];
	this.selectionIdx;
	
	this.homeScreenDiv = document.createElement("div");
	this.homeScreenDiv.className = "homescreen";
	this.homeScreenDiv.style.display = "none";
	
	let mmTitle = document.createElement("h1");
	mmTitle.innerHTML = "Music Mercenary";
	this.homeScreenDiv.appendChild(mmTitle);
}

// class for song selections which are attached to the home screen
function HomeSelection(id, name, artist, difficulty, duration){
	this.div;
	this.highlighted;
	
	this.songID;
	this.name;
	this.artist;
	this.difficulty;
	this.duration;
	
	this.songID = id;
	this.name = name;
	this.artist = artist;
	this.difficulty = difficulty;
	this.duration = duration;
	
	this.div = document.createElement("div");
	this.div.className = "home-selection";
	
	this.nameField = document.createElement("p");
	this.artistField = document.createElement("p");
	this.infoField = document.createElement("p");
	
	this.nameField.innerHTML = name;
	this.artistField.innerHTML = artist;
	this.infoField.innerHTML = `Difficulty: ${difficulty} -- Duration: ${duration}`;
	
	this.div.appendChild(this.nameField);
	this.div.appendChild(this.artistField);
	this.div.appendChild(this.infoField);
}

// class for the homescreen which holds song selections
function EndGameScreen(eventPropagator){
	this.endScreenDiv;
	this.eventPropagator;
	this.textDiv;
	this.textElement1;
	this.textElement2;
	this.scoreTextElement;
	this.exitTextElement;
	
	this.endScreenDiv = document.createElement("div");
	this.endScreenDiv.className = "end-game-screen";
	this.endScreenDiv.style.display = "none";
	
	this.textDiv = document.createElement("div");
	this.textDiv.className = "end-game-screen-text-div";
	
	this.textElement1 = document.createElement("p");
	this.textElement2 = document.createElement("p");
	this.scoreTextElement = document.createElement("p");
	this.exitTextElement = document.createElement("p");
	
	this.exitTextElement.innerHTML = "Enter to move on";
	
	this.textDiv.appendChild(this.textElement1);
	this.textDiv.appendChild(this.textElement2);
	this.textDiv.appendChild(this.scoreTextElement);
	this.textDiv.appendChild(this.exitTextElement);
	this.endScreenDiv.appendChild(this.textDiv);
	
	this.eventPropagator = eventPropagator;
}

// EditorOverlay class, contains the editor's guiding lines and the editor's controls
// !!! range scroller isn't modified when the song is modified
function EditorOverlay(songData, eventPropagator){
	this.div;
	this.editorCanvas;
	this.scroller;
	this.controls;
	
	this.div = document.createElement("div");
	this.div.className = "editor-overlay";
	this.div.style.display = "none";
	
	this.editorCanvas = new EditorCanvas(songData, eventPropagator);
	this.controls = new EditorControls(songData, eventPropagator);
	
	this.div.appendChild(this.editorCanvas.domElement());
	this.div.appendChild(this.controls.domElement());
}

// EditorCanvas class, displays lines that (should) represent beat breakpoints in a song
	// clicking on the canvas adds notes, wheel scrolling changes the song time
function EditorCanvas(songData, eventPropagator){
	this.canvas;
	this.beatInterval; // how long between beats in seconds // !!! get from game every time or store as state?
	this.groundPosOffset = wasm.ground_pos();
	this.mouseDown;
	this.changeBrickType;
	this.selectedBrick;
	this.songData;
	this.eventPropagator;
	
	this.mouseDown = false;
	this.changeBrickType = false;
	this.selectedBrick = null;
	this.songData = songData;
	this.eventPropagator = eventPropagator;
	
	let dims = wasm.game_dimensions();
	this.canvas = document.createElement("canvas");
	this.canvas.width = dims.x;
	this.canvas.height = dims.y;
	this.canvas.className = "full-sized";
	
	this.updateSongData(songData);
	this.canvas.addEventListener("mousedown", evt => { this.handleMouseDown(evt); });
	this.canvas.addEventListener("mouseup", evt => { this.handleMouseUp(evt); });
	this.canvas.addEventListener("mousemove", evt => { this.handleMouseMove(evt); });
	this.canvas.addEventListener("wheel", evt => { this.handleWheel(evt); });
	
}

// EditorControls class, contains controls which can control EditorCanvas scrolling and game playing/pausing
// TODO add triplet note button to convert editor canvas's selected brick to triplet note
function EditorControls(songData, eventPropagator){
	this.div;
	this.rangesDiv;
	this.buttonDiv;
	this.broadRange;
	this.preciseRange;
	this.playPauseButton;
	this.songDuration;
	this.beatInterval;
	
	this.songDuration = songData.gameData.duration;
	this.beatInterval = songData.gameData.beat_interval;
	
	this.div = document.createElement("div");
	this.div.className = "editor-controls";
	
	this.rangesDiv = document.createElement("div");
	this.rangesDiv.className = "editor-ranges-div";
	
	this.buttonDiv = document.createElement("div");
	this.buttonDiv.className = "editor-buttons-div";
	
	this.broadRange = document.createElement("input");
	this.broadRange.className = "editor-broad-range";
	this.broadRange.type = "range";
	this.broadRange.max = 100;
	this.broadRange.step = 0.1;
	this.broadRange.value = 0;
	
	this.preciseRange = document.createElement("input");
	this.preciseRange.className = "editor-precise-range";
	this.preciseRange.type = "range";
	this.preciseRange.max = 4;
	this.preciseRange.step = 0.05;
	this.preciseRange.value = 0;
	
	this.playPauseButton = document.createElement("button");
	this.playPauseButton.innerHTML = "|> / ||";
	
	this.rangesDiv.addEventListener("input", evt => {
        let t = parseFloat(this.broadRange.value) / 100 * this.songDuration 
			+ parseFloat(this.preciseRange.value) * this.beatInterval;
			
		let fn = game => {
			game.seek(t);
		}
		
		eventPropagator.runOnGame(fn, true);
    });
	
	this.playPauseButton.addEventListener("click", evt => {
		// prevent artificial clicks
		if(evt.clientX != 0 || evt.clientY != 0) {
			eventPropagator.togglePlay();
		}
	});
	
	this.rangesDiv.appendChild(this.preciseRange);
	this.rangesDiv.appendChild(this.broadRange);
	this.buttonDiv.appendChild(this.playPauseButton);
	this.div.appendChild(this.rangesDiv);
	this.div.appendChild(this.buttonDiv);
}

// Score class, displays a score which may be updated
function Score(){
	this.scoreDiv;
	this.score;
	this.scoreInner;
	
	this.scoreDiv = document.createElement("div");
	this.scoreDiv.className = "score";
	this.scoreDiv.style.display = "none";
	
	this.score = 0;
	this.scoreInner = document.createElement("p");
	
	this.scoreInner.innerHTML = this.score;
	
	this.scoreDiv.appendChild(this.scoreInner);
	document.body.appendChild(this.scoreDiv);
}

// Menu class, contains the menu div on which menu panels are displayed and contains logic of menu panels and their selections
function Menu(eventPropagator, controlsMap){
	this.menuDiv;
	this.currentDisplayed;
	this.mainMenu; // each sub div contains an array of selections, each selection contains a select function
	this.controlsMenu;
	this.saveLoadMenu;
	
	this.menuDiv = document.createElement("div");
	this.menuDiv.style.display = "none";
	this.menuDiv.className = "menu";
	
	this.homeMenu = new MenuPanel();
	this.gameMenu = new MenuPanel();
	this.masterGameMenu = new MenuPanel();
	this.controlsMenu = new MenuPanel();
	this.saveLoadMenu = new MenuPanel();
	
	// --- home menu selections ---
	
	this.homeMenu.addSelection(() => { 
		this.currentDisplayed.hide();
		this.controlsMenu.show();
		this.currentDisplayed = this.controlsMenu;
	}, "Controls");
	
	// --- game menu selections ---
	
	this.gameMenu.addSelection(() => { 
		this.currentDisplayed.hide();
		this.controlsMenu.show();
		this.currentDisplayed = this.controlsMenu;
	}, "Controls");
	
	this.gameMenu.addSelection(() => {
		let fn = game => { game.restart() }
		eventPropagator.runOnGame(fn, true);
	}, "Restart song");
	
	this.gameMenu.addSelection(() => {
		eventPropagator.exitToHomeScreen();
	}, "Quit song");
	
	// --- master game menu selections ---
	
	this.masterGameMenu.addSelection(() => { 
		this.currentDisplayed.hide();
		this.controlsMenu.show();
		this.currentDisplayed = this.controlsMenu;
	}, "Controls");
	
	this.masterGameMenu.addSelection(() => {
		let fn = game => { game.restart() }
		eventPropagator.runOnGame(fn, true);
	}, "Restart song");
	
	this.masterGameMenu.addSelection(() => {
		eventPropagator.exitToHomeScreen();
	}, "Quit song");
	
	this.masterGameMenu.addSelection(() => {
		eventPropagator.enableEditor();
	}, "Enable Editor");
	
	this.masterGameMenu.addSelection(() => {
		eventPropagator.disableEditor();
	}, "Disable Editor");
	
	this.masterGameMenu.addSelection(() => {
		this.currentDisplayed.hide();
		this.saveLoadMenu.show();
		this.currentDisplayed = this.saveLoadMenu;
	}, "Save/Load");
	
	// --- save/load menu selections ---
	
	this.saveLoadMenu.addSelection(() => {
		saveSongDialog(eventPropagator);
	}, "Save song");
	
	this.saveLoadMenu.addSelection(() => {
		loadSongDialog(eventPropagator);
	}, "Load song");
	
	this.saveLoadMenu.addSelection(() => {
		newSongDialog(eventPropagator);
	}, "New song");
	
	this.saveLoadMenu.addSelection(() => {
		uploadMP3Dialog(eventPropagator);
	}, "Load mp3");
	
	this.saveLoadMenu.addSelection(() => {
		alert("Not yet implemented"); // !!! load database
	}, "Load database");
	
	// --- controls menu selections ---
	
	// add a selection for each possible input
	let possible_inputs = wasm.Input;
	let num_inputs = Object.keys(possible_inputs).length
	for(let i = 0; i < num_inputs; ++i){
		let inputName = "";
		let defaultKey = "";
		
		// find the control name mapping to input i (Dash, Slash1, etc.)
		for (const key in possible_inputs){
			if(possible_inputs[key] == i){
				inputName = key;
				break;
			}
		}
		
		// find the default key mapping to input i
		for (const key in controlsMap){
			if(controlsMap[key] == i){
				defaultKey = key;
			}
		}
		
		// add the selection
		this.controlsMenu.addSelection(() => { 
			changeControlDialog(controlsMap, this.controlsMenu, i)
		}, inputName);
		
		// set the selection name
		let defaultKeyName = g_keyCodeNames[defaultKey] ? g_keyCodeNames[defaultKey] : String.fromCharCode(defaultKey);
		this.controlsMenu.setSelectionText(i,  inputName + " - " + defaultKeyName);
	}
	
	this.menuDiv.appendChild(this.homeMenu.domElement());
	this.menuDiv.appendChild(this.gameMenu.domElement());
	this.menuDiv.appendChild(this.masterGameMenu.domElement());
	this.menuDiv.appendChild(this.controlsMenu.domElement());
	this.menuDiv.appendChild(this.saveLoadMenu.domElement());
}

// MenuPanel class, contains panel selections and handles up and down keypresses
// !!! add support for hiding buttons (making navigation ignore inactive buttons)
	// !!! once done, make disable editor and enable editor buttons mutually exclusive / non buggy
function MenuPanel(){
	this.div;
	this.selections;
	this.selectionIdx;
	
	this.div = document.createElement("div");
	this.div.className = "menu-panel";
	this.div.style.display = "none";
	this.selections = [];
	this.selectionIdx = 0;
}

// MenuSelection class. Controls highlighting style and the selection function
function MenuSelection(onSelect, value, parentPanelDiv){
	this.div;
	this.value;
	this.selectionText;
	this.onSelect;
	this.highlighted;
	
	this.div = document.createElement("div");
	this.div.className = "menu-selection";
	
	this.value = value;
	
	this.selectionText = document.createElement("p");
	this.selectionText.innerHTML = value;
	
	this.onSelect = onSelect;
	this.highlighted = false;
	
	this.div.appendChild(this.selectionText);
	parentPanelDiv.appendChild(this.div);
}

Overlay.prototype.showElement = function(elementName){
	if(this[elementName] != undefined){
		this[elementName].show();
	}
	else{
		console.log("the overlay does not have member \"" + elementName + "\"");
	}
}

Overlay.prototype.hideElement = function(elementName){
	if(this[elementName] != undefined){
		this[elementName].hide();
	}
	else{
		console.log("the overlay does not have member \"" + elementName + "\"");
	}
}

Overlay.prototype.populateMenu = function(mode){
	this.menu.populate(mode);
}

Overlay.prototype.isElementShowing = function(elementName){
	if(this[elementName] != undefined && typeof this[elementName].domElement == "function"){
		return this[elementName].domElement().style.display == "block";
	}
}

Overlay.prototype.passEvent = function(elementName, evt){
	if(this[elementName] != undefined && typeof this[elementName].handleEvent == "function"){
		this[elementName].handleEvent(evt);
	}
}

Overlay.prototype.updateSongData = function(songData){
	this.editorOverlay.updateSongData(songData);
	this.updateScore(songData.score);
}

Overlay.prototype.updateScore = function(newScore){
	this.score.update(newScore);
}

HomeScreen.prototype.show = function(){
	let retrieveSongs = game => {
		return game.songs();
	}
	let songs = this.eventPropagator.runOnGame(retrieveSongs);
	this.populateSelections(songs);
	
	this.homeScreenDiv.style.display = "block";
}

HomeScreen.prototype.hide = function(){
	this.homeScreenDiv.style.display = "none";
}

HomeScreen.prototype.domElement = function(){
	return this.homeScreenDiv;
}

HomeScreen.prototype.handleEvent = function(evt){
	if(evt.keyCode == 38 && this.selectionIdx > 0){ // up arrow
		this.songSelections[this.selectionIdx].toggleHighlight();
		--this.selectionIdx;
		this.songSelections[this.selectionIdx].toggleHighlight();
	}
	else if(evt.keyCode == 40 && this.selectionIdx + 1 < this.songSelections.length){ // down arrow
		this.songSelections[this.selectionIdx].toggleHighlight();
		++this.selectionIdx;
		this.songSelections[this.selectionIdx].toggleHighlight();
	}
	else if(evt.keyCode == 13){ // enter
		if(this.songSelections[this.selectionIdx]){
			let songID = this.songSelections[this.selectionIdx].getSongID();
			let fn = game => game.loadSong(songID);
			this.eventPropagator.runOnGame(fn);
			this.eventPropagator.start();
		}
	}
}

HomeScreen.prototype.populateSelections = function(songs){
	for(let i = 0; i < this.songSelections.length; ++i){
		this.songSelections[i].domElement().remove();
	}
	
	this.songSelections = [];
	
	let idIDX;
	let nameIDX;
	let artistIDX;
	let difficultyIDX;
	let durationIDX;
	let timeCreatedIDX;
	let timeModifiedIDX;
	
	if(songs.length != 0){
		songs[0]["columns"].forEach( (columnName, idx) => {
			if(columnName.toUpperCase() === "SONGID"){
				idIDX = idx;
			}
			else if(columnName.toUpperCase() === "NAME"){
				nameIDX = idx;
			}
			else if(columnName.toUpperCase() === "ARTIST"){
				artistIDX = idx;
			}
			else if(columnName.toUpperCase() === "DIFFICULTY"){
				difficultyIDX = idx;
			}
			else if(columnName.toUpperCase() === "DURATION"){
				durationIDX = idx;
			}
			else if(columnName.toUpperCase() === "TIMECREATED"){
				timeCreatedIDX = idx;
			}
			else if(columnName.toUpperCase() === "TIMEMODIFIED"){
				timeModifiedIDX = idx;
			}
		});
		
		songs[0]["values"].forEach( (song, idx) => {
			if(song[idIDX] == 6 && song[nameIDX].toUpperCase() === "WAKE ME UP"){
				let selection = new HomeSelection(song[idIDX], song[nameIDX], song[artistIDX], song[difficultyIDX], song[durationIDX]);
				this.songSelections.push(selection);
				this.homeScreenDiv.appendChild(selection.domElement());
			}
			if(song[idIDX] == 7 && song[nameIDX].toUpperCase() === "ANIMALS"){
				let selection = new HomeSelection(song[idIDX], song[nameIDX], song[artistIDX], song[difficultyIDX], song[durationIDX]);
				this.songSelections.push(selection);
				this.homeScreenDiv.appendChild(selection.domElement());
			}
		});
	}
	
	if(this.songSelections.length > 0){
		this.selectionIdx = 0;
		this.songSelections[0].toggleHighlight();
	}
}

HomeSelection.prototype.domElement = function(){
	return this.div;
}

HomeSelection.prototype.toggleHighlight = function(){
	if(this.highlighted){
		this.div.className = "home-selection";
		this.highlighted = false;
	}
	else{
		this.div.className = "home-selection highlighted-home-selection";
		this.highlighted = true;
	}
}

HomeSelection.prototype.getSongID = function(){
	return this.songID;
}

EndGameScreen.prototype.domElement = function(){
	return this.endScreenDiv;
}

EndGameScreen.prototype.show = function(){
	let songData = this.eventPropagator.runOnGame( game => {
		return game.getSongData();
	});
	this.textElement1.innerHTML = songData.name + " - " + songData.artist;
	this.textElement2.innerHTML = "Difficulty: "+ songData.difficulty + " --- time: " + songData.duration;
	this.scoreTextElement.innerHTML = "Score: " + songData.gameData.score + " / " + songData.gameData.max_score;
	
	this.endScreenDiv.style.display = "block";
}

EndGameScreen.prototype.hide = function(){
	this.endScreenDiv.style.display = "none";
}

EndGameScreen.prototype.handleEvent = function(evt){
	if(evt.keyCode == 13){ // enter
		this.eventPropagator.exitToHomeScreen();
	}
}

EditorOverlay.prototype.show = function(){
	this.div.style.display = "block";
}

EditorOverlay.prototype.hide = function(){
	this.div.style.display = "none";
}

EditorOverlay.prototype.updateSongData = function(songData){
	if(this.div.style.display != "none"){
		this.editorCanvas.updateSongData(songData);
		this.controls.updateSongData(songData);
	}
}

EditorOverlay.prototype.domElement = function(){
	return this.div;
}

EditorOverlay.prototype.handleEvent = function(evt){
	this.editorCanvas.handleKeyDown(evt);
}

EditorCanvas.prototype.domElement = function(){
	return this.canvas;
}

EditorCanvas.prototype.timeToY = function(time){
	let timeDifference = time - this.songData.gameData.time_running;
	let currentY = wasm.ground_pos() - wasm.player_dimensions().y / 2;
	let newY = currentY + timeDifference * this.songData.gameData.brick_speed;
	return newY;
}

EditorCanvas.prototype.yToTime = function(y){
	let currentY = wasm.ground_pos() - wasm.player_dimensions().y / 2;
	let yDifference = y - currentY;
	let timeDifference = yDifference / this.songData.gameData.brick_speed;
	
	return this.songData.gameData.time_running + timeDifference;
}

EditorCanvas.prototype.xToNotePos = function(x){
	return Math.floor(x / wasm.brick_dimensions().x);
}

EditorCanvas.prototype.notePosToX = function(notePos){
	return x * wasm.brick_dimensions().x;
}

EditorCanvas.prototype.updateSongData = function(songData){
	this.songData = songData;
	
	this.draw();
}

EditorCanvas.prototype.draw = function(){
	let songData = this.songData;
	let ctx = this.canvas.getContext("2d");
	ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
	
	let time = songData.gameData.time_running;
	
	// get the beat positions that mark when lines should start and stop being drawn
	let topScreenTime = time - (wasm.ground_pos() / songData.gameData.brick_speed);
	let bottomScreenTime = time + (wasm.game_dimensions().y / songData.gameData.brick_speed);
	let beginningBeatPos = wasm.BrickData.closest_beat_pos(topScreenTime, songData.gameData.bpm);
	let endBeatPos = wasm.BrickData.closest_beat_pos(bottomScreenTime, songData.gameData.bpm);
	let screenWidth = wasm.game_dimensions().x;
	
	// draw lines at the specified beat positions
	for(let i = beginningBeatPos; i < endBeatPos; ++i) {
		let beatTime = wasm.BrickData.approx_time(i, songData.gameData.bpm);
		let y = this.timeToY(beatTime);
		if(i % 4 == 0){
			ctx.fillRect(0, y-1, this.canvas.width, 3);
			for(let x = 0; x <= screenWidth; x+=wasm.brick_dimensions().x){
				ctx.fillRect(x-1, y-4, 3, 8);
			}
		} else if(i % 2 == 0){
			ctx.fillRect(0, y, this.canvas.width, 1);
		} else {
			ctx.beginPath();
			ctx.setLineDash([6, 16]);
			ctx.lineWidth = 1;
			ctx.moveTo(0, y);
			ctx.lineTo(this.canvas.width, y);
			ctx.stroke();
		}
	}
	
	// draw a highlight box around the selected brick
	if(this.selectedBrick){
		let brickDims = wasm.brick_dimensions();
		let startX = this.selectedBrick.x_pos * brickDims.x;
		let startTime = wasm.BrickData.approx_time(this.selectedBrick.beat_pos, songData.gameData.bpm);
		let endTime = wasm.BrickData.approx_time(this.selectedBrick.end_beat_pos, songData.gameData.bpm);
		let startY = this.timeToY(startTime) - brickDims.y / 2;
		let endY = this.timeToY(endTime) + brickDims.y / 2;
		if(this.selectedBrick.is_leading || this.selectedBrick.is_trailing){
			let minutesPerBeat = 1 / songData.gameData.bpm;
			let secondsPerBeat = 60 * minutesPerBeat;
			let secondsPerEighthBeat = secondsPerBeat / 8;
			let difference = secondsPerEighthBeat * songData.gameData.brick_speed;
			
			if(this.selectedBrick.is_leading){
				startY -= difference;
				if(!this.selectedBrick.is_hold_note){
					endY -= difference;
				}
			}
			if(this.selectedBrick.is_trailing){
				startY += difference;
				if(!this.selectedBrick.is_hold_note){
					endY += difference;
				}
			}
		}
		
		ctx.setLineDash([]);
		ctx.lineWidth = 3;
		ctx.strokeRect(startX, startY, brickDims.x, endY - startY);
	}
}

EditorCanvas.prototype.handleKeyDown = function(evt){
	if(evt.keyCode == 32){ // space
		this.eventPropagator.togglePlay();
	}
	
	if(this.selectedBrick){
		let brick = this.selectedBrick;
		
		if(evt.keyCode == 46 || evt.keyCode == 8) { // delete or backspace
			this.eventPropagator.runOnGame(game => { game.removeBrick(brick.brick_type, brick.beat_pos, brick.end_beat_pos, brick.x_pos, 
				brick.is_triplet, brick.is_trailing, brick.is_leading, brick.is_hold_note); });
			this.selectedBrick = null;
		}
		
		if(evt.keyCode == 38){ // up arrow. delete the old brick, move the brick up and recreate
			this.eventPropagator.runOnGame(game => { game.removeBrick(brick.brick_type, brick.beat_pos, brick.end_beat_pos, brick.x_pos, 
				brick.is_triplet, brick.is_trailing, brick.is_leading, brick.is_hold_note); });
			
			if(brick.is_trailing){
				brick.is_trailing = false;
			} else if(brick.is_leading){
				brick.is_leading = false;
				brick.beat_pos -= 1;
				brick.end_beat_pos -= 1;
			} else {
				brick.is_leading = true;
			}
			
			this.eventPropagator.runOnGame(game => { game.createBrick(brick.brick_type, brick.beat_pos, brick.end_beat_pos, brick.x_pos, 
				brick.is_triplet, brick.is_trailing, brick.is_leading, brick.is_hold_note); });
			this.selectedBrick = this.eventPropagator.runOnGame( game => { return game.selectBrick(brick.beat_pos, brick.x_pos); });
		}
		
		if(evt.keyCode == 40){ // down arrow. delete the old brick, move the brick down and recreate
			this.eventPropagator.runOnGame(game => { game.removeBrick(brick.brick_type, brick.beat_pos, brick.end_beat_pos, brick.x_pos, 
				brick.is_triplet, brick.is_trailing, brick.is_leading, brick.is_hold_note); });
				
			if(brick.is_leading){
				brick.is_leading = false;
			} else if(brick.is_trailing){
				brick.is_trailing = false;
				brick.beat_pos += 1;
				brick.end_beat_pos += 1;
			} else {
				brick.is_trailing = true;
			}
			
			this.eventPropagator.runOnGame(game => { game.createBrick(brick.brick_type, brick.beat_pos, brick.end_beat_pos, brick.x_pos, 
				brick.is_triplet, brick.is_trailing, brick.is_leading, brick.is_hold_note); });
			this.selectedBrick = this.eventPropagator.runOnGame( game => { return game.selectBrick(brick.beat_pos, brick.x_pos); });
		}
		
		if(evt.keyCode == 37){ // left arrow. delete the old brick, move the brick to the left and recreate
			this.eventPropagator.runOnGame(game => { game.removeBrick(brick.brick_type, brick.beat_pos, brick.end_beat_pos, brick.x_pos, 
				brick.is_triplet, brick.is_trailing, brick.is_leading, brick.is_hold_note); });
				
			brick.x_pos -= brick.x_pos > 0 ? 1 : 0;
			
			this.eventPropagator.runOnGame(game => { game.createBrick(brick.brick_type, brick.beat_pos, brick.end_beat_pos, brick.x_pos, 
				brick.is_triplet, brick.is_trailing, brick.is_leading, brick.is_hold_note); });
			this.selectedBrick = this.eventPropagator.runOnGame( game => { return game.selectBrick(brick.beat_pos, brick.x_pos); });
		}
		
		if(evt.keyCode == 39){ // right arrow. delete the old brick, move the brick to the right and recreate
			this.eventPropagator.runOnGame(game => { game.removeBrick(brick.brick_type, brick.beat_pos, brick.end_beat_pos, brick.x_pos, 
				brick.is_triplet, brick.is_trailing, brick.is_leading, brick.is_hold_note); });
				
			brick.x_pos += brick.x_pos + 1 < wasm.max_notes_per_screen_width() ? 1 : 0;
			
			this.eventPropagator.runOnGame(game => { game.createBrick(brick.brick_type, brick.beat_pos, brick.end_beat_pos, brick.x_pos, 
				brick.is_triplet, brick.is_trailing, brick.is_leading, brick.is_hold_note); });
			this.selectedBrick = this.eventPropagator.runOnGame( game => { return game.selectBrick(brick.beat_pos, brick.x_pos); });
		}
	}
	
	this.draw();
}

EditorCanvas.prototype.handleMouseDown = function(evt){
	let x = evt.clientX - this.canvas.offsetLeft;
	let y = evt.clientY - this.canvas.offsetTop;
	
	let dimFactors = this.eventPropagator.runOnGame( game => { return game.dimensionFactors(); } );
	x = x / dimFactors.xFactor;
	y = y / dimFactors.yFactor;
	let approxTime = this.yToTime(y);
	let xPos = this.xToNotePos(x);
	let beatPos = wasm.BrickData.closest_beat_pos(approxTime, this.songData.gameData.bpm);
	
	let brick = this.eventPropagator.runOnGame( game => { return game.selectBrick(beatPos, xPos); } );
	
	if(brick){
		// if clicking on the selected brick, remove the selected brick and recreate it with new brick type
		if(this.selectedBrick && this.selectedBrick.beat_pos == brick.beat_pos && this.selectedBrick.x_pos == brick.x_pos
		&& this.selectedBrick.is_leading == brick.is_leading && this.selectedBrick.is_trailing == brick.is_trailing
		&& this.selectedBrick.is_triplet == brick.is_triplet){
			this.changeBrickType = true;
		}
	} else {
		this.eventPropagator.runOnGame( game => { game.createDefaultBrick(beatPos, xPos); } );
		brick = this.eventPropagator.runOnGame( game => { return game.selectBrick(beatPos, xPos); } );
	}
	
	this.selectedBrick = brick;
	
	this.mouseDown = true;
	this.draw();
}

EditorCanvas.prototype.handleMouseUp = function(evt){
	this.mouseDown = false;
	if(this.changeBrickType){
		let brick = this.selectedBrick;
		this.eventPropagator.runOnGame(game => { game.removeBrick(brick.brick_type, brick.beat_pos, brick.end_beat_pos, brick.x_pos, 
			brick.is_triplet, brick.is_trailing, brick.is_leading, brick.is_hold_note); });
		
		if(brick.brick_type < 2){
			brick.brick_type += 1;
		} else {
			brick.brick_type = 0;
		}
		
		this.eventPropagator.runOnGame(game => { game.createBrick(brick.brick_type, brick.beat_pos, brick.end_beat_pos, brick.x_pos, 
			brick.is_triplet, brick.is_trailing, brick.is_leading, brick.is_hold_note); });
		this.selectedBrick = this.eventPropagator.runOnGame( game => { return game.selectBrick(brick.beat_pos, brick.x_pos); } );
	}
	this.draw();
}
	
EditorCanvas.prototype.handleMouseMove = function(evt){
	if(this.mouseDown && this.selectedBrick){
		let x = evt.clientX - this.canvas.offsetLeft;
		let y = evt.clientY - this.canvas.offsetTop;
		
		let dimFactors = this.eventPropagator.runOnGame( game => { return game.dimensionFactors(); } );
		x = x / dimFactors.xFactor;
		y = y / dimFactors.yFactor;
		let approxTime = this.yToTime(y);
		let xPos = this.xToNotePos(x);
		let beatPos = wasm.BrickData.closest_beat_pos(approxTime, this.songData.gameData.bpm);
		
		if(beatPos != this.selectedBrick.end_beat_pos || xPos != this.selectedBrick.x_pos){
			this.changeBrickType = false;
			
			let brick = this.selectedBrick;
			this.eventPropagator.runOnGame(game => { game.removeBrick(brick.brick_type, brick.beat_pos, brick.end_beat_pos, brick.x_pos, 
				brick.is_triplet, brick.is_trailing, brick.is_leading, brick.is_hold_note); });
			
			if(beatPos > brick.beat_pos){
				brick.is_hold_note = true;
				brick.end_beat_pos = beatPos;
			} else {
				brick.is_hold_note = false;
				brick.end_beat_pos = brick.beat_pos;
			}
			
			if(xPos != brick.x_pos){
				brick.x_pos = xPos;
			}
			
			this.eventPropagator.runOnGame(game => { game.createBrick(brick.brick_type, brick.beat_pos, brick.end_beat_pos, brick.x_pos, 
				brick.is_triplet, brick.is_trailing, brick.is_leading, brick.is_hold_note); });
				
			this.selectedBrick = this.eventPropagator.runOnGame( game => { return game.selectBrick(beatPos, xPos); });
		}
	}
	this.draw();
}

EditorCanvas.prototype.handleWheel = function(evt){
	let time;
	let getTime = game => {
		return game.getSongData().gameData.time_running;
	}
	let updateTime = game => {
		game.seek(time);
	}
	
	time = this.eventPropagator.runOnGame(getTime);
	time += evt.deltaY / 256;
	
	this.eventPropagator.runOnGame(updateTime, true);
}

EditorControls.prototype.domElement = function(){
	return this.div;
}

EditorControls.prototype.updateSongData = function(songData){
	let time = songData.gameData.time_running;
	let prevT = parseFloat(this.broadRange.value) / 100 * this.songDuration 
		+ parseFloat(this.preciseRange.value) * this.beatInterval;
	if(time - prevT > 0.5 || prevT - time > 0.5){
		this.broadRange.value = time / this.songDuration * 100;
		this.preciseRange.value = 0;
	}
}

MenuPanel.prototype.show = function(){
	if(this.selectionIdx != 0){
		this.selections[this.selectionIdx].toggleHighlight();
		this.selections[0].toggleHighlight();
		this.selectionIdx = 0;
	}
	this.div.style.display = "block";
}

MenuPanel.prototype.hide = function(){
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

MenuPanel.prototype.handleEvent = function(evt){
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
	else if(evt.keyCode == 13){ // enter
		if(this.selections[this.selectionIdx]){
			this.selections[this.selectionIdx].select();
		}
	}
}

Score.prototype.domElement = function(){
	return this.scoreDiv;
}

Score.prototype.show = function(){
	this.scoreDiv.style.display = "block";
}

Score.prototype.hide = function(){
	this.scoreDiv.style.display = "none";
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

Menu.prototype.show = function(){
	this.menuDiv.style.display = "block";
	if(this.currentDisplayed){
		this.currentDisplayed.hide();
	}
	this.currentDisplayed = null;
}

Menu.prototype.hide = function(){
	this.menuDiv.style.display = "none";
	if(this.currentDisplayed){
		this.currentDisplayed.hide();
		this.currentDisplayed = null;
	}
}

Menu.prototype.populate = function(menuPanelName){
	if(this.currentDisplayed){
		this.currentDisplayed.hide();
	}
	
	if(this[menuPanelName] instanceof MenuPanel){
		this[menuPanelName].show();
		this.currentDisplayed = this[menuPanelName];
	}
}

Menu.prototype.handleEvent = function(evt){
	this.currentDisplayed.handleEvent(evt);
}

MenuPanel.prototype.setSelectionText = function(selectionID, newText){
	this.selections[selectionID].setText( newText );
}

MenuPanel.prototype.getSelectionValue = function(selectionID){
	if(this.selections[selectionID]){
		return this.selections[selectionID].getValue();
	}
}

MenuSelection.prototype.select = function(){
	if(typeof(this.onSelect) == "function"){
		this.onSelect();
	}
}

MenuSelection.prototype.toggleHighlight = function(){
	if(this.highlighted){
		this.div.className = "menu-selection";
		this.highlighted = false;
	}
	else{
		this.div.className = "highlighted-menu-selection menu-selection";
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

MenuSelection.prototype.getValue = function(){
	return this.value;
}

function changeControlDialog(controlsMap, controlsMenu, inputID){
	let enterKeyDiv = document.createElement("div");
	let enterKeyText = document.createElement("p");
	
	enterKeyDiv.className = "change-control-dialog";
	
	enterKeyText.innerHTML = "Enter Key";
	enterKeyText.fontSize = "3.5em";
	
	enterKeyDiv.appendChild(enterKeyText);
	document.body.appendChild(enterKeyDiv);
	
	// !!! !!! !!! preventDefault() and {capture: true} do not prevent event from being sent to other event handlers. 
		// Way to make this temporarily the only even handler for key presses?
		// Solution:
			// eventPropagator flag when a dialog is launched (changeControlDialog, loadSongDialog, etc.)
			// prevents events until the dialog is closed via escape or otherwise
	document.addEventListener("keydown", evt => {
		if(evt.keyCode != 27){
			let keyCode = evt.keyCode;
		
			let newKeyName = g_keyCodeNames[keyCode] ? g_keyCodeNames[keyCode] : (evt.key).toUpperCase();
			
			// unbind the previous key for this input
			for (const key in controlsMap){
				if(controlsMap[key] == inputID){
					controlsMap[key] = undefined;
					break;
				}
			}
			
			// if the new key is mapped to a different input, set that input to have no key mapping to it
			if(controlsMap[keyCode] != undefined){
				let controlID = controlsMap[keyCode];
				let controlName = controlsMenu.getSelectionValue(controlID);
				controlsMenu.setSelectionText(controlID, "" + controlName + " - UNBOUND");
			}
			
			controlsMap[keyCode] = inputID;
			
			let inputName = controlsMenu.getSelectionValue(inputID);
			controlsMenu.setSelectionText(inputID, inputName + " - " + newKeyName);
		}
		
		enterKeyDiv.remove();
	},{once: true});
}

// TODO rename or accept fields
function newSongDialog(eventPropagator){
	
	let songData = eventPropagator.runOnGame( game => {
		return game.getSongData();
	})
	let bpm = songData.gameData.bpm;
	let songStartOffset = songData.startOffset;
	let brickSpeed = songData.gameData.brickSpeed;
	let duration = songData.duration;
	
	eventPropagator.runOnGame( game => {
		game.newSong(bpm, brickSpeed, duration, songStartOffset);
	}, true);
}

// TODO less ugly, this and loadSongDialog
function saveSongDialog(eventPropagator){
	let div = document.createElement("div");
	let saveButton = document.createElement("button");
	let overwriteButton = document.createElement("button");
	let nameLabel = document.createElement("label");
	let nameField = document.createElement("input");
	let artistLabel = document.createElement("label");
	let artistField = document.createElement("input");
	let difficultyLabel = document.createElement("label");
	let difficultyField = document.createElement("input");
	let bpmLabel = document.createElement("label");
	let bpmField = document.createElement("input");
	let brickSpeedLabel = document.createElement("label"); // !!! modify brick speed in editor
	let brickSpeedField = document.createElement("input");
	let durationLabel = document.createElement("label");
	let durationField = document.createElement("input");
	let startOffsetLabel = document.createElement("label");
	let startOffsetField = document.createElement("input");
	let filenameLabel = document.createElement("label");
	let filenameField = document.createElement("input");
	let newLine = () => { return document.createElement("br"); }
	
	div.className = "extra-dialog";
	
	nameLabel.innerHTML = "Song name";
	artistLabel.innerHTML = "Artist";
	difficultyLabel.innerHTML = "Difficulty";
	bpmLabel.innerHTML = "BPM";
	brickSpeedLabel.innerHTML = "Brick Speed";
	durationLabel.innerHTML = "Song Duration";
	startOffsetLabel.innerHTML = "Start Offset";
	filenameLabel.innerHTML = "Song file name";
	
	let songData = eventPropagator.runOnGame(game => {
		return game.getSongData();
	});
	
	nameField.type = "text";
	nameField.defaultValue = songData.name;
	artistField.type = "text";
	artistField.defaultValue = songData.artist;
	difficultyField.type = "text";
	difficultyField.defaultValue = songData.difficulty;
	bpmField.type = "text";
	bpmField.defaultValue = songData.gameData.bpm;
	brickSpeedField.type = "text";
	brickSpeedField.defaultValue = songData.gameData.brick_speed;
	durationField.type = "text";
	durationField.defaultValue = songData.duration;
	startOffsetField.type = "text";
	startOffsetField.defaultValue = songData.startOffset;
	filenameField.type = "text";
	filenameField.defaultValue = songData.filename;
	
	saveButton.innerHTML = "Save";
	overwriteButton.innerHTML = "Overwrite";
	
	div.appendChild(nameLabel);
	div.appendChild(nameField);
	div.appendChild(newLine());
	div.appendChild(artistLabel);
	div.appendChild(artistField);
	div.appendChild(newLine());
	div.appendChild(difficultyLabel);
	div.appendChild(difficultyField);
	div.appendChild(newLine());
	div.appendChild(bpmLabel);
	div.appendChild(bpmField);
	div.appendChild(newLine());
	div.appendChild(brickSpeedLabel);
	div.appendChild(brickSpeedField);
	div.appendChild(newLine());
	div.appendChild(durationLabel);
	div.appendChild(durationField);
	div.appendChild(newLine());
	div.appendChild(startOffsetLabel);
	div.appendChild(startOffsetField);
	div.appendChild(newLine());
	div.appendChild(filenameLabel);
	div.appendChild(filenameField);
	div.appendChild(newLine());
	div.appendChild(saveButton);
	div.appendChild(overwriteButton);
	
	document.body.appendChild(div);
	
	let submitForm = evt => {
		document.removeEventListener("keydown", pressKey);
		saveButton.removeEventListener("click", submitForm);
		overwriteButton.removeEventListener("click", submitForm);
		
		let overwrite = (evt.target == overwriteButton) ? true : false;
		
		songData.name = nameField.value,
		songData.artist = artistField.value,
		songData.difficulty = difficultyField.value,
		songData.bpm = bpmField.value,
		songData.brickSpeed = brickSpeedField.value,
		songData.duration = durationField.value,
		songData.startOffset = startOffsetField.value,
		songData.filename = filenameField.value
		
		let fn = game => {
			game.saveSong(songData, overwrite);
		}
		eventPropagator.runOnGame(fn);
		
		div.remove();
	};
	let pressKey = evt => {
		if(evt.keyCode == 27){
			document.removeEventListener("keydown", pressKey);
			saveButton.removeEventListener("click", submitForm);
			overwriteButton.removeEventListener("click", submitForm);
			
			div.remove();
		}
	}
	
	saveButton.addEventListener("click", submitForm);
	overwriteButton.addEventListener("click", submitForm);
	document.addEventListener("keydown", pressKey);
}

function loadSongDialog(eventPropagator){
	let div = document.createElement("div");
	let songSelector = document.createElement("select");
	let submitButton = document.createElement("button");
	let options = [];
	
	div.className = "extra-dialog";
	
	submitButton.innerHTML = "Load song";
	
	let retrieveSongs = game => {
		return game.songs();
	}
	
	let songs = eventPropagator.runOnGame(retrieveSongs);
	let idIDX;
	let nameIDX;
	let artistIDX;
	let difficultyIDX;
	let durationIDX;
	let timeCreatedIDX;
	let timeModifiedIDX;
	
	if(songs.length != 0){
		songs[0]["columns"].forEach( (columnName, idx) => {
			if(columnName.toUpperCase() === "SONGID"){
				idIDX = idx;
			}
			else if(columnName.toUpperCase() === "NAME"){
				nameIDX = idx;
			}
			else if(columnName.toUpperCase() === "ARTIST"){
				artistIDX = idx;
			}
			else if(columnName.toUpperCase() === "DIFFICULTY"){
				difficultyIDX = idx;
			}
			else if(columnName.toUpperCase() === "DURATION"){
				durationIDX = idx;
			}
			else if(columnName.toUpperCase() === "TIMECREATED"){
				timeCreatedIDX = idx;
			}
			else if(columnName.toUpperCase() === "TIMEMODIFIED"){
				timeModifiedIDX = idx;
			}
		});
		songs[0]["values"].forEach( (song, idx) => {
			let newOption = document.createElement("option");
			let timeCreated = new Date(song[timeCreatedIDX]).toString();
			let timeModified = new Date(song[timeModifiedIDX]).toString();
			newOption.value = song[idIDX];
			newOption.innerHTML = `Name: ${song[nameIDX]}, Artist: ${song[artistIDX]}, Difficulty: ${song[difficultyIDX]}, Duration: ${song[durationIDX]}, Time Created: ${timeCreated}, Time Modified: ${timeModified}`;
			
			options.push(newOption);
		});
	}
	
	options.forEach( o => {
		songSelector.appendChild(o);
	});
	div.appendChild(songSelector);
	div.appendChild(submitButton);
	document.body.appendChild(div);
	
	let submitForm = () => {
		document.removeEventListener("keydown", pressKey);
		submitButton.removeEventListener("click", submitForm);
		
		let fn = game => {
			game.loadSong(parseInt(songSelector.value));
		}
		eventPropagator.runOnGame(fn, true);
		
		div.remove();
	};
	let pressKey = evt => {
		if(evt.keyCode == 27){
			document.removeEventListener("keydown", pressKey);
			submitButton.removeEventListener("click", submitForm);
			
			div.remove();
		}
	}
	
	submitButton.addEventListener("click", submitForm);
	document.addEventListener("keydown", pressKey);
}

function uploadMP3Dialog(eventPropagator){
	let div = document.createElement("div");
	let input = document.createElement("input");
	let mp3File;
	
	input.type = "file"
	
	div.className = "extra-dialog";
	
	div.appendChild(input);
	document.body.appendChild(div);
	
	let pressKey = evt => {
		if(evt.keyCode == 27){
			document.removeEventListener("keydown", pressKey);
			input.removeEventListener("change", inputFile);
			
			div.remove();
		}
	}
	
	let inputFile = evt => {
		document.removeEventListener("keydown", pressKey);
		input.removeEventListener("change", inputFile);
			
		mp3File = input.files[0];
		
		let fn = game => {
			game.loadMP3(mp3File);
		}
		
		eventPropagator.runOnGame(fn);
		
		div.remove();
	} 
	
	input.addEventListener("change", inputFile);
	document.addEventListener("keydown", pressKey);
}