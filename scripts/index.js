import * as wasm from "../pkg/rhythm_warrior.js";
import * as main from "./main.js";

let wasmMemoryObj;

export function wasmMemory(){
	return wasmMemoryObj; // TODO what if not loaded
}

async function start(){
	window.removeEventListener("click", start);
	let wasmObj = await wasm.default();
	wasmMemoryObj = wasmObj.memory;
	main.run();
}

window.addEventListener("click", start);