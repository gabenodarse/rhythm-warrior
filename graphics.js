import * as myWasm from "./pkg/music_mercenary.js"; // !!! split into modules and change import name from myWasm


// >:< working with memory
// let foo; // unused
// myWasm.default().then(exports => foo = exports)
// let inputArray = new Uint8Array(foo.memory.buffer, myWasm.get_array_ptr(), 6);


const NUM_GRAPHICS = 2; // >:< derive from the rust source code


let g_gameCanvas = document.createElement('canvas');
let g_gameContext = g_gameCanvas.getContext('2d');
let g_canvases = new Array(NUM_GRAPHICS);


// !!! remove magic numbers and take into account screen size / options
g_gameCanvas.width = 1000; 
g_gameCanvas.height = 600;


// !!! refactor to a file other than graphics
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



//load all images from files into canvas contexts
async function load_images(resourceLocations){
	let results = [];
	
	function loadImage(imgKey, canvasID) {
		return new Promise(r => {
			// !!! make sure context is defined, throw error if not
			let img = new Image();
			img.onload = (() => {
				console.log("loaded 1");
				g_canvases[canvasID] = document.createElement('canvas'); // !!! make sure canvas can be created
				
				let size = myWasm.get_graphic_size(canvasID);
				g_canvases[canvasID].height = size.y;
				g_canvases[canvasID].width = size.x;
				g_canvases[canvasID].getContext('2d').drawImage(img, 0, 0, size.x, size.y); 
				r();
			});
			img.src = resourceLocations[imgKey]; // !!! add error handling (onfail if that exists, or a timeout)
		});
	}
	
	// IF AMOUNT OF RESOURCES DON'T MATCH THROW ERROR !!!
	results.push(
		loadImage("Player", myWasm.Graphic.Player)
		.catch( (rej) => { console.log(rej) })
	);
	results.push(
		loadImage("Brick", myWasm.Graphic.Brick)
		.catch( (rej) => { console.log(rej) })
	);
	
	console.log(results.length); // >:< make sure it's equal to num graphics
	
	return Promise.all(results).catch( () => { console.log("failed load"); } );
}


//renderAll
function renderAll(instructions) {
	// !!! error handling: check if instructions is an array of PositionedGraphic objects
	instructions.forEach((function(instruction) {
		g_gameContext.drawImage(g_canvases[instruction.g],instruction.x,instruction.y); 
	}));
}


export async function run() {
	let resourceLocations;
	let gameInstance;
	
	
	await Promise.all( [ 
		myWasm.default(),
		fetch("./resources.json")
			.then(r => r.json())
			.then(r => { resourceLocations = r }) // !!! add error handling
	]);
	
	
	await load_images(resourceLocations).then( 
		() => { 
			gameInstance = initGame(); // !!! add error handling
			document.body.appendChild(g_gameCanvas); // !!! determine position
			renderAll(gameInstance.get_instructions()); },
		e => { alert("load_images FAILED" + e); } // !!! add error handling
	);
	
	
	
	
	let last = (window.performance && window.performance.now) ? window.performance.now() : new Date().getTime();
	let now;
	const renderLoop = () => {
		// at a certain time threshold, get instruction
			// then render, asynchronously if possible to keep game ticking

		now = (window.performance && window.performance.now) ? window.performance.now() : new Date().getTime();
		
		
		// >:<
		if (now - last > 2000) { 
			gameInstance.tick((now - last) / 1000); // convert to seconds
			renderAll(gameInstance.get_instructions());
			// !!! get fps
			
			last = now;
		}
		
		
		requestAnimationFrame(renderLoop);
	};
	
	requestAnimationFrame(renderLoop);
	
}
