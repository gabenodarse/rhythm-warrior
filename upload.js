import * as fileReader from "./file-reader.js";

let db;
let notes;

async function handleMIDIUpload (evt) {
	notes = await fileReader.convertMIDI(this.files[0]);
}

async function handleDBUpload (evt) {
	db = await fileReader.loadDB(this.files[0]);
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
				
				notes.forEach( note => {
					let sql = "INSERT INTO NOTES \
						(SongID, BrickType, Time, XPos, Duration, Note, Velocity, IsTonal, ProgramNumber) \
						VALUES (" 
						+ newSongID + ", "
						+ note.brickType + ", "
						+ note.time / 1000 + ", "
						+ note.xPos + ", "
						+ note.duration / 1000 + ", "
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
