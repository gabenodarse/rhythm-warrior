
export function EventPropagator(){
	this.game;
	this.overlay;
	
	this.controls;
	
	this.paused;
	this.gameLoop;
	this.editorLoop;
	this.loop;
	this.resumeGame;
	this.handleKeyDown;
	this.handleKeyUp;
	this.resize;
	
	
}

// >:< moving functions to prototype means more difficult removal of listeners? or should some/all of this go to prototype?
EventPropagator.prototype.init = function(game, overlay, controls){
	this.game = game;
	this.overlay = overlay;
	this.controls = controls;
	
	this.gameLoop = () => {
		if(this.paused) {
			this.overlay.toggleElement("menu");
			this.game.pause();
			return;
		}
		
		this.game.tick();
		this.game.renderGame();
		requestAnimationFrame(this.loop); // !!! set timeout or request animation frame better?
	}
	
	this.editorLoop = () => {
		if(this.paused) {
			this.overlay.toggleElement("menu");
			this.game.pause();
			return;
		}
		
		this.game.tick();
		this.overlay.updateEditor(this.game.songData().songTime);
		this.game.renderGame();
		requestAnimationFrame(this.loop);
	}
	
	this.resumeGame = () => {
		this.overlay.toggleElement("menu");
		this.game.start(this.loop);
	}
	
	this.togglePause = () => {
		// unpause
		if(this.paused){
			this.paused = false;
			this.resumeGame();
			return;
		}
		
		else{
			// !!! handle game key states on pause/unpause (as of now fires key up events on pause)
			// pause
			for(const key in this.controls) {
				let evt = new KeyboardEvent("keyup", {
					keyCode: key,
				});
				this.handleKeyUp(evt);
			}
			this.paused = true;
		}
	}
	
	this.handleKeyDown = evt => {
		// TODO faster handling of repeated key inputs from holding down a key?
		if (evt.keyCode === 27){
			this.togglePause();
		}
		else if(typeof(this.controls[event.keyCode]) === "number" && !this.paused){
			this.game.startControl(this.controls[event.keyCode]);
		}
		evt.preventDefault();
	}
	
	this.handleKeyUp = evt => {
		if(typeof(this.controls[event.keyCode]) === "number" && !this.paused){
			this.game.stopControl(this.controls[event.keyCode]);
		}
	}
	
	let resizeRefresher = true;
	this.resize = () => {
		const delay = 50; // minimum delay between screen size refreshes
		if(resizeRefresher){
			resizeRefresher = false;
			let game = this.game;
			setTimeout(async function(){
				resizeRefresher = true;
				game.resize();
			},delay);
		}
	}
	
	this.loop = this.gameLoop;
	
	window.addEventListener("keydown", this.handleKeyDown);
	window.addEventListener("keyup", this.handleKeyUp);
	window.addEventListener("resize", this.resize); 
}

EventPropagator.prototype.start = function(){
	this.paused = false;
	this.resize();
	this.game.start(this.gameLoop);
}

EventPropagator.prototype.enableEditor = function(){
	this.overlay.toggleElement("editorOverlay");
	this.overlay.toggleElement("score");
	this.game = this.game.toEditor();
	this.loop = this.editorLoop;
	this.overlay.updateEditor(this.game.songData().songTime);
}
// !!! make screwing with game through UI impossible. Through hacking IDC. 
	// Distinction from game and editor (going to game from editor starts at 0?)
EventPropagator.prototype.disableEditor = function(){
	this.overlay.toggleElement("editorOverlay");
	this.overlay.toggleElement("score");
	this.game = this.game.toGame();
	this.loop = this.gameLoop;
	this.game.renderGame();
}

