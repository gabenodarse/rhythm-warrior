import * as fileReader from "./file-reader.js";

// TODO handle errors
export async function loadDefaultDB(){
	return fetch("music-mercenary.db")
		.then( res => {
			return fileReader.loadDB(res);
		}
	);
}

// TODO handle errors
export function loadSong(songID, db){
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