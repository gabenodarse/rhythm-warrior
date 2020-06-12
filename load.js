import * as MIDIReader from "./read-MIDI.js";
import * as sqljs from "./sql-wasm.js";
import * as wasm from "./pkg/music_mercenary.js";

export async function convertMIDI(MIDIFile){
	let buffer = await MIDIFile.arrayBuffer();
	let bytes = new Uint8Array(buffer, 0, buffer.byteLength);
	let notes = MIDIReader.readMIDI(bytes);
	return notes;
}

export async function initDB(dbFile){
	let db;
	
	let buffer = await dbFile.arrayBuffer();
	let bytes = new Uint8Array(buffer, 0, buffer.byteLength);
	await sqljs.initSqlJs()
		.then(res => {
			db = new res.Database(bytes);
		})
		.catch(err => {
			console.log(err);// TODO handle errors
		});
	
	return db;
}

// TODO handle errors
export async function loadDB(filename){
	
	return fetch(filename)
		.then( res => {
			return initDB(res);
		}
	);
}

// TODO handle errors
function getSong(songID, db){
	if(!db){
		throw Error("No DB Loaded");
	}
	if(typeof(songID) != "number"){
		throw Error("invalid song ID");
	}
	
	let sql = "SELECT * FROM NOTES WHERE SongID=" + songID +";";
	let contents = db.exec(sql);
	
	// TODO if sql query fails
	
	return contents;
}

export function Loader(dbFilename){
	this.db;
	this.resourceLocations;
}

Loader.prototype.init = async function(dbFilename){
	if(typeof dbFilename != "string"){ dbFilename = "music-mercenary.db"; }
	
	let promises = [
		loadDB(dbFilename)
			.then( res => {this.db = res;}),
		fetch("./resources.json")
			.then(res => res.json())
			.then(res => { this.resourceLocations = res })
	];
	
	await Promise.all(promises);
}

Loader.prototype.getSong = function(song){
	if(typeof song == "string"){
		
	}
	if(typeof song == "number"){
		return getSong(song, this.db);
	}
}

//load all images from files into canvas contexts
Loader.prototype.loadGraphics = async function(){
	
	if(!this.resourceLocations){
		throw Error("unknown (unloaded) resource locations");
	}
	
	let resourceLocations = this.resourceLocations;
	
	let num_graphic_groups = wasm.num_graphic_groups();
	if(Object.keys(resourceLocations).length != num_graphic_groups){
		throw Error("Expected number of graphics " + num_graphic_groups +
			" and number of resource locations " + Object.keys(resourceLocations).length + " do not match");
	}
	
	let results = [];
	let g_canvases = new Array(num_graphic_groups);// >:< pixi
	
	// >:< 
	let bgSize = {x: 1920, y:1080};
	let g_sizeXFactor = 1100 / bgSize.x;
	let g_sizeYFactor = 600 / bgSize.y;
	
	
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
	
	for(const resID in resourceLocations){
		results.push(
			loadImage(resID, wasm.GraphicGroup[resID])
			.catch( onReject )
		);
	}
	
	// TODO can move all catches on results to this Promise.all
	await Promise.all(results).catch( () => { console.log("failed loading images"); } );
	
	return g_canvases;
}