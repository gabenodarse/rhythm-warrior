import * as MIDIReader from "./read-MIDI.js";
import * as sqljs from "./sql-wasm.js";

export async function convertMIDI(MIDIFile){
	let buffer = await MIDIFile.arrayBuffer();
	let bytes = new Uint8Array(buffer, 0, buffer.byteLength);
	let notes = MIDIReader.readMIDI(bytes);
	return notes;
}

export async function loadDB(dbFile){
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