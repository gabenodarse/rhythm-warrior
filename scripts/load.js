import * as MIDIReader from "./read-MIDI.js";
import * as sqljs from "../sql-wasm.js";
import * as wasm from "../pkg/music_mercenary.js";
import {WebGLGraphics, CanvasGraphics} from "./graphics.js";

export function MMDatabase(database){
	this.database = database;
}

// TODO if query fails
MMDatabase.prototype.loadSong = function(songID){
	if(typeof(songID) != "number"){
		throw Error("invalid song ID");
	}
	
	let sql = "SELECT * FROM BRICKS WHERE SongID=" + songID +";";
	let bricks = this.database.exec(sql);
	sql = "SELECT * FROM SONGS WHERE SongID=" + songID +";";
	let song = this.database.exec(sql);
	
	return {song: song, bricks: bricks};
}

// !!! error handling on save and overwrite. Don't want to lose data
	// check song data fields validity and for existence of bricks
MMDatabase.prototype.saveSong = function(songData, notes){
	let {name, artist, difficulty, bpm, brickSpeed, duration, startOffset, filename} = songData;
	let now = new Date().getTime();
			
	let sql = "INSERT INTO SONGS \
		(Name, Artist, Difficulty, Bpm, BrickSpeed, Duration, StartOffset, TimeCreated, TimeModified, Filename) \
		VALUES (\"" 
		+ name + "\", \""
		+ artist + "\", \""
		+ difficulty + "\", "
		+ bpm + ", "
		+ brickSpeed + ", "
		+ duration + ", "
		+ startOffset + ", "
		+ now + ", "
		+ now + ", \""
		+ filename + "\"" 
		+ ");"
	;
	
	this.database.run(sql);
	let lastInsert = this.database.exec("SELECT last_insert_rowid()");
	let newSongID = lastInsert[0].values[0];
	
	// TODO combine insertion of all notes into 1 large query?
	notes.forEach( note => {
		let {brick_type, beat_pos, end_beat_pos, x_pos, is_triplet, is_trailing, is_leading, is_hold_note, approx_time} = note;
		let sql = `INSERT INTO BRICKS \
			(SongID, BrickType, BeatPos, EndBeatPos, XPos, IsTriplet, IsTrailing, IsLeading, IsHoldNote, ApproxTime) VALUES \
			(${newSongID}, ${brick_type}, ${beat_pos}, ${end_beat_pos}, ${x_pos}, ${is_triplet}, ${is_trailing}, ${is_leading}, \
			${is_hold_note}, ${approx_time});`;
		
		this.database.run(sql);
	});
	
	this.exportDatabase();
}

MMDatabase.prototype.overwriteSong = function(songData, notes){
	// TODO if write doesn't work, data isn't saved while old data in database is lost. Problem?
	let {songID, name, artist, difficulty, bpm, brickSpeed, duration, startOffset, timeCreated, filename} = songData;
	let now = new Date().getTime();
			
	let sql = "INSERT INTO SONGS \
		(Name, Artist, Difficulty, Bpm, BrickSpeed, Duration, StartOffset, TimeCreated, TimeModified, Filename) \
		VALUES (\"" 
		+ name + "\", \""
		+ artist + "\", \""
		+ difficulty + "\", "
		+ bpm + ", "
		+ brickSpeed + ", "
		+ duration + ", "
		+ startOffset + ", "
		+ timeCreated + ", "
		+ now + ", \""
		+ filename + "\"" 
		+ ");"
	;
	
	this.database.run(sql);
	let lastInsert = this.database.exec("SELECT last_insert_rowid()");
	let newSongID = lastInsert[0].values[0];
	
	// TODO combine insertion of all notes into 1 large query?
	notes.forEach( note => {
		let {brick_type, beat_pos, end_beat_pos, x_pos, is_triplet, is_trailing, is_leading, is_hold_note, approx_time} = note;
		let sql = `INSERT INTO BRICKS \
			(SongID, BrickType, BeatPos, EndBeatPos, XPos, IsTriplet, IsTrailing, IsLeading, IsHoldNote, ApproxTime) VALUES \
			(${newSongID}, ${brick_type}, ${beat_pos}, ${end_beat_pos}, ${x_pos}, ${is_triplet}, ${is_trailing}, ${is_leading}, \
			${is_hold_note}, ${approx_time});`;
		
		this.database.run(sql);
	});
	
	// After everything has been inserted, delete old values set the songID of the new values to the old songID
	sql = `DELETE FROM SONGS WHERE SONGID=${songID}`;
	this.database.run(sql);
	sql = `DELETE FROM BRICKS WHERE SONGID=${songID}`;
	this.database.run(sql);
	sql = `UPDATE SONGS SET SONGID=${songID} WHERE SONGID=${newSongID}`;
	this.database.run(sql);
	sql = `UPDATE BRICKS SET SONGID=${songID} WHERE SONGID=${newSongID}`;
	this.database.run(sql);
	
	this.exportDatabase();
}

MMDatabase.prototype.searchSong = function(songData){
	let sql;
	if(!songData){
		sql = "SELECT * FROM SONGS;";
	}
	// TODO combine multiple search criteria
	else{
		let {name, artist, difficulty} = songData;
		if(name) sql = "SELECT * FROM SONGS WHERE name=" + name +";";
		else if(artist) sql = "SELECT * FROM SONGS WHERE artist=" + artist +";";
		else if(difficulty) sql = "SELECT * FROM SONGS WHERE difficulty=" + difficulty +";";
		else{
			throw Error("Invalid song data for search");
		}
	}
	
	return this.database.exec(sql);
}

// !!! if a download doesn't appear, what can you do?
MMDatabase.prototype.exportDatabase = function(){
	let file = this.database.export();
	let data = new Blob([file], {type: "application/vnd.sqlite3"});
	let textfile = window.URL.createObjectURL(data);
	let link = document.createElement('a');
	link.setAttribute('download', 'music-mercenary.db');
	link.href = textfile;
	
	document.body.appendChild(link);
	
	link.click();
	
	document.body.removeChild(link);
}

export function Loader(dbFilename){
	this.resourceLocations;
}

Loader.prototype.init = async function(){
	await fetch("./resources.json")
		.then(res => res.json())
		.then(res => { this.resourceLocations = res });
}

Loader.prototype.loadDatabase = async function(dbFilename){
	if(typeof dbFilename != "string"){ dbFilename = "music-mercenary.db"; }
	
	let mmdb;
	await fetch(dbFilename)
		.then( res => initDB(res) )
		.then( res => mmdb = new MMDatabase(res));
	
	return mmdb;
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
			if(type == "webGL"){
				done(new WebGLGraphics(images, screenDiv));
			}
			else if(type == "canvases"){
				done(new CanvasGraphics(images, screenDiv));
			}
			else{ throw Error(); }
		}
	}
	
	// !!! creates a new image for every file in resources... many files in resources are duplicates and should not take more data
		// (animations repeating frames)
	for(const resourcesKey in resourceLocations){
		let graphicGroup = wasm.GraphicGroup[resourcesKey];
		images[ graphicGroup ] = [];
		for(let i = 0; i < resourceLocations[resourcesKey].length; ++i){
			++totalImages;
			images[ graphicGroup ][i] = new Image();
			images[ graphicGroup ][i].onload = imgLoaded;
			images[ graphicGroup ][i].src = "./assets/images/" + resourceLocations[resourcesKey][i];
		}
	}
	
	return p;
}

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

