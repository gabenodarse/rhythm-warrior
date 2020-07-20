import * as MIDIReader from "./read-MIDI.js";
import * as sqljs from "./sql-wasm.js";
import * as wasm from "./pkg/music_mercenary.js";
import * as graphics from "./graphics.js";

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

// load all images from files before returning a WebGLGraphics object from those images
// !!! add error handling (timeout on image loading?)
Loader.prototype.loadGraphics = async function(type, screenDiv){
	
	if(!this.resourceLocations){
		throw Error("unknown (unloaded) resource locations");
	}
	
	if(type != "canvases" && type != "webGL"){
		throw Error("unspecified graphics type");
	}
	
	let resourceLocations = this.resourceLocations;
	
	let numGraphics = wasm.num_graphic_groups();
	if(Object.keys(resourceLocations).length != numGraphics){
		throw Error("Expected number of graphics " + numGraphics +
			" and number of resource locations " + Object.keys(resourceLocations).length + " do not match");
	}
	
	let numLoaded = 0;
	let images = new Array(numGraphics);
	let done;
	let p = new Promise((res, rej) => {
		done = res;
	});
	let imgLoaded = function(){
		++numLoaded;
		if(numLoaded == numGraphics){
			if(type == "webGL"){
				done(new graphics.WebGLGraphics(images, screenDiv));
			}
			else if(type == "canvases"){
				done(new graphics.CanvasGraphics(images, screenDiv));
			}
			else{ throw Error(); }
		}
	}
	
	for(const resourcesKey in resourceLocations){
		images[ wasm.GraphicGroup[resourcesKey] ] = new Image();
		images[ wasm.GraphicGroup[resourcesKey] ].onload = imgLoaded;
		images[ wasm.GraphicGroup[resourcesKey] ].src = resourceLocations[resourcesKey];;
	}
	
	return p;
}