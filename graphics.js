import * as myWasm from "./pkg/music_mercenary.js"; // !!! split into modules and change import name from myWasm


const NUM_GRAPHICS = 5; // >:< derive from the rust source code

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
	
	let bgSize = myWasm.get_graphic_size(myWasm.Graphic.Background);
	g_sizeXFactor = g_gameCanvas.width / bgSize.x;
	g_sizeYFactor = g_gameCanvas.height / bgSize.y;
	

	function loadImage(imgKey, canvasID) {
		return new Promise(r => {
			// !!! make sure context is defined, throw error if not
			let img = new Image();
			img.onload = (() => {
				console.log("loaded 1");
				g_canvases[canvasID] = document.createElement('canvas'); // !!! make sure canvas can be created
				
				let size = myWasm.get_graphic_size(canvasID);
				g_canvases[canvasID].width = (size.x * g_sizeXFactor);
				g_canvases[canvasID].height = (size.y * g_sizeYFactor);
				g_canvases[canvasID].getContext('2d').drawImage(img, 0, 0, (size.x * g_sizeXFactor), (size.y * g_sizeYFactor));
				
				r();
			});
			img.src = resourceLocations[imgKey]; // !!! add error handling (onfail if that exists, or a timeout)
		});
	}
	
	
	// !!! create a consistent rej function and handle error
	// IF AMOUNT OF RESOURCES DON'T MATCH THROW ERROR !!!
	results.push(
		loadImage("Background", myWasm.Graphic.Background)
		.catch( (rej) => { console.log(rej) })
	);
	results.push(
		loadImage("Player", myWasm.Graphic.Player)
		.catch( (rej) => { console.log(rej) })
	);
	results.push(
		loadImage("Brick", myWasm.Graphic.Brick)
		.catch( (rej) => { console.log(rej) })
	);
	results.push(
		loadImage("SlashLeft", myWasm.Graphic.SlashLeft)
		.catch( (rej) => { console.log(rej) })
	);
	results.push(
		loadImage("SlashRight", myWasm.Graphic.SlashRight)
		.catch( (rej) => { console.log(rej) })
	);
	
	console.log(results.length); // >:< make sure it's equal to num graphics
	
	return Promise.all(results).catch( () => { console.log("failed load"); } );
}


//renderAll
export function renderAll(instructions) {
	// !!! error handling: check if instructions is an array of PositionedGraphic objects
	instructions.forEach( instruction => {
		g_gameContext.drawImage(g_canvases[instruction.g],instruction.x * g_sizeXFactor,instruction.y * g_sizeYFactor); 
	});
}

