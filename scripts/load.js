import * as MIDIReader from "./read-MIDI.js";
import * as wasm from "../pkg/music_mercenary.js";
import {WebGLGraphics} from "./graphics.js";

function MMDatabase(database){
	this.database = database;
}

// TODO if query fails
// returns the song data and brick data of the specified song
MMDatabase.prototype.loadSong = async function(songData){
	// !!! !!! !!! match return value based on songData provided
	let songJSON = this.database[0];
	
	return songJSON;
}

// saves the song (song data and note data) to a json file
// !!! error handling on save. Don't want to lose data
	// check song data fields validity and for existence of bricks
MMDatabase.prototype.saveSong = function(songData, notes){
	let songObject = {
		name: songData.name, 
		artist: songData.artist, 
		difficulty: songData.difficulty, 
		bpm: songData.bpm, 
		brickSpeed: songData.brickSpeed, 
		duration: songData.duration, 
		startOffset: songData.startOffset, 
		timeCreated: songData.timeCreated,
		timeModified: songData.timeModified,
		filename: songData.filename,
		jsonname: songData.jsonname};
		
	songObject.notes = []
	for(let i = 0; i < notes.length; ++i){
		songObject.notes[i] = [
			notes[i].brick_type,
			notes[i].beat_pos,
			notes[i].end_beat_pos,
			notes[i].x_pos,
			notes[i].is_triplet,
			notes[i].is_trailing,
			notes[i].is_leading,
			notes[i].is_hold_note,
			notes[i].approx_time
		];
	}
	
	let songJSON = JSON.stringify(songObject);
	let data = new Blob([songJSON], {type: "application/json"});
	let jsonFile = window.URL.createObjectURL(data);
	
	// !!! if a download doesn't appear, what can you do?
	let link = document.createElement('a');
	link.setAttribute('download', songData.jsonname);
	link.href = jsonFile;
	
	document.body.appendChild(link);
	link.click();
	document.body.removeChild(link);
}

// returns all loaded song data (not including note data / game data)
MMDatabase.prototype.searchSong = function(songData){
	if(!songData){
		// !!! !!! !!! return the data of all loaded songs
		return [{
			name: this.database[0].name,
			artist: this.database[0].artist,
			difficulty: this.database[0].difficulty,
			bpm: this.database[0].bpm,
			brickSpeed: this.database[0].brickSpeed,
			duration: this.database[0].duration,
			startOffset: this.database[0].startOffset,
			timeCreated: this.database[0].timeCreated,
			timeModified: this.database[0].timeModified,
			filename: this.database[0].filename,
			jsonname: this.database[0].jsonname
		}];
	}
}

export function Loader(){
	this.resourceLocations;
}

Loader.prototype.init = async function(){
	await fetch("./graphics-resources.json")
		.then(res => res.json())
		.then(res => { this.resourceLocations = res });
}

// load all images from files before returning a WebGLGraphics object from those images
// !!! add error handling (timeout on image loading?)
Loader.prototype.loadGraphics = async function(screenDiv){
	
	if(!this.resourceLocations){
		throw Error("unknown (unloaded) resource locations");
	}
	
	let resourceLocations = this.resourceLocations;
	
	let numGraphics = wasm.num_graphic_groups();
	if(Object.keys(resourceLocations).length != numGraphics){
		throw Error("Expected number of graphic groups " + numGraphics +
			" and number of resource locations " + Object.keys(resourceLocations).length + " do not match");
	}
	
	let numLoaded = 0;
	let images = new Array(numGraphics);
	let totalImages = 0;
	let done;
	let p = new Promise((res, rej) => {
		done = res;
	});
	
	let imgLoaded = function(){
		++numLoaded;
		if(numLoaded == totalImages){
			done(new WebGLGraphics(images, screenDiv));
		}
	}
	
	// track how many images need to load
	for(const resourcesKey in resourceLocations){
		for(let i = 0; i < resourceLocations[resourcesKey].length; ++i){
			++totalImages;
		}
	}

	// load images
	// !!! creates a new image for every file in resources... many files in resources are duplicates and should not take more data
		// (animations repeating frames)
	for(const resourcesKey in resourceLocations){
		let graphicGroup = wasm.GraphicGroup[resourcesKey];
		images[ graphicGroup ] = [];
		for(let i = 0; i < resourceLocations[resourcesKey].length; ++i){
			images[ graphicGroup ][i] = new Image();
			images[ graphicGroup ][i].onload = imgLoaded;
			images[ graphicGroup ][i].src = "./assets/images/" + resourceLocations[resourcesKey][i];
		}
	}
	
	return p;
}

Loader.prototype.loadDatabase = async function(){
	// !!! !!! !!! fetch all songs in song-data directory
	let mmdb = [];
	let songJSON;
	await fetch("./song-data/ivern.json")
		.then(res => res.json())
		.then(res => { songJSON = res });
		
	mmdb[0] = songJSON;
	return new MMDatabase(mmdb);
}

export async function convertMIDI(MIDIFile){
	let buffer = await MIDIFile.arrayBuffer();
	let bytes = new Uint8Array(buffer, 0, buffer.byteLength);
	let notes = MIDIReader.readMIDI(bytes);
	return notes;
}