
import * as wasm from "../pkg/music_mercenary.js";

let g_keyCodeNames = [];
g_keyCodeNames[32] = "Space";
g_keyCodeNames[13] = "Enter";

// !!! does overlay ever have to be resized?
// TODO all these classes have a DOMelement function, can make them all inherit from a DOMWrapper class

// main overlay class. all overlay elements are children, directly or nested
export function Overlay(game, controlsMap){
	this.game;
	this.controlsMap;

	this.overlayDiv;
	this.menu;
	this.capturingComponent;
	this.currentOverlay;
	
	this.game = game;
	this.controlsMap = controlsMap;

	this.overlayDiv = document.createElement("div");
	this.overlayDiv.id = "overlay";

	document.getElementById("screen").appendChild(this.overlayDiv);

	this.goToHomeScreen();
}

// class for the overlay active when the game is running
function GameOverlay(overlayParent){
	this.overlayParent;
	this.div;
	this.score;
	this.fps;

	this.overlayParent = overlayParent;

	this.div = document.createElement("div");
	this.div.className = "game-overlay";
	
	this.score = new Score();
	this.fps = new FPS();
	this.div.appendChild(this.score.domElement());
	this.div.appendChild(this.fps.domElement());
}

// class for the homescreen which holds song selections
function HomeScreen(overlayParent){
	this.overlayParent;
	this.homeScreenDiv;
	this.songSelectionsDiv;
	this.controlsDiv;
	
	this.mmTitle;
	this.songSelections;
	this.selectionIdx;
	
	this.logLink;
	
	this.overlayParent = overlayParent;

	this.homeScreenDiv = document.createElement("div");
	this.homeScreenDiv.id = "homescreen";
	
	this.mmTitle = document.createElement("h1");
	this.mmTitle.innerHTML = "Music Mercenary";
	this.songSelectionsDiv = document.createElement("div");
	this.songSelectionsDiv.className = "home-songs-div";
	this.controlsDiv = document.createElement("div");
	this.controlsDiv.className = "home-controls-div";

	this.logLink = document.createElement("button");
	this.logLink.className = "log-link";
	this.logLink.innerHTML = "Log";
	this.homeScreenDiv.appendChild(this.logLink);
	
	this.homeScreenDiv.appendChild(this.mmTitle);
	this.homeScreenDiv.appendChild(this.songSelectionsDiv);
	this.homeScreenDiv.appendChild(this.controlsDiv);
	
	this.controlsDiv.innerHTML = 
		"<h2>Controls</h2>\
		<p>The controls are Q, W, E, and Enter</p>\
		<p>Q to hit red notes<img src=\"./assets/images/brick1.png\"></p>\
		<p>W to hit yellow notes<img src=\"./assets/images/brick2.png\"></p>\
		<p>E to hit blue notes<img src=\"./assets/images/brick3.png\"></p>\
		<p>Enter to dash when indicated<img src=\"./assets/images/dash-indicator.png\"><img src=\"./assets/images/brick3.png\"></p>\
		<p>(Q,W,E) + Enter to hit groups\
			<img src=\"./assets/images/brick2.png\"><img src=\"./assets/images/brick2.png\"><img src=\"./assets/images/brick2.png\">\
			</p>\
		<p>Hold (Q,W,E) to hit hold notes<img src=\"./assets/images/brick1-hold-example.png\"></p>";
		

	this.songSelections = [];
	this.update();
}

// class for song selections which are attached to the home screen
function HomeSelection(songData){
	this.div;
	this.highlighted;
	
	this.songData;
	this.name;
	this.artist;
	this.difficulty;
	this.duration;
	
	this.songData = songData;
	this.name = songData.name;
	this.artist = songData.artist;
	this.difficulty = songData.difficulty;
	this.duration = songData.duration;
	
	this.div = document.createElement("div");
	this.div.className = "home-selection";
	
	this.nameField = document.createElement("p");
	this.artistField = document.createElement("p");
	this.infoField = document.createElement("p");
	
	this.nameField.innerHTML = this.name;
	this.artistField.innerHTML = this.artist;
	this.infoField.innerHTML = `Difficulty: ${this.difficulty} -- Duration: ${this.duration}`;
	
	this.div.appendChild(this.nameField);
	this.div.appendChild(this.artistField);
	this.div.appendChild(this.infoField);
}

// class for the homescreen which holds song selections
function EndGameScreen(overlayParent){
	this.endScreenDiv;
	this.overlayParent;
	this.textDiv;
	this.textElement1;
	this.textElement2;
	this.scoreTextElement;
	this.exitTextElement;
	
	this.endScreenDiv = document.createElement("div");
	this.endScreenDiv.className = "end-game-screen";
	
	this.textDiv = document.createElement("div");
	this.textDiv.className = "end-game-screen-text-div";
	
	this.textElement1 = document.createElement("p");
	this.textElement2 = document.createElement("p");
	this.scoreTextElement = document.createElement("p");
	this.exitTextElement = document.createElement("p");
	
	let game = overlayParent.getGame();
	let songData = game.getSongData();
	
	this.textElement1.innerHTML = songData.name + " - " + songData.artist;
	this.textElement2.innerHTML = "Difficulty: "+ songData.difficulty + " --- time: " + songData.duration;
	this.scoreTextElement.innerHTML = "Score: " + songData.gameData.score + " / " + songData.gameData.max_score;

	this.exitTextElement.innerHTML = "Enter to move on";

	this.textDiv.appendChild(this.textElement1);
	this.textDiv.appendChild(this.textElement2);
	this.textDiv.appendChild(this.scoreTextElement);
	this.textDiv.appendChild(this.exitTextElement);
	this.endScreenDiv.appendChild(this.textDiv);
	
	this.overlayParent = overlayParent;
}

// EditorOverlay class, contains a canvas where editor elements are drawn and the editor's controls
// !!! range scroller isn't modified when the song is modified
function EditorOverlay(overlayParent){
	let songData = overlayParent.getGame().getSongData();

	this.overlayParent;
	this.div;
	this.canvas;
	this.controlsDiv;
	
	// editor controls
	this.rangesDiv;
	this.buttonDiv;
	this.broadRange;
	this.preciseRange;
	this.playPauseButton;

	// necessary data
	this.groundPosOffset;
	this.mouseDown; // boolean describing if the mouse is down
	this.changeBrickType; // boolean describing whether to change the brick type
	this.selectedBrick; // current selected brick
	this.xFactor;
	this.yFactor;
	this.songTranscriptWidth;
	
	// initialize
	this.overlayParent = overlayParent;

	this.div = document.createElement("div");
	this.div.className = "editor-overlay";
	
	// controls
	this.controlsDiv = document.createElement("div");
	this.controlsDiv.className = "editor-controls";
	this.rangesDiv = document.createElement("div");
	this.rangesDiv.className = "editor-ranges-div";
	this.buttonDiv = document.createElement("div");
	this.buttonDiv.className = "editor-buttons-div";
	
	this.broadRange = document.createElement("input");
	this.broadRange.className = "editor-broad-range";
	this.broadRange.type = "range";
	this.broadRange.max = 100;
	this.broadRange.step = 0.1;
	this.broadRange.value = songData.gameData.time_running / songData.gameData.duration * 100;
	
	this.preciseRange = document.createElement("input");
	this.preciseRange.className = "editor-precise-range";
	this.preciseRange.type = "range";
	this.preciseRange.max = 4;
	this.preciseRange.step = 0.05;
	this.preciseRange.value = 0;
	
	this.playPauseButton = document.createElement("button");
	this.playPauseButton.innerHTML = "Play/Pause";

	this.rangesDiv.appendChild(this.preciseRange);
	this.rangesDiv.appendChild(this.broadRange);
	this.buttonDiv.appendChild(this.playPauseButton);
	this.controlsDiv.appendChild(this.rangesDiv);
	this.controlsDiv.appendChild(this.buttonDiv);

	// canvas
	this.canvas = document.createElement("canvas");
	let dims = wasm.game_dimensions();
	this.canvas.width = dims.x;
	this.canvas.height = dims.y;
	this.canvas.className = "full-sized";
	
	// data
	this.groundPosOffset = wasm.ground_pos();
	this.mouseDown = false;
	this.changeBrickType = false;
	this.selectedBrick = null;
	this.xFactor = 1;
	this.yFactor = 1;

	this.songTranscriptWidth = 280;
	
	this.div.appendChild(this.canvas);
	this.div.appendChild(this.controlsDiv);

	this.draw();
}



// Score class, displays a score which may be updated
function Score(){
	this.scoreDiv;
	this.score;
	this.scoreInner;
	
	this.scoreDiv = document.createElement("div");
	this.scoreDiv.className = "score";
	
	this.score = 0;
	this.scoreInner = document.createElement("p");
	
	this.scoreInner.innerHTML = this.score;
	
	this.scoreDiv.appendChild(this.scoreInner);
	document.body.appendChild(this.scoreDiv);
}

// FPS class, displays a fps which may be updated
function FPS(){
	this.fpsDiv;
	this.fps;
	this.fpsInner;
	
	this.fpsDiv = document.createElement("div");
	this.fpsDiv.className = "fps";
	
	this.fps = 0;
	this.fpsInner = document.createElement("p");
	
	this.fpsInner.innerHTML = this.fps;
	
	this.fpsDiv.appendChild(this.fpsInner);
	document.body.appendChild(this.fpsDiv);
}

// Menu class, contains selections and handles up and down keypresses
// !!! add support for hiding buttons (making navigation ignore inactive buttons)
	// !!! once done, make disable editor and enable editor buttons mutually exclusive / non buggy
function Menu(overlayParent){
	this.div;
	this.selections; // the different selections on the menu
	this.selectionIdx; // the index of the currently highlighted selection
	this.overlayParent;
	this.getInputDialog; // optional pop-up dialog to get input from user
	
	this.div = document.createElement("div");
	this.div.className = "menu";
	this.selections = [];
	this.selectionIdx = 0;

	this.getInputDialog = null;

	this.overlayParent = overlayParent;
}

// Menu when accessed from the homescreen. Extends Menu class
function HomeMenu(overlayParent){
	Menu.call(this, overlayParent);

	this.addSelection(() => { 
		this.overlayParent.closeMenu();
		this.overlayParent.openControlsMenu();
		return null;
	}, "Controls");
}
Object.setPrototypeOf(HomeMenu.prototype, Menu.prototype);

// Menu when accessed from the game. Extends Menu class
function GameMenu(overlayParent){
	Menu.call(this, overlayParent);
	
	this.menuKeyPresses = []; // keep track of key presses for opening master menu

	this.addSelection(() => { 
		this.overlayParent.closeMenu();
		this.overlayParent.openControlsMenu();
		return null;
	}, "Controls");
	
	this.addSelection(() => {
		return "restart-song";
	}, "Restart song");
	
	this.addSelection(() => {
		this.overlayParent.closeMenu();
		this.overlayParent.goToHomeScreen();
		return "stop-loop";
	}, "Quit song");
}
Object.setPrototypeOf(GameMenu.prototype, Menu.prototype);

// Menu when accessed from the game and toggled to master. Extends Menu class
function MasterGameMenu(overlayParent){
	Menu.call(this, overlayParent);

	this.addSelection(() => { 
		this.overlayParent.closeMenu();
		this.overlayParent.openControlsMenu();
		return null;
	}, "Controls");

	this.addSelection(() => {
		return "restart-song";
	}, "Restart song");

	this.addSelection(() => {
		this.overlayParent.closeMenu();
		this.overlayParent.goToHomeScreen();
		return "stop-loop";
	}, "Quit song");

	this.addSelection(() => {
		let game = this.overlayParent.getGame();
		game = game.toEditor();

		this.overlayParent.closeMenu();
		this.overlayParent.goToEditorOverlay();
		return "pre-render";
	}, "Enable Editor");

	this.addSelection(() => {
		let game = this.overlayParent.getGame();
		game = game.toGame();

		this.overlayParent.goToGameOverlay();
		return "pre-render";
	}, "Disable Editor");

	this.addSelection(() => {
		this.overlayParent.closeMenu();
		this.overlayParent.openSaveLoadMenu();
		return null;
	}, "Save/Load");
}
Object.setPrototypeOf(MasterGameMenu.prototype, Menu.prototype);

// Sub-menu accessed from another menu in order to save/load songs. Extends Menu class
function SaveLoadMenu(overlayParent){
	Menu.call(this, overlayParent);
	
	this.addSelection(() => { 
		this.openDialog(new NewSongDialog(this.overlayParent, this));
		return null;
	}, "New Song");

	this.addSelection(() => { 
		this.openDialog(new ModifySongDialog(this.overlayParent, this));
		return null;
	}, "Modify Song");

	this.addSelection(() => { 
		this.openDialog(new SaveSongDialog(this.overlayParent, this));
		return null;
	}, "Save Song");

	this.addSelection(() => { 
		this.openDialog(new LoadSongDialog(this.overlayParent, this));
		return null;
	}, "Load Song");
}
Object.setPrototypeOf(SaveLoadMenu.prototype, Menu.prototype);

// sub-menu accessed from another menu in order to change the controls. Extends Menu class
function ControlsMenu(overlayParent, controlsMap){
	Menu.call(this, overlayParent);
	
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
		let defaultKeyName = g_keyCodeNames[defaultKey] ? g_keyCodeNames[defaultKey] : String.fromCharCode(defaultKey);
		this.addSelection(() => { 
			this.openDialog(new ChangeControlDialog(this.overlayParent, this, defaultKeyName, inputName, controlsMap, i));
			return null;
		}, inputName);
		this.setSelectionText(i,  inputName + " - " + defaultKeyName);
	}
}
Object.setPrototypeOf(ControlsMenu.prototype, Menu.prototype);

// MenuSelection class. Controls highlighting style and the selection function
function MenuSelection(onSelect, text, parentDiv){
	this.div;
	this.selectionText;
	this.onSelect;
	this.highlighted;
	
	this.div = document.createElement("div");
	this.div.className = "menu-selection";
	
	this.selectionText = document.createElement("p");
	this.selectionText.innerHTML = text;
	
	this.onSelect = onSelect;
	this.highlighted = false;
	
	this.div.appendChild(this.selectionText);
	parentDiv.appendChild(this.div);
}

function GetInputDialog(overlayParent, menuParent){
	this.overlayParent;
	this.menuParent;
	this.div;
	this.formDiv;
	this.formTitle;
	this.buttonsDiv;
	this.submitButton;
	this.cancelButton;
	this.submitFunction; // function to run when the submit button is pressed

	this.overlayParent = overlayParent;
	this.menuParent = menuParent;

	this.div = document.createElement("div");
	this.div.className = "get-input-dialog";
	this.formDiv = document.createElement("div");
	this.formDiv.className = ("dialog-form-div");
	this.buttonsDiv = document.createElement("div");
	this.buttonsDiv.className = "dialog-buttons-div";

	this.formTitle = document.createElement("p");
	this.submitButton = document.createElement("button");
	this.cancelButton = document.createElement("button");
	this.submitButton.innerHTML = "Submit";
	this.cancelButton.innerHTML = "Cancel";

	this.formDiv.appendChild(this.formTitle);
	this.buttonsDiv.appendChild(this.submitButton);
	this.buttonsDiv.appendChild(this.cancelButton);
	this.div.appendChild(this.formDiv);
	this.div.appendChild(this.buttonsDiv);

	this.submitFunction = () => {};
}

// class for creating a dialog to change the selected control. extends GetInputDialog
function ChangeControlDialog(overlayParent, menuParent, oldKeyName, controlName, controlsMap, inputID){
	GetInputDialog.call(this, overlayParent, menuParent);

	this.controlLabel;
	this.oldKeyLabel;
	this.newKeyLabel;
	this.newKeyCode;
	this.submitFunction;
	
	this.formTitle.innerHTML = "Change Control";

	this.controlLabel = document.createElement("label");
	this.oldKeyLabel = document.createElement("label");
	this.newKeyLabel = document.createElement("label");

	this.controlLabel.innerHTML = "Set key for: " + controlName;
	this.oldKeyLabel.innerHTML = "Old key: " + oldKeyName;
	this.newKeyLabel.innerHTML = "Enter a new key";
	
	// set the submit function
	this.submitFunction = () => {
		// unbind the previous key for this input
		for (const key in controlsMap){
			if(controlsMap[key] == inputID){
				controlsMap[key] = undefined;
				break;
			}
		}

		// if the new key is mapped to a different input, set that input to have no key mapping to it
		if(controlsMap[this.newKeyCode]){
			let controlID = controlsMap[this.newKeyCode];
			let controlName = wasm.Input[controlID];
			menuParent.setSelectionText(controlID, controlName + " - UNBOUND");
		}

		controlsMap[this.newKeyCode] = inputID;

		let newKeyName = g_keyCodeNames[this.newKeyCode] ? g_keyCodeNames[this.newKeyCode] : String.fromCharCode(this.newKeyCode);
		menuParent.setSelectionText(inputID, controlName + " - " + newKeyName);
	}

	this.formDiv.appendChild(this.controlLabel);
	this.formDiv.appendChild(document.createElement("br"));
	this.formDiv.appendChild(this.oldKeyLabel);
	this.formDiv.appendChild(document.createElement("br"));
	this.formDiv.appendChild(this.newKeyLabel);
}
Object.setPrototypeOf(ChangeControlDialog.prototype, GetInputDialog.prototype);

// class for creating a dialog to create a new song. extends GetInputDialog
function NewSongDialog(overlayParent, menuParent){
	GetInputDialog.call(this, overlayParent, menuParent);
	
	this.nameLabel;
	this.nameField;
	this.artistLabel;
	this.artistField;
	this.difficultyLabel;
	this.difficultyField;
	this.bpmLabel;
	this.bpmField;
	this.brickSpeedLabel;
	this.brickSpeedField;
	this.durationLabel;
	this.durationField;
	this.songStartOffsetLabel;
	this.songStartOffsetField;
	this.jsonNameLabel;
	this.jsonNameField;
	this.fileInput;

	this.formTitle.innerHTML = "New Song";

	let game = overlayParent.getGame();
	let songData = game.getSongData();
	let newLine = () => { return document.createElement("br"); }

	this.nameLabel = document.createElement("label");
	this.nameField = document.createElement("input");
	this.artistLabel = document.createElement("label");
	this.artistField = document.createElement("input");
	this.difficultyLabel = document.createElement("label");
	this.difficultyField = document.createElement("input");
	this.bpmLabel = document.createElement("label");
	this.bpmField = document.createElement("input");
	this.brickSpeedLabel = document.createElement("label");
	this.brickSpeedField = document.createElement("input");
	this.durationLabel = document.createElement("label");
	this.durationField = document.createElement("input");
	this.songStartOffsetLabel = document.createElement("label");
	this.songStartOffsetField = document.createElement("input");
	this.jsonNameLabel = document.createElement("label");
	this.jsonNameField = document.createElement("input");
	
	this.nameLabel.innerHTML = "Name: ";
	this.artistLabel.innerHTML = "Artist: ";
	this.difficultyLabel.innerHTML = "Difficulty(0-10): ";
	this.bpmLabel.innerHTML = "BPM(40-160): ";
	this.brickSpeedLabel.innerHTML = "Brick Speed(100-5000): ";
	this.durationLabel.innerHTML = "Duration(0-600): ";
	this.songStartOffsetLabel.innerHTML = "Song start offset (0-6, use 0 if unknown): ";
	this.jsonNameLabel.innerHTML = "Song data file (json): ";

	this.nameField.defaultValue = "";
	this.nameField.type = "text";
	this.artistField.defaultValue = "";
	this.artistField.type = "text";
	this.difficultyField.defaultValue = "";
	this.difficultyField.type = "text";
	this.bpmField.defaultValue = "";
	this.bpmField.type = "text";
	this.brickSpeedField.defaultValue = "";
	this.brickSpeedField.type = "text";
	this.durationField.defaultValue = "";
	this.durationField.type = "text";
	this.songStartOffsetField.defaultValue = "";
	this.songStartOffsetField.type = "text";
	this.jsonNameField.defaultValue = "my-song.json";
	this.jsonNameField.type = "text";

	this.fileInput = document.createElement("input");
	this.fileInput.innerHTML = "song"
	this.fileInput.type = "file"

	this.submitFunction = () => {
		// validate data
		let name = this.nameField.value;
		let artist = this.artistField.value;
		let difficulty = parseInt(this.difficultyField.value);
			if(isNaN(difficulty) || difficulty < 0){
				difficulty = 0;
			}
			else if(difficulty > 10){
				difficulty = 10;
			}
		let bpm = parseInt(this.bpmField.value);
			if(isNaN(bpm) || bpm < 40){
				bpm = 40;
			}
			else if(bpm > 160){
				bpm = 160;
			}
		let brickSpeed = parseInt(this.brickSpeedField.value);
			if(isNaN(brickSpeed) || brickSpeed < 100){
				brickSpeed = 100;
			}
			else if(brickSpeed > 5000){
				brickSpeed = 5000;
			}
		let duration = parseInt(this.durationField.value);
			if(isNaN(duration) || duration < 0){
				duration = 0;
			}
			else if(duration > 600){
				duration = 600;
			}
		let songStartOffset = parseFloat(this.songStartOffsetField.value);
			if(isNaN(songStartOffset) || songStartOffset < 0){
				songStartOffset = 0;
			}
			else if(songStartOffset > 6){
				songStartOffset = 6;
			}
		let jsonFileName = this.jsonNameField.value;
			if (jsonFileName.slice(-5) != ".json"){
				jsonFileName = jsonFileName + ".json";
			}
		let songFile = this.fileInput.files[0];
		
		if(!songFile){
			alert("no audio was uploaded. The previous song's audio will be used.")
			game.newSong(name, artist, difficulty, bpm, brickSpeed, duration, songStartOffset, "", jsonFileName);
		}
		else{
			game.newSong(name, artist, difficulty, bpm, brickSpeed, duration, songStartOffset, songFile.name, jsonFileName);
			game.loadMP3(songFile);
		}

		return "wait-song-load";
	}

	this.formDiv.appendChild(this.nameLabel);
	this.formDiv.appendChild(this.nameField);
	this.formDiv.appendChild(newLine());
	this.formDiv.appendChild(this.artistLabel);
	this.formDiv.appendChild(this.artistField);
	this.formDiv.appendChild(newLine());
	this.formDiv.appendChild(this.difficultyLabel);
	this.formDiv.appendChild(this.difficultyField);
	this.formDiv.appendChild(newLine());
	this.formDiv.appendChild(this.bpmLabel);
	this.formDiv.appendChild(this.bpmField);
	this.formDiv.appendChild(newLine());
	this.formDiv.appendChild(this.brickSpeedLabel);
	this.formDiv.appendChild(this.brickSpeedField);
	this.formDiv.appendChild(newLine());
	this.formDiv.appendChild(this.durationLabel);
	this.formDiv.appendChild(this.durationField);
	this.formDiv.appendChild(newLine());
	this.formDiv.appendChild(this.songStartOffsetLabel);
	this.formDiv.appendChild(this.songStartOffsetField);
	this.formDiv.appendChild(newLine());
	this.formDiv.appendChild(this.jsonNameLabel);
	this.formDiv.appendChild(this.jsonNameField);
	this.formDiv.appendChild(newLine());
	this.formDiv.appendChild(this.fileInput);
}
Object.setPrototypeOf(NewSongDialog.prototype, GetInputDialog.prototype);

// class for creating a dialog to modify the current song data. extends GetInputDialog
function ModifySongDialog(overlayParent, menuParent){
	GetInputDialog.call(this, overlayParent, menuParent);
	
	this.nameLabel;
	this.nameField;
	this.artistLabel;
	this.artistField;
	this.difficultyLabel;
	this.difficultyField;
	this.bpmLabel;
	this.bpmField;
	this.brickSpeedLabel;
	this.brickSpeedField;
	this.durationLabel;
	this.durationField;
	this.songStartOffsetLabel;
	this.songStartOffsetField;
	this.jsonNameLabel;
	this.jsonNameField;
	this.fileInput;

	this.formTitle.innerHTML = "Modify Song";

	let game = overlayParent.getGame();
	let songData = game.getSongData();
	let newLine = () => { return document.createElement("br"); }

	this.nameLabel = document.createElement("label");
	this.nameField = document.createElement("input");
	this.artistLabel = document.createElement("label");
	this.artistField = document.createElement("input");
	this.difficultyLabel = document.createElement("label");
	this.difficultyField = document.createElement("input");
	this.bpmLabel = document.createElement("label");
	this.bpmField = document.createElement("input");
	this.brickSpeedLabel = document.createElement("label");
	this.brickSpeedField = document.createElement("input");
	this.durationLabel = document.createElement("label");
	this.durationField = document.createElement("input");
	this.songStartOffsetLabel = document.createElement("label");
	this.songStartOffsetField = document.createElement("input");
	this.jsonNameLabel = document.createElement("label");
	this.jsonNameField = document.createElement("input");
	this.fileInputLabel = document.createElement("label");
	this.fileInput = document.createElement("input");

	this.nameLabel.innerHTML = "Name: ";
	this.artistLabel.innerHTML = "Artist: ";
	this.difficultyLabel.innerHTML = "Difficulty(0-10): ";
	this.bpmLabel.innerHTML = "BPM(40-160): ";
	this.brickSpeedLabel.innerHTML = "Brick Speed(100-5000): ";
	this.durationLabel.innerHTML = "Duration(0-600): ";
	this.songStartOffsetLabel.innerHTML = "Song start offset (0-6): ";
	this.jsonNameLabel.innerHTML = "Song data file (json): ";

	this.nameField.defaultValue = songData.name;
	this.nameField.type = "text";
	this.artistField.defaultValue = songData.artist;
	this.artistField.type = "text";
	this.difficultyField.defaultValue = songData.difficulty;
	this.difficultyField.type = "text";
	this.bpmField.defaultValue = songData.gameData.bpm;
	this.bpmField.type = "text";
	this.brickSpeedField.defaultValue = songData.gameData.brick_speed;
	this.brickSpeedField.type = "text";
	this.durationField.defaultValue = songData.duration;
	this.durationField.type = "text";
	this.songStartOffsetField.defaultValue = songData.startOffset;
	this.songStartOffsetField.type = "text";
	this.jsonNameField.defaultValue = songData.jsonname;
	this.jsonNameField.type = "text";

	this.fileInputLabel.innerHTML = "Song audio (if updating)";
	this.fileInput.type = "file";

	this.submitFunction = () => {
		// validate data
		let name = this.nameField.value;
		let artist = this.artistField.value;
		let difficulty = parseInt(this.difficultyField.value);
			if(isNaN(difficulty) || difficulty < 0){
				difficulty = 0;
			}
			else if(difficulty > 10){
				difficulty = 10;
			}
		let bpm = parseInt(this.bpmField.value);
			if(isNaN(bpm) || bpm < 40){
				bpm = 40;
			}
			else if(bpm > 160){
				bpm = 160;
			}
		let brickSpeed = parseInt(this.brickSpeedField.value);
			if(isNaN(brickSpeed) || brickSpeed < 100){
				brickSpeed = 100;
			}
			else if(brickSpeed > 5000){
				brickSpeed = 5000;
			}
		let duration = parseInt(this.durationField.value);
			if(isNaN(duration) || duration < 0){
				duration = 0;
			}
			else if(duration > 600){
				duration = 600;
			}
		let songStartOffset = parseFloat(this.songStartOffsetField.value);
			if(isNaN(songStartOffset) || songStartOffset < 0){
				songStartOffset = 0;
			}
			else if(songStartOffset > 6){
				songStartOffset = 6;
			}
		let jsonFileName = this.jsonNameField.value;
			if (jsonFileName.slice(-5) != ".json"){
				jsonFileName = jsonFileName + ".json";
			}
			
		let songFile = this.fileInput.files[0];
		let songFileName = songData.filename;
		
		if(songFile){
			game.loadMP3(songFile);
			songFileName = songFile.name;
			this.fileInputLabel.innerHTML = "Modified ";
		}
		
		game.modifySong(name, artist, difficulty, bpm, brickSpeed, duration, songStartOffset, songFileName, jsonFileName);

		return null;
	}

	this.formDiv.appendChild(this.nameLabel);
	this.formDiv.appendChild(this.nameField);
	this.formDiv.appendChild(newLine());
	this.formDiv.appendChild(this.artistLabel);
	this.formDiv.appendChild(this.artistField);
	this.formDiv.appendChild(newLine());
	this.formDiv.appendChild(this.difficultyLabel);
	this.formDiv.appendChild(this.difficultyField);
	this.formDiv.appendChild(newLine());
	this.formDiv.appendChild(this.bpmLabel);
	this.formDiv.appendChild(this.bpmField);
	this.formDiv.appendChild(newLine());
	this.formDiv.appendChild(this.brickSpeedLabel);
	this.formDiv.appendChild(this.brickSpeedField);
	this.formDiv.appendChild(newLine());
	this.formDiv.appendChild(this.durationLabel);
	this.formDiv.appendChild(this.durationField);
	this.formDiv.appendChild(newLine());
	this.formDiv.appendChild(this.songStartOffsetLabel);
	this.formDiv.appendChild(this.songStartOffsetField);
	this.formDiv.appendChild(newLine());
	this.formDiv.appendChild(this.jsonNameLabel);
	this.formDiv.appendChild(this.jsonNameField);
	this.formDiv.appendChild(newLine());
	this.formDiv.appendChild(this.fileInputLabel);
	this.formDiv.appendChild(this.fileInput);
}
Object.setPrototypeOf(ModifySongDialog.prototype, GetInputDialog.prototype);

// class for creating a dialog to save the current song to the database. extends GetInputDialog
function SaveSongDialog(overlayParent, menuParent){
	GetInputDialog.call(this, overlayParent, menuParent);
	
	this.nameLabel;
	this.artistLabel;
	this.difficultyLabel;
	this.bpmLabel;
	this.brickSpeedLabel;
	this.durationLabel;
	this.songStartOffsetLabel;
	this.jsonNameLabel;

	this.formTitle.innerHTML = "Save Song";

	let game = overlayParent.getGame();
	let songData = game.getSongData();
	let newLine = () => { return document.createElement("br"); }

	this.nameLabel = document.createElement("label");
	this.artistLabel = document.createElement("label");
	this.difficultyLabel = document.createElement("label");
	this.bpmLabel = document.createElement("label");
	this.brickSpeedLabel = document.createElement("label");
	this.durationLabel = document.createElement("label");
	this.songStartOffsetLabel = document.createElement("label");
	this.jsonNameLabel = document.createElement("label");

	this.nameLabel.innerHTML = "Name: " + songData.name;
	this.artistLabel.innerHTML = "Artist: " + songData.artist;
	this.difficultyLabel.innerHTML = "Difficulty(0-10): " + songData.difficulty;
	this.bpmLabel.innerHTML = "BPM(40-160): " + songData.gameData.bpm;
	this.brickSpeedLabel.innerHTML = "Brick Speed(100-5000): " + songData.gameData.brick_speed;
	this.durationLabel.innerHTML = "Duration(0-600): " + songData.duration;
	this.songStartOffsetLabel.innerHTML = "Song start offset (0-6): " + songData.startOffset;
	this.jsonNameLabel.innerHTML = "Song data file (json): " + songData.jsonname;

	this.submitFunction = () => {
		game.saveSong(songData);
		return null;
	}

	this.formDiv.appendChild(this.nameLabel);
	this.formDiv.appendChild(newLine());
	this.formDiv.appendChild(this.artistLabel);
	this.formDiv.appendChild(newLine());
	this.formDiv.appendChild(this.difficultyLabel);
	this.formDiv.appendChild(newLine());
	this.formDiv.appendChild(this.bpmLabel);
	this.formDiv.appendChild(newLine());
	this.formDiv.appendChild(this.brickSpeedLabel);
	this.formDiv.appendChild(newLine());
	this.formDiv.appendChild(this.durationLabel);
	this.formDiv.appendChild(newLine());
	this.formDiv.appendChild(this.songStartOffsetLabel);
	this.formDiv.appendChild(newLine());
	this.formDiv.appendChild(this.jsonNameLabel);
}
Object.setPrototypeOf(SaveSongDialog.prototype, GetInputDialog.prototype);

// class for creating a dialog to load song data from the database. extends GetInputDialog
function LoadSongDialog(overlayParent, menuParent){
	GetInputDialog.call(this, overlayParent, menuParent);

	let game = overlayParent.getGame();
	
	this.songFileLabel = document.createElement("label");
	this.songFileLabel.innerHTML = "Song Audio File (MP3 / WAV): ";
	this.songFileInput = document.createElement("input");
	this.songFileInput.innerHTML = "Song audio file (mp3/wav)";
	this.songFileInput.type = "file";
	
	this.jsonFileLabel = document.createElement("label");
	this.jsonFileLabel.innerHTML = "Song JSON Data File: ";
	this.jsonFileInput = document.createElement("input");
	this.jsonFileInput.innerHTML = "Song data file (json)";
	this.jsonFileInput.type = "file";
	
	let newLine = () => { return document.createElement("br"); }
	
	this.submitFunction = () => {
		let songFile = this.songFileInput.files[0];
		
		let jsonFile = this.jsonFileInput.files[0];
		
		game.userLoadSong(songFile, jsonFile);
		return "wait-song-load";
	};
	
	this.formDiv.appendChild(this.songFileLabel);
	this.formDiv.appendChild(this.songFileInput);
	this.formDiv.appendChild(newLine());
	this.formDiv.appendChild(this.jsonFileLabel);
	this.formDiv.appendChild(this.jsonFileInput);
}
Object.setPrototypeOf(LoadSongDialog.prototype, GetInputDialog.prototype);

Overlay.prototype.goToGameOverlay = function(){
	this.removeCurrentOverlay();

	let gameOverlay = new GameOverlay(this);
	this.overlayDiv.appendChild(gameOverlay.domElement());
	this.currentOverlay = gameOverlay;
}

Overlay.prototype.goToEditorOverlay = function(){
	this.removeCurrentOverlay();

	let editorOverlay = new EditorOverlay(this);
	this.overlayDiv.appendChild(editorOverlay.domElement());
	this.setCapturingComponent(editorOverlay);
	this.currentOverlay = editorOverlay;
}

Overlay.prototype.goToHomeScreen = function(){
	this.removeCurrentOverlay();

	let homeScreen = new HomeScreen(this);
	this.overlayDiv.appendChild(homeScreen.domElement());
	this.setCapturingComponent(homeScreen);
	this.currentOverlay = homeScreen;
}

Overlay.prototype.goToEndGameScreen = function(){
	this.removeCurrentOverlay();

	let endGameScreen = new EndGameScreen(this);
	this.overlayDiv.appendChild(endGameScreen.domElement());
	this.setCapturingComponent(endGameScreen);
	this.currentOverlay = endGameScreen;
}

Overlay.prototype.removeCurrentOverlay = function(){
	if(this.currentOverlay){
		// TODO make sure currentOverlay has a DOM element and is a child of the parent overlay
		this.currentOverlay.domElement().remove();
		this.currentOverlay = null;
	}
}

Overlay.prototype.openHomeMenu = function(){
	if(this.menu){
		throw Error("attempting to open a menu while one is already open");
	}

	this.menu = new HomeMenu(this);
	this.overlayDiv.appendChild(this.menu.domElement());
	this.setCapturingComponent(this.menu);
}

Overlay.prototype.openGameMenu = function(){
	if(this.menu){
		throw Error("attempting to open a menu while one is already open");
	}

	this.menu = new GameMenu(this);
	this.overlayDiv.appendChild(this.menu.domElement())
	this.setCapturingComponent(this.menu);
}

Overlay.prototype.openMasterGameMenu = function(){
	if(this.menu){
		throw Error("attempting to open a menu while one is already open");
	}

	this.menu = new MasterGameMenu(this);
	this.overlayDiv.appendChild(this.menu.domElement())
	this.setCapturingComponent(this.menu);
}

Overlay.prototype.openSaveLoadMenu = function(){
	if(this.menu){
		throw Error("attempting to open a menu while one is already open");
	}

	this.menu = new SaveLoadMenu(this);
	this.overlayDiv.appendChild(this.menu.domElement())
	this.setCapturingComponent(this.menu);
}

Overlay.prototype.openControlsMenu = function(){
	if(this.menu){
		throw Error("attempting to open a menu while one is already open");
	}

	this.menu = new ControlsMenu(this, this.controlsMap);
	this.overlayDiv.appendChild(this.menu.domElement())
	this.setCapturingComponent(this.menu);
}

Overlay.prototype.closeMenu = function(){
	if(this.menu == this.capturingComponent){
		this.removeCapturingComponent();
		if(!(this.currentOverlay instanceof GameOverlay)){
			this.setCapturingComponent(this.currentOverlay);
		}
	} else if(this.menu){
		throw Error("Closing a menu that isn't capturing events");
		this.menu.domElement().remove();
	} else {
		throw Error("Attempting to close menu when no menu is open");
	}

	this.menu = null;
}

Overlay.prototype.isCapturing = function(){
	if(this.capturingComponent){
		return true;
	} else {
		return false;
	}
}

Overlay.prototype.setCapturingComponent = function(component){
	// !!! check to make sure component is a child of the overlay?
	this.capturingComponent = component;
}

Overlay.prototype.removeCapturingComponent = function(){ 
	if(this.capturingComponent){
		this.capturingComponent.domElement().remove();
	}
	this.capturingComponent = null;
}

Overlay.prototype.passEvent = function(evt){
	if(!this.capturingComponent){
		throw Error("Attempting to pass an event to overlay when there is no capturing component");
	}
	if(!(typeof this.capturingComponent.handleEvent == "function")){
		throw Error("The capturing component in Overlay does not have a handleEvent function");
	}

	return this.capturingComponent.handleEvent(evt);
}

// TODO updating song data should update score?
Overlay.prototype.update = function(data=null){
	this.currentOverlay.update();

	if(this.currentOverlay instanceof GameOverlay){
		if(data){
			this.currentOverlay.updateFPS(data.fps);
		}
	}

	if(this.currentOverlay instanceof EditorOverlay){
		if(!data){
			throw Error("No data to update overlay");
		}
		this.currentOverlay.updateDimFactors(data.xFactor, data.yFactor);
	}
}

Overlay.prototype.getGame = function(){
	return this.game;
}

GameOverlay.prototype.update = function(){
	let game = this.overlayParent.getGame();
	
	this.score.update(game.getScore());
}

GameOverlay.prototype.updateFPS = function(fps){
	this.fps.update(fps);
}

GameOverlay.prototype.domElement = function(){
	return this.div;
}

HomeScreen.prototype.domElement = function(){
	return this.homeScreenDiv;
}

HomeScreen.prototype.handleEvent = function(evt){
	if(evt.type == "click" && evt.target == this.logLink){
		return "download-log";
	}

	if(evt.type == "keydown"){
		if(evt.keyCode === 27){ // escape key
			let overlay = this.overlayParent;
			overlay.openHomeMenu();
	
			return null;
		} 
		else if(evt.keyCode == 38 && this.selectionIdx > 0){ // up arrow
			this.songSelections[this.selectionIdx].toggleHighlight();
			--this.selectionIdx;
			this.songSelections[this.selectionIdx].toggleHighlight();
	
			return null;
		}
		else if(evt.keyCode == 40 && this.selectionIdx + 1 < this.songSelections.length){ // down arrow
			this.songSelections[this.selectionIdx].toggleHighlight();
			++this.selectionIdx;
			this.songSelections[this.selectionIdx].toggleHighlight();
	
			return null;
		}
		else if(evt.keyCode == 13){ // enter
			if(this.songSelections[this.selectionIdx]){
				let songData = this.songSelections[this.selectionIdx].getSongData();
				let game = this.overlayParent.getGame();
				game.loadSong(songData);
	
				this.overlayParent.removeCapturingComponent();
				this.overlayParent.goToGameOverlay();
				
				return("start-from-homescreen");
			}
			else{
				throw Error("Home Menu song selection idx out of bounds");
			}
		}
	}

	return null;
}

HomeScreen.prototype.update = function(){
	let songs = this.overlayParent.getGame().songs();

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
		songs.forEach( songData => {
			let selection = new HomeSelection(songData);
			this.songSelections.push(selection);
			this.songSelectionsDiv.appendChild(selection.domElement());
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

HomeSelection.prototype.getSongData = function(){
	return this.songData;
}

EndGameScreen.prototype.domElement = function(){
	return this.endScreenDiv;
}

EndGameScreen.prototype.handleEvent = function(evt){
	if(evt.type == "keydown" && evt.keyCode == 13){ // enter
		this.overlayParent.removeCapturingComponent();
		this.overlayParent.goToHomeScreen();
	}

	return null;
}

EditorOverlay.prototype.update = function(){
	let songData = this.overlayParent.getGame().getSongData();

	let songDuration = songData.gameData.duration;
	let beatInterval = songData.gameData.beat_interval;
	let time = songData.gameData.time_running;
	
	// reset the precise range to 0 if the time has changed since the ranges were used
	let prevT = parseFloat(this.broadRange.value) / 100 * songDuration 
		+ parseFloat(this.preciseRange.value) * beatInterval;
	if(time - prevT > 0.5 || prevT - time > 0.5){
		this.broadRange.value = time / songDuration * 100;
		this.preciseRange.value = 0;
	}
	
	this.draw();
}

EditorOverlay.prototype.updateDimFactors = function(xFactor, yFactor){
	this.xFactor = xFactor;
	this.yFactor = yFactor;
}

EditorOverlay.prototype.domElement = function(){
	return this.div;
}

EditorOverlay.prototype.timeToY = function(time){
	let songData = this.overlayParent.getGame().getSongData();

	let timeDifference = time - songData.gameData.time_running;
	let currentY = wasm.time_zero_brick_pos();
	let newY = currentY + timeDifference * songData.gameData.brick_speed;
	return newY;
}

EditorOverlay.prototype.yToTime = function(y){
	let songData = this.overlayParent.getGame().getSongData();

	let currentY = wasm.time_zero_brick_pos();
	let yDifference = y - currentY;
	let timeDifference = yDifference / songData.gameData.brick_speed;
	
	return songData.gameData.time_running + timeDifference;
}

EditorOverlay.prototype.xToNotePos = function(x){
	return Math.floor(x / wasm.brick_dimensions().x);
}

EditorOverlay.prototype.notePosToX = function(notePos){
	return x * wasm.brick_dimensions().x;
}

EditorOverlay.prototype.draw = function(){
	let game = this.overlayParent.getGame();
	let songData = game.getSongData();

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
		let y = this.timeToY(beatTime) + wasm.brick_dimensions().y / 2;
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
		let startY = this.timeToY(startTime);
		let endY = this.timeToY(endTime) + wasm.brick_dimensions().y;
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
	
	// draw the song data buffer / song transcript
	{
		let songBuffer = game.getSongBuffer();
		let imageDataWidth = this.songTranscriptWidth;
		let imageDataHeight = wasm.game_dimensions().y;
		let songTranscriptData = new Uint8ClampedArray(imageDataWidth * imageDataHeight * 4);
		
		let bufferData = songBuffer.getChannelData(0);
		let startingSample = Math.floor(
			songData.gameData.time_running * songBuffer.sampleRate // time of the song
			+ songData.startOffset * songBuffer.sampleRate // plus start offset
			- (wasm.time_zero_brick_pos() + wasm.brick_dimensions().y / 2)  / songData.brickSpeed * songBuffer.sampleRate); // adjusted to notes start position

		let endingSample = Math.floor(startingSample + (wasm.game_dimensions().y / songData.brickSpeed) * songBuffer.sampleRate);
		
		// for each sample, draw a 1 pixel point at a corresponding location
		for(let i = startingSample; i < endingSample; ++i){
			if(i < 0){ 
				continue;
			}
			else if (i > bufferData.length){
				break;
			}
			
			let timeFromTop = (i - startingSample) / songBuffer.sampleRate;
			
			let x = Math.floor(bufferData[i] * imageDataWidth / 2.0 + (imageDataWidth / 2.0)); // amplitude of the sample set in range 0 - imageDataWidth
			let y = Math.floor(timeFromTop * songData.brickSpeed); // time of the sample transcribed onto game
			let dataIdx = Math.floor((y * imageDataWidth * 4) + (x * 4));
			
			// set RGBA to 40,40,40,255 (solid black)
			songTranscriptData[dataIdx] = 40;
			songTranscriptData[dataIdx + 1] = 40;
			songTranscriptData[dataIdx + 2] = 40;
			songTranscriptData[dataIdx + 3] = 255;
		}
		
		// create a new ImageData object for the song transcript, then draw it on the canvas
		let songTranscriptImageData = new ImageData(songTranscriptData, imageDataWidth);
		ctx.putImageData(songTranscriptImageData, 0, 0);
	}
}

EditorOverlay.prototype.handleEvent = function(evt){
	let songData = this.overlayParent.getGame().getSongData();

	if(evt.type == "keydown" && evt.keyCode == 27){
		let overlay = this.overlayParent;
		overlay.openMasterGameMenu();
		return "stop-loop";
	}
	
	else if(evt.type == "input" && (evt.target == this.broadRange || evt.target == this.preciseRange)){
		let t = parseFloat(this.broadRange.value) / 100 * songData.gameData.duration 
			+ parseFloat(this.preciseRange.value) * songData.gameData.beat_interval;
			
		let game = this.overlayParent.getGame();
		game.seek(t);
		this.draw();
		return "stop-loop"
	}

	else if(evt.type == "click" && evt.target == this.playPauseButton){
		return "toggle-play"
	}

	else if(evt.type == "keydown"){
		return this.handleKeyDown(evt);
	} else if(evt.type == "mousedown"){
		return this.handleMouseDown(evt);
	} else if(evt.type == "mouseup"){
		return this.handleMouseUp(evt);
	} else if(evt.type == "mousemove"){
		return this.handleMouseMove(evt);
	} else if(evt.type == "wheel"){
		return this.handleWheel(evt);
	}

	return null;
}

EditorOverlay.prototype.handleKeyDown = function(evt){
	if(evt.keyCode == 32 || evt.keyCode == 13){ // space or enter
		this.selectedBrick = null;
		return("toggle-play");
	}
	
	if(this.selectedBrick){
		let brick = this.selectedBrick;
		let game = this.overlayParent.getGame();
		
		if(evt.keyCode == 46 || evt.keyCode == 8) { // delete or backspace
			let game = this.overlayParent.getGame();
			game.removeBrick(brick.brick_type, brick.beat_pos, brick.end_beat_pos, brick.x_pos, 
				brick.is_triplet, brick.is_trailing, brick.is_leading, brick.is_hold_note);
			this.selectedBrick = null;
		}
		
		if(evt.keyCode == 38){ // up arrow. delete the old brick, move the brick up and recreate
			game.removeBrick(brick.brick_type, brick.beat_pos, brick.end_beat_pos, brick.x_pos, 
				brick.is_triplet, brick.is_trailing, brick.is_leading, brick.is_hold_note);
			
			if(brick.is_trailing){
				brick.is_trailing = false;
			} else if(brick.is_leading){
				brick.is_leading = false;
				brick.beat_pos -= 1;
				brick.end_beat_pos -= 1;
			} else {
				brick.is_leading = true;
			}
			
			game.createBrick(brick.brick_type, brick.beat_pos, brick.end_beat_pos, brick.x_pos, 
				brick.is_triplet, brick.is_trailing, brick.is_leading, brick.is_hold_note);
			this.selectedBrick = game.selectBrick(brick.beat_pos, brick.x_pos);
		}
		
		if(evt.keyCode == 40){ // down arrow. delete the old brick, move the brick down and recreate
			game.removeBrick(brick.brick_type, brick.beat_pos, brick.end_beat_pos, brick.x_pos, 
				brick.is_triplet, brick.is_trailing, brick.is_leading, brick.is_hold_note);
				
			if(brick.is_leading){
				brick.is_leading = false;
			} else if(brick.is_trailing){
				brick.is_trailing = false;
				brick.beat_pos += 1;
				brick.end_beat_pos += 1;
			} else {
				brick.is_trailing = true;
			}
			
			game.createBrick(brick.brick_type, brick.beat_pos, brick.end_beat_pos, brick.x_pos, 
				brick.is_triplet, brick.is_trailing, brick.is_leading, brick.is_hold_note);
			this.selectedBrick = game.selectBrick(brick.beat_pos, brick.x_pos);
		}
		
		if(evt.keyCode == 37){ // left arrow. delete the old brick, move the brick to the left and recreate
			game.removeBrick(brick.brick_type, brick.beat_pos, brick.end_beat_pos, brick.x_pos, 
				brick.is_triplet, brick.is_trailing, brick.is_leading, brick.is_hold_note);
				
			brick.x_pos -= brick.x_pos > 0 ? 1 : 0;
			
			game.createBrick(brick.brick_type, brick.beat_pos, brick.end_beat_pos, brick.x_pos, 
				brick.is_triplet, brick.is_trailing, brick.is_leading, brick.is_hold_note);
			this.selectedBrick = game.selectBrick(brick.beat_pos, brick.x_pos);
		}
		
		if(evt.keyCode == 39){ // right arrow. delete the old brick, move the brick to the right and recreate
			game.removeBrick(brick.brick_type, brick.beat_pos, brick.end_beat_pos, brick.x_pos, 
				brick.is_triplet, brick.is_trailing, brick.is_leading, brick.is_hold_note);
				
			brick.x_pos += brick.x_pos + 1 < wasm.max_notes_per_screen_width() ? 1 : 0;
			
			game.createBrick(brick.brick_type, brick.beat_pos, brick.end_beat_pos, brick.x_pos, 
				brick.is_triplet, brick.is_trailing, brick.is_leading, brick.is_hold_note);
			this.selectedBrick = game.selectBrick(brick.beat_pos, brick.x_pos);
		}
		
		this.draw();
		return "pre-render";
	}
	
	return null;
}

EditorOverlay.prototype.handleMouseDown = function(evt){
	if(evt.target != this.canvas){
		return null;
	}
	
	let game = this.overlayParent.getGame();
	let songData = game.getSongData();
	let x = evt.clientX - this.canvas.offsetLeft;
	let y = evt.clientY - this.canvas.offsetTop;
	
	x = x / this.xFactor;
	y = y / this.yFactor;
	let approxTime = this.yToTime(y);
	let xPos = this.xToNotePos(x);
	let beatPos = wasm.BrickData.closest_beat_pos(approxTime, songData.gameData.bpm);
	
	let brick = game.selectBrick(beatPos, xPos);
	
	if(brick){
		// if clicking on an already selected brick, indicate to change the brick type
		if(this.selectedBrick && this.selectedBrick.beat_pos == brick.beat_pos && this.selectedBrick.x_pos == brick.x_pos){
			this.changeBrickType = true;
		}
	} else {
		game.createDefaultBrick(beatPos, xPos);
		brick = game.selectBrick(beatPos, xPos);
	}
	
	this.selectedBrick = brick;
	
	this.mouseDown = true;
	
	this.draw();

	return "stop-loop";
}

EditorOverlay.prototype.handleMouseUp = function(evt){
	this.mouseDown = false;
	if(this.changeBrickType){
		let game = this.overlayParent.getGame();
		let brick = this.selectedBrick;
		game.removeBrick(brick.brick_type, brick.beat_pos, brick.end_beat_pos, brick.x_pos, 
			brick.is_triplet, brick.is_trailing, brick.is_leading, brick.is_hold_note);
		
		if(brick.brick_type < 2){
			brick.brick_type += 1;
		} else {
			brick.brick_type = 0;
		}
		
		game.createBrick(brick.brick_type, brick.beat_pos, brick.end_beat_pos, brick.x_pos, 
			brick.is_triplet, brick.is_trailing, brick.is_leading, brick.is_hold_note);
		this.selectedBrick = game.selectBrick(brick.beat_pos, brick.x_pos);

		this.changeBrickType = false;

		this.draw();
		return "pre-render";
	}

	return null;
}
	
EditorOverlay.prototype.handleMouseMove = function(evt){
	if(evt.target != this.canvas){
		return null;
	}
	
	if(this.mouseDown && this.selectedBrick){
		let game = this.overlayParent.getGame();
		let songData = game.getSongData();
		let x = evt.clientX - this.canvas.offsetLeft;
		let y = evt.clientY - this.canvas.offsetTop;
		
		x = x / this.xFactor;
		y = y / this.yFactor;
		let approxTime = this.yToTime(y);
		let xPos = this.xToNotePos(x);
		let beatPos = wasm.BrickData.closest_beat_pos(approxTime, songData.gameData.bpm);
		
		// if the beat pos or the x pos has changed, move the brick
		if(beatPos != this.selectedBrick.end_beat_pos || xPos != this.selectedBrick.x_pos){
			this.changeBrickType = false;
			
			let brick = this.selectedBrick;
			game.removeBrick(brick.brick_type, brick.beat_pos, brick.end_beat_pos, brick.x_pos, 
				brick.is_triplet, brick.is_trailing, brick.is_leading, brick.is_hold_note);
			
			if(beatPos > brick.beat_pos){
				brick.is_hold_note = true;
				brick.end_beat_pos = beatPos;
			} else {
				brick.is_hold_note = false;
				brick.beat_pos = beatPos;
				brick.end_beat_pos = beatPos;
			}
			
			if(xPos != brick.x_pos){
				brick.x_pos = xPos;
			}
			
			game.createBrick(brick.brick_type, brick.beat_pos, brick.end_beat_pos, brick.x_pos, 
				brick.is_triplet, brick.is_trailing, brick.is_leading, brick.is_hold_note);
				
			this.selectedBrick = game.selectBrick(beatPos, xPos);
			
			this.draw();
			return "pre-render";
		}
	}
	
	return null;
}

EditorOverlay.prototype.handleWheel = function(evt){
	let game = this.overlayParent.getGame();
	let time = game.getSongData().gameData.time_running;
	let brickSpeed = game.getSongData().gameData.brick_speed;
	
	time += evt.deltaY / brickSpeed;
	game.seek(time);

	this.draw();

	return "stop-loop";
}

Menu.prototype.addSelection = function(onSelect, selectionText){
	this.selections.push( new MenuSelection(onSelect, selectionText, this.div) );
	if(this.selections.length == 1){
		this.selections[0].toggleHighlight();
	}
}

Menu.prototype.domElement = function(){
	return this.div;
}

Menu.prototype.handleEvent = function(evt){
	if(this.getInputDialog instanceof GetInputDialog){
		return this.getInputDialog.handleEvent(evt);
	}

	if(evt.type == "keydown"){
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
				return this.selections[this.selectionIdx].select();
			}
		}
		else if(evt.keyCode == 27){
			this.overlayParent.closeMenu();
		}
	}

	return null;
}

Menu.prototype.openDialog = function(dialog){
	if(this.getInputDialog){
		throw Error("Menu attempting to open a dialog when one is already open")
	}

	if(dialog instanceof GetInputDialog){
		this.getInputDialog = dialog;
		this.div.appendChild(dialog.domElement());
		return;
	}
	else{
		throw Error("Menu.openDialog called with a non-dialog argument");
	}
}

Menu.prototype.removeDialog = function(){
	if(this.getInputDialog instanceof GetInputDialog){
		this.getInputDialog.domElement().remove();
		this.getInputDialog = null;
		return;
	}
	else{
		throw Error("Menu attempting to close dialog while none is open");
	}
}

Menu.prototype.setSelectionText = function(selectionID, newText){
	this.selections[selectionID].setText( newText );
}

GameMenu.prototype.handleEvent = function(evt){
	if(evt.type == "keydown" && evt.keyCode == 27){
		this.overlayParent.closeMenu();
		return "toggle-play";
	} 
	else if(evt.type == "keydown"){
		// master menu
		this.menuKeyPresses.unshift(evt);
		if(this.menuKeyPresses.length > 6){
			this.menuKeyPresses.pop();
		}
		if( this.menuKeyPresses.length >= 6 && this.menuKeyPresses[5].keyCode == 77
		&& this.menuKeyPresses[4].keyCode == 65  && this.menuKeyPresses[3].keyCode == 83 
		&& this.menuKeyPresses[2].keyCode == 84 && this.menuKeyPresses[1].keyCode == 69 
		&& this.menuKeyPresses[0].keyCode == 82 ){
			this.overlayParent.closeMenu();
			this.overlayParent.openMasterGameMenu();
			return null;
		}
	} 

	return Menu.prototype.handleEvent.call(this, evt);
}

MasterGameMenu.prototype.handleEvent = function(evt){
	if(evt.type == "keydown" && evt.keyCode == 27){
		this.overlayParent.closeMenu();
		return "toggle-play";
	} 

	return Menu.prototype.handleEvent.call(this, evt);
}

MenuSelection.prototype.select = function(){
	if(typeof(this.onSelect) == "function"){
		return this.onSelect();
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

Score.prototype.domElement = function(){
	return this.scoreDiv;
}

Score.prototype.update = function(newScore){
	if(newScore != this.score){
		this.score = newScore;
		this.scoreInner.innerHTML = newScore;
	}
}

FPS.prototype.domElement = function(){
	return this.fpsDiv;
}

FPS.prototype.update = function(newFPS){
	if(newFPS != this.fps){
		this.fps = newFPS;
		this.fpsInner.innerHTML = newFPS;
	}
}

GetInputDialog.prototype.domElement = function(){
	return this.div;
}

GetInputDialog.prototype.handleEvent = function(evt){
	if(evt.type == "keydown"){
		if(evt.keyCode == 27){
			this.menuParent.removeDialog();
		}

		return null;
	}

	if(evt.type == "click"){
		if(evt.target == this.submitButton){
			let instruction = this.submitFunction();
			this.menuParent.removeDialog();
			return instruction;
		}
		else if (evt.target == this.cancelButton){
			this.menuParent.removeDialog();
		}
	}
}

ChangeControlDialog.prototype.handleEvent = function(evt){
	if(evt.type == "keydown" && evt.keyCode != 27){
		this.newKeyCode = evt.keyCode;
		let newKeyName = g_keyCodeNames[this.newKeyCode] ? g_keyCodeNames[this.newKeyCode] : String.fromCharCode(this.newKeyCode);
		this.newKeyLabel.innerHTML = "New Key: " + newKeyName;

		return null;
	}

	GetInputDialog.prototype.handleEvent.call(this, evt);
}
