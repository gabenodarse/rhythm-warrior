import * as graphics from "./graphics.js";
import * as myWasm from "./pkg/music_mercenary.js"; // !!! split into modules and change import name from myWasm


export async function run() {
	let resourceLocations;
	let gameInstance;
	
	
	await Promise.all( [ 
		myWasm.default(),
		fetch("./resources.json")
			.then(r => r.json())
			.then(r => { resourceLocations = r }) // !!! add error handling
	]);
	
	
	await graphics.loadImages(resourceLocations).then( 
		() => { 
			gameInstance = initGame(); // !!! add error handling
			graphics.renderAll(gameInstance.get_instructions()); },
		e => { alert("loadImages FAILED" + e); } // !!! add error handling
	);
	
	
	let last = (window.performance && window.performance.now) ? window.performance.now() : new Date().getTime();
	let now;
	const renderLoop = () => {
		// at a certain time threshold, get instruction
			// then render, asynchronously if possible to keep game ticking

		now = (window.performance && window.performance.now) ? window.performance.now() : new Date().getTime();
		
		
		// >:<
		// if (now - last > 2000) { 
		gameInstance.tick((now - last) / 1000); // convert to seconds
		graphics.renderAll(gameInstance.get_instructions());
		// !!! get fps
		last = now;
		
		
		requestAnimationFrame(renderLoop);
	};
	
	requestAnimationFrame(renderLoop);
	
}


function initGame() {
	// !!! throw any errors
	let myGame = myWasm.Game.new();
	
	document.addEventListener("keydown", event => {
		switch (event.keyCode) {
			case myWasm.InputKey.Space:
			case myWasm.InputKey.LeftArrow:
			case myWasm.InputKey.UpArrow:
			case myWasm.InputKey.RightArrow:
			case myWasm.InputKey.DownArrow:
				myGame.input_command(event.keyCode);
				break;
			
		}
	});
	document.addEventListener("keyup", event => {
		switch (event.keyCode) {
			case myWasm.InputKey.Space:
			case myWasm.InputKey.LeftArrow:
			case myWasm.InputKey.UpArrow:
			case myWasm.InputKey.RightArrow:
			case myWasm.InputKey.DownArrow:
				myGame.stop_command(event.keyCode);
				break;
			
		}
	});
	
	
	return myGame;
}