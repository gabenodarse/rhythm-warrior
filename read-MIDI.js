
//return object {numBytes: x, val: y}
const readVariableLength = (bytes, i) => {
	
	let beginningIdx = i;
	
	let val = 0;
	//Read variable length block
	while (bytes[i] & 0x80) {
		val *= 0x80;
		val += bytes[i] - 0x80;
		++i;
	}
	
	if(i >= bytes.length){
		throw Error("Read variable length went past the end of the array");
	}
	
	val *= 0x80;
	val += bytes[i];
	++i;
	
	return {numBytes: i - beginningIdx, val: val};
}

class Channel{
	identifier;
	output = [];
	minNote;
	maxNote;
	notesOn = [];
	programNumber;
	constructor(identifier){
		if (identifier > 0x0F) {
			throw Error("Tried to construct an invalid channel");
		}
		this.identifier = identifier;
		this.maxNote = -Infinity;
		this.minNote = Infinity;
	}
}

class Output{
	brickType;
	time;
	xPos;
	duration;
	note;
	velocity;
	isTonal;
	programNumber;
	
	constructor(brickType, time, xPos, duration, note, velocity, isTonal, programNumber){
		this.brickType = brickType;
		this.time = time;
		this.xPos = xPos;
		this.duration = duration;
		this.note = note;
		this.velocity = velocity;
		this.isTonal = isTonal;
		this.programNumber = programNumber;
	}
}

// !!! error checking pass and universality pass are needed
// !!! on error the file should be logged so the source of the error can be tracked
export function readMIDI (bytes) {
	
	let format;
	let numTracks;
	let ticksPerQuarterNote;
	let channels = [];
	for (let i = 0; i < 16; ++i) {
		channels[i] = new Channel(i);
	}
	
	let tempo;
	let runningStatus;
	let channel;
	let time = 0; // in milliseconds since the start of a track
	
	let chunkState;
	let chunkBoundary = 0;
	
	let i = 0;
	while( i < bytes.length ) {
		if (i > chunkBoundary) {
			throw Error("Read past chunk boundary");
		}
		
		// start new chunk
		if (i == chunkBoundary) {
			if(bytes.length - i < 8) {
				console.log("Not in a chunk, but there are not enough bytes left to fill a chunk description at bit: " + i);
				break;
			}
			
			if(bytes[i] == 0x4D && bytes[i+1] == 0x54 && bytes[i+2] == 0x68 && bytes[i+3] == 0x64) { //MThd
				if (typeof(format) != "undefined") {
					throw Error("Format was already defined, but another header chunk exists");
				}
				chunkState = "head";
			} else if (bytes[i] == 0x4D && bytes[i+1] == 0x54 && bytes[i+2] == 0x72 && bytes[i+3] == 0x6B) {
				chunkState = "track";
			} else {
				chunkState = "unknown";
			}
			
			chunkBoundary = i + 8 + bytes[i+4] * 256 * 256 * 256 + bytes[i+5] * 256 * 256 + bytes[i+6] * 256 + bytes[i+7];
			i += 8;
			
			if( chunkBoundary > bytes.length ) {
				console.log("Chunk boundary is beyond bytes length");
			}
			if ( chunkState == "unknown" ) {
				console.log("Unknown chunk state, skipping");
				i = chunkBoundary;
			}
			
			console.log("chunk type " + chunkState + " length " + (chunkBoundary - i));
			time = 0;
			continue;
		}
		
		if(chunkState == "head") {
			if (chunkBoundary - i != 6) {
				console.log("head chunk != 6, == " + (chunkBoundary - i));
			}
			
			if (bytes[i] != 0x0) {
				console.log("First byte of head is non-zero");
			}
			
			if(bytes[i + 1] != 0x0 && bytes[i + 1] != 0x1 && bytes[i + 1] != 0x2) {
				console.log("Format isn't 0, 1, or 2");
			}
			
			if(bytes[i + 4] & 0x80) {
				throw Error('Unhandled time format for MIDI'); // !!!
			}
			
			format = bytes[i + 1];
			numTracks = bytes[i + 2] * 256 + bytes[i + 3];
			ticksPerQuarterNote = bytes[i + 4] * 256 + bytes[i + 5];
			
			if(format == 0 && numTracks != 1) {
				console.log("Format 0 file is described as not having 1 track");
			}
			
			console.log("Format " + format);
			console.log("Num tracks " + numTracks);
			console.log("Ticks per quarter note " + ticksPerQuarterNote);
			
			i = chunkBoundary;
			continue;
		}
			
		if (typeof(format) == "undefined") {
			throw Error("Undefined format with chunk state " + chunkState);
		}
		
		if (chunkState == "track") {
			let deltaTime;
			
			//Read variable length delta time block
			let tmp = readVariableLength (bytes, i);
			i = i + tmp.numBytes;
			deltaTime = tmp.val * tempo / ticksPerQuarterNote;
			deltaTime /= 1000; // convert to milliseconds
			time += deltaTime;
			
			// handle meta event
			if ( bytes[i] == 0xFF ) {
				++i;
				if (bytes[i] == 0x00) {
					console.log("meta event starting with 0xFF00");
					break;
				}
				
				let varLen;
				switch (bytes[i]) {
					case 0x01:
					case 0x02:
					case 0x03:
					case 0x04:
					case 0x05:
					case 0x06:
					case 0x07:
						// !!! classify text based on byte code case
						++i;
						varLen = readVariableLength (bytes, i);
						i += varLen.numBytes;
						console.log( // !!! log somewhere other than console
							"Read meta text starting at " + i + ": " 
							+ String.fromCharCode.apply(null, bytes.slice(i, i + varLen.val))
						);
						i += varLen.val;
						continue;
					
					case 0x20:
						++i;
						if(bytes[i] != 0x01) {
							throw Error("unhandled extra length on Channel prefix meta event");
						} 
						
						++i;
						if(bytes[i] & 0xF0) {
							throw Error("channel prefix not in range 0-15");
						}
						
						channel = bytes[i];
						++i;
						continue;
						
					case 0x2F:
						++i;
						if(bytes[i] != 0x00) {
							throw Error("expected 0 length end of track meta event");
						}
						
						++i;
						if(i != chunkBoundary) {
							throw Error("end of track did not coincide with chunk boundary");
						}
						continue;
					
					case 0x51:
						++i;
						if(bytes[i] != 0x03) {
							throw Error("unhandled extra length on set tempo meta event");
						}
						
						tempo = bytes[i + 1] * 0x10000 + bytes[i + 2] * 0x100 + bytes[i + 3];
						i += 4;
						console.log("Tempo: " + tempo);
						continue;
					
					case 0x54:
						++i;
						if(bytes[i] != 0x05) {
							throw Error("unhandled extra length on SMPTE offset event");
						}
						throw Error("unhandled SMPTE offset meta event"); // !!!
						
					case 0x58:
						++i;
						if(bytes[i] != 0x04) {
							throw Error("unhandled extra length on time signature meta event");
						}
						i += 5;
						continue;
					
					case 0x59:
						++i;
						if(bytes[i] != 0x02) {
							throw Error("unhandled extra length on key signature meta event");
						}
						i += 3;
						continue;
						
					case 0x7f:
						// skip sequencer-specific meta event
						++i;
						varLen = readVariableLength (bytes, i);
						i += varLen.numBytes;
						i += varLen.val;
						continue;
					
					default:
						// skip unknown meta event
						++i;
						varLen = readVariableLength (bytes, i);
						i += varLen.numBytes;
						i += varLen.val;
						continue;
				}
			}
			
			if (bytes[i] | 0x80) {
				
				channel = bytes[i] & 0x0F;
				if ( (bytes[i] | 0xE0) == bytes[i]) {
					runningStatus = "pitch wheel";
				}
				else if ( (bytes[i] | 0xD0) == bytes[i]) {
					runningStatus = "channel pressure";
				}
				else if ( (bytes[i] | 0xC0) == bytes[i]) {
					runningStatus = "program change";
				}
				else if ( (bytes[i] | 0xB0) == bytes[i]) {
					runningStatus = "control change";
				}
				else if ( (bytes[i] | 0xA0) == bytes[i]) {
					runningStatus = "polyphonic key pressure";
				}
				else if ( (bytes[i] | 0x90) == bytes[i]) {
					runningStatus = "note on";
				}
				else {
					runningStatus = "note off";
				}
				++i;
			}
			
			
			// handle changes of running status before the event is complete
			if(bytes[i] & 0x80) {
				continue;
			}
			
			switch(runningStatus) {
				case "note on":
					if(bytes[i+1] & 0x80) {
						i+=1;
						continue;
					}
					
					if(channels[channel].notesOn[bytes[i]]) {
						let noteOn = channels[channel].notesOn[bytes[i]];
						let out = new Output(
							0, noteOn.time, 0, time - noteOn.time, noteOn.note,
							noteOn.velocity, true, noteOn.programNumber
						);
						channels[channel].output.push(out);
					}
					
					channels[channel].notesOn[bytes[i]] = {time: time, note: bytes[i],
														   velocity: bytes[i+1], programNumber: channels[channel].programNumber};
														   
					if (bytes[i] > channels[channel].maxNote) {
						channels[channel].maxNote = bytes[i];
					} else if (bytes[i] < channels[channel].minNote) {
						channels[channel].minNote = bytes[i];
					}
					
					i += 2;
					continue;
					
				case "note off":
					if(bytes[i+1] & 0x80) {
						i+=1;
						continue;
					}
					
					if(channels[channel].notesOn[bytes[i]]) {
						let noteOn = channels[channel].notesOn[bytes[i]];
						let out = new Output(
							0, noteOn.time, 0, time - noteOn.time, noteOn.note,
							noteOn.velocity, true, noteOn.programNumber
						);
						channels[channel].output.push(out);
					}
					
					channels[channel].notesOn[bytes[i]] = false;
					i += 2;
					continue;
					
				case "program change":
					channels[channel].programNumber = bytes[i];
					i += 1;
					continue;
					
				case "control change":
					if(bytes[i+1] & 0x80) {
						i+=1;
						continue;
					}
					console.log("control change event");
					i += 2;
					continue;
					
				default:
					throw Error("unknown running status " + runningStatus);
			}
		}
		
		throw Error("readMIDI shouldn't reach here");
	}
	
	return channels;
	
}
