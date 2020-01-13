import * as myWasm from "./pkg/music_mercenary.js"; //>:< split into modules and change import name from myWasm
// >:< delete resources.js

let g_gameCanvas = document.createElement('canvas');
let g_gameContext = g_gameCanvas.getContext('2d');
g_gameCanvas.width = 800;
g_gameCanvas.height = 600;

let i = 0;
let g_canvases = new Array(2);
let g_contexts = new Array(2); //>:<remove magic number >:<need global for canvases, not contexts

g_canvases[i] = document.createElement('canvas');
g_contexts[i] = g_canvases[i].getContext('2d');
++i;
g_canvases[i] = document.createElement('canvas');
g_contexts[i] = g_canvases[i].getContext('2d');
++i;
//>:<i should == number of graphics


//load all images from files into canvas contexts
async function load_images(resourceLocations){
	console.log("begin load");
	let results = [];
	
	function loadImage(imgKey, context) {
		return new Promise(r => {
			let img = new Image();
			img.onload = (() => {
				console.log("loaded 1");
				context.drawImage(img, 0, 0, 200, 200); // >:< arbitrary dimensions
				r(img);
			});
			img.src = resourceLocations[imgKey]; //what about on fail???>:<
		});
	}
	
	// IF AMOUNT OF RESOURCES DON'T MATCH THROW ERROR >:<
	results.push(
		loadImage("BALL", g_contexts[myWasm.Graphic.BALL])
		.catch( (rej) => { console.log(rej) })
	);
	results.push(
		loadImage("BRICK", g_contexts[myWasm.Graphic.BRICK])
		.catch( (rej) => { console.log(rej) })
	);
	
	return Promise.all(results).catch( () => { console.log("failed load"); } );
}


//renderAll
function renderAll() { 
	console.log("rendering");//>:<
	let instructions = myWasm.get_instruction().split("|");
	instructions.forEach((function(entry) {
		let instruction = entry.split(",");
		g_gameContext.drawImage(g_canvases[instruction[0]],instruction[1],instruction[2]); 
	}));
	
}


export async function bar() { //>:< rename
	let resourceLocations;
	
	await Promise.all( [ 
		myWasm.default(),
		fetch("./resources.json")
			.then(r => r.json())
			.then(r => { console.log(r); resourceLocations = r }) // >:<
	]);
	
	load_images(resourceLocations).then( 
		() => { 
			document.body.appendChild(g_gameCanvas);
			renderAll(); },
		() => { alert("load_images FAILED"); } // >:<
	);
	
}
