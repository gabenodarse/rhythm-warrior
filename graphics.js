import * as wasm from "./pkg/music_mercenary.js";


const NUM_GRAPHICS = 6; // >:< derive from the rust source code

// !!! create function to assign to both this and Y Factor, and gameCanvas size, then remove the calculation in loadImages
let g_sizeXFactor; 
let g_sizeYFactor;
// !!! remove magic numbers and take into account screen size / options
let g_gameCanvas = document.createElement('canvas');
let g_gameContext = g_gameCanvas.getContext('2d');
g_gameCanvas.width = 1100; 
g_gameCanvas.height = 600;
document.body.appendChild(g_gameCanvas); // !!! determine position

let g_canvases = new Array(NUM_GRAPHICS);


//load all images from files into canvas contexts
export async function loadImages(resourceLocations){
	let results = [];
	
	let bgSize = wasm.graphic_size(wasm.Graphic.Background);
	g_sizeXFactor = g_gameCanvas.width / bgSize.x;
	g_sizeYFactor = g_gameCanvas.height / bgSize.y;
	
	function loadImage(imgKey, canvasID) {
		return new Promise(res => {
			let img = new Image();
			img.onload = (() => {
				console.log("loaded 1");
				g_canvases[canvasID] = document.createElement('canvas');
				
				let size = wasm.graphic_size(canvasID);
				g_canvases[canvasID].width = (size.x * g_sizeXFactor);
				g_canvases[canvasID].height = (size.y * g_sizeYFactor);
				g_canvases[canvasID].getContext('2d').drawImage(img, 0, 0, (size.x * g_sizeXFactor), (size.y * g_sizeYFactor));
				
				res();
			});
			img.src = resourceLocations[imgKey]; // TODO add error handling (onfail if that exists, or a timeout)
		});
	}
	
	// TODO better error handling
	function onReject(rej) {
		console.log(rej);
	}
	
	// >:< more automatic loading of graphics
	g_canvases[wasm.Graphic.None] = document.createElement('canvas');
	results.push(
		loadImage("Background", wasm.Graphic.Background)
		.catch( onReject )
	);
	results.push(
		loadImage("Player", wasm.Graphic.Player)
		.catch( onReject )
	);
	results.push(
		loadImage("Brick", wasm.Graphic.Brick)
		.catch( onReject )
	);
	results.push(
		loadImage("SlashLeft", wasm.Graphic.SlashLeft)
		.catch( onReject )
	);
	results.push(
		loadImage("SlashRight", wasm.Graphic.SlashRight)
		.catch( onReject )
	);
	results.push(
		loadImage("Dash", wasm.Graphic.Dash)
		.catch( onReject )
	);
	results.push(
		loadImage("DashL0", wasm.Graphic.DashL0)
		.catch( onReject )
	);
	results.push(
		loadImage("DashL1", wasm.Graphic.DashL1)
		.catch( onReject )
	);
	results.push(
		loadImage("DashL2", wasm.Graphic.DashL2)
		.catch( onReject )
	);
	results.push(
		loadImage("DashL3", wasm.Graphic.DashL3)
		.catch( onReject )
	);
	results.push(
		loadImage("DashR0", wasm.Graphic.DashR0)
		.catch( onReject )
	);
	results.push(
		loadImage("DashR1", wasm.Graphic.DashR1)
		.catch( onReject )
	);
	results.push(
		loadImage("DashR2", wasm.Graphic.DashR2)
		.catch( onReject )
	);
	results.push(
		loadImage("DashR3", wasm.Graphic.DashR3)
		.catch( onReject )
	);
	
	console.log(results.length); // TODO make sure it's equal to num graphics
	
	
	// TODO can move all catches on results to this Promise.all
	return Promise.all(results).catch( () => { console.log("failed load"); } );
}


export function renderAll(instructions) {
	// TODO error handling: check if instructions is an array of PositionedGraphic objects
	instructions.forEach( instruction => {
		g_gameContext.drawImage(g_canvases[instruction.g],instruction.x * g_sizeXFactor,instruction.y * g_sizeYFactor); 
	});
}

