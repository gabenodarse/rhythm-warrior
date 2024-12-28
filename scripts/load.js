import * as MIDIReader from "./read-MIDI.js";
import * as wasm from "../pkg/music_mercenary.js";
import {WebGLGraphics} from "./graphics.js";

function MMDatabase(database){
	this.database = database;
}

// TODO if query fails
// returns the song data and brick data of the specified song
MMDatabase.prototype.loadSong = function(songData){
	let foundSong;
	
	this.database.forEach( songObject => {
		if(songObject.bpm == songData.bpm
		&& songObject.brickSpeed == songData.brickSpeed
		&& songObject.duration == songData.duration
		&& songObject.startOffset == songData.startOffset
		&& songObject.filename == songData.filename
		&& songObject.jsonname == songData.jsonname){
			foundSong = songObject;
		}
	});
	
	return foundSong;
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
		
	songObject.notes = Array(notes.length);
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
MMDatabase.prototype.searchSong = function(){
	let songs = [];
	this.database.forEach( songObject => {
		songs.push({
			name: songObject.name,
			artist: songObject.artist,
			difficulty: songObject.difficulty,
			bpm: songObject.bpm,
			brickSpeed: songObject.brickSpeed,
			duration: songObject.duration,
			startOffset: songObject.startOffset,
			timeCreated: songObject.timeCreated,
			timeModified: songObject.timeModified,
			filename: songObject.filename,
			jsonname: songObject.jsonname
		});
	});
	return songs;
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
	let ahriObject;
	let ivernObject;
	
	let ahriPromise = fetch("./song-data/ahri.json")
		.then(res => res.json())
		.then(res => { ahriObject = res; });
		
	let ivernPromise = fetch("./song-data/ivern.json")
		.then(res => res.json())
		.then(res => {ivernObject = res;} );
	
	await Promise.all([ivernPromise, ahriPromise]);
	
	let mmdb = [];
	mmdb.push(ahriObject);
	mmdb.push(ivernObject);
		
	return new MMDatabase(mmdb);
}

export async function convertMIDI(MIDIFile){
	let buffer = await MIDIFile.arrayBuffer();
	let bytes = new Uint8Array(buffer, 0, buffer.byteLength);
	let notes = MIDIReader.readMIDI(bytes);
	return notes;
}