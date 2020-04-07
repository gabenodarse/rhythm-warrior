import * as MIDIReader from "./read-MIDI.js";
import * as sqljs from "./sql-wasm.js";

let db;
let notes;

async function convertMIDI(MIDIFile){
	let buffer = await MIDIFile.arrayBuffer();
	let bytes = new Uint8Array(buffer, 0, buffer.byteLength);
	let notes = MIDIReader.readMIDI(bytes);
	return notes;
}

async function loadDB(dbFile){
	let db;
	
	let buffer = await dbFile.arrayBuffer();
	let bytes = new Uint8Array(buffer, 0, buffer.byteLength);
	await sqljs.initSqlJs()
	.then(res => {
		db = new res.Database(bytes);
	})
	.catch(err => {
		console.log(err);// !!! handle errors
	});
	
	return db;
}

async function handleMIDIUpload (evt) {
	notes = await convertMIDI(this.files[0]);
}

async function handleDBUpload (evt) {
	db = await loadDB(this.files[0]);
	let contents = db.exec("SELECT * FROM Songs");
}

let MIDIFileSelector = document.createElement('input');
MIDIFileSelector.setAttribute('type', 'file');
MIDIFileSelector.addEventListener('change', handleMIDIUpload, false);

let dbFileSelector = document.createElement('input');
dbFileSelector.setAttribute('type', 'file');
dbFileSelector.addEventListener('change', handleDBUpload, false);
	
if (window.File && window.FileReader && window.FileList && window.Blob) {
	// !!! upload song and db through buttons
	document.addEventListener("keydown", event => {
		if (event.keyCode == 49) { // 1
			MIDIFileSelector.click();
		}
	});
	document.addEventListener("keydown", event => {
		if (event.keyCode == 50) { // 2
			dbFileSelector.click();
		}
	});
	document.addEventListener("keydown", event => {
		if (event.keyCode == 51 && db && notes) {
			let name = document.querySelector("#songname");
			let artist = document.querySelector("#artist");
			let difficulty = document.querySelector("#difficulty");
			
			if(name.value && artist.value && difficulty.value){			
				let sql = "INSERT INTO SONGS (Name, Artist, Difficulty) VALUES (\"" 
					+ name.value + "\", \""
					+ artist.value + "\", \""
					+ difficulty.value 
					+ "\");"
				;
				
				db.run(sql);
				let lastInsert = db.exec("SELECT last_insert_rowid()");
				let newSongID = lastInsert[0].values[0];
				console.log(newSongID);
				
				notes.forEach( note => {
					let sql = "INSERT INTO NOTES \
						(SongID, BrickType, Time, XPos, Duration, Note, Velocity, IsTonal, ProgramNumber) \
						VALUES (" 
						+ newSongID + ", "
						+ note.brickType + ", "
						+ note.time + ", "
						+ note.xPos + ", "
						+ note.duration + ", "
						+ note.note + ", "
						+ note.velocity + ", "
						+ note.isTonal + ", "
						+ note.programNumber
						+ ");"
					;
					db.run(sql);
				});
				
				let file = db.export();
				let data = new Blob([file], {type: 'text/plain'});
				let textfile = window.URL.createObjectURL(data);
				let link = document.createElement('a');
				link.setAttribute('download', 'music-mercenary.db');
				link.href = textfile;
				document.body.appendChild(link);
				
				let event = new MouseEvent('click');
				link.dispatchEvent(event);
			}
		}
	});
}
