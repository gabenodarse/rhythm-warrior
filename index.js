import * as game from "./main.js";
import * as MIDIReader from "./read-MIDI.js";

game.run();

function deriveNoteXPositions (output, minNote, maxNote) {
	let midNote = (maxNote + minNote) / 2;
	
	for (let i = 0; i < output.length; ++i) {
		output[i].xPos = output[i].note - midNote;
	}
}

function convertMIDI(MIDIFile){
	let bytes;
	
	fetch(MIDIFile)
	.then(res => res.arrayBuffer())
	.then(res => { 
		bytes = new Uint8Array(res, 0, res.byteLength);
		let channels = MIDIReader.readMIDI(bytes);
		deriveNoteXPositions (channels[0].output, channels[0].minNote, channels[0].maxNote);
		// >:<
		console.log(channels[0].output);
		console.log(channels[0].minNote + " - " + channels[0].maxNote);
	});
}


convertMIDI("./buns.mid");


