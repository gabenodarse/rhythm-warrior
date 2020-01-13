// TODO
// export renderAll to js
	// renderAll from wasm calls javascript function renderObject OR
	// returns single, long rendering instruction (array or string)
// create the data structure to hold objects in order of layer
// how to manage pixel location in the event of resizing??>:<
// Should SVGs be translated to a jpg/png of an appropriate size?
// Function to load a new image for a given identifier>:<

// How to make sure the instruction for rendering is in sync with the game logic??>:<
// Create a test exporting the expected names of image files for each variant of Graphic and seeing if the actual image name matches

// mark every #[wasm_bindgen] with just javascript or not just javascript


mod utils;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;




// !!! just javascript
#[wasm_bindgen]
extern {
    fn alert(s: &str);
}


// structure with option type for ordering objects>:<


// !!! not just javascript
#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Graphic {
	BALL = 0,
	BRICK = 1,
}


struct Animation {
	frame: Graphic,
	num_frames: u8	//number of frames remaining in the animation
}


// !!! just javascript
#[wasm_bindgen]
pub fn get_instruction() -> String {
	return String::from("0,0,0|1,200,0");
}

