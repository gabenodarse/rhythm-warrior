import * as wasm from "../pkg/music_mercenary.js";
import * as main from "./main.js";

let wasmMemoryObj;

export function wasmMemory(){
	return wasmMemoryObj; // TODO what if not loaded
}

(async function(){
	let wasmObj = await wasm.default();
	wasmMemoryObj = wasmObj.memory;
	main.run();
})()


