import * as wasm from "./pkg/music_mercenary.js";
import * as main from "./main.js";

(async function(){
	await wasm.default();
	main.runEditor();
})()


