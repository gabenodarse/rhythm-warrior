# Documentation:

# Rust source code

## src/lib.rs
contains some types and functions common to multiple files, and exports to wasm

## src/objects.rs
contains types and constants common to different objects

## src/game.rs
contains the Game class. Game is exported to wasm and through Game all functionality in the game runs

## src/player.rs
contains the Player class. Controls player movement and action, one of the main components of the Game class

## src/brick.rs
contains the Brick class. Information on bricks travelling up the screen

# javascript scripts

## file naming

scripts with an uppercase naming hold a class of the same name. Lower case holds multiple classes/functions. 

## scripts/Game.js

Game.js holds Game class and a subclass Editor. 
Contains logic and controls for running and querying the game
interacts heavily with web assembly bindings
plays and pauses audio

## scripts/Overlay.js

contains Overlay class, which contains majority of DOM components overlayed on top of the game
contains logic for interacting with overlay (components such as menu and home screen) and instructions for what happens when interactions occur
overlay components generally are show/hide, they are not destroyed and reconstructed

## scripts/EventPropagator.js

contains EventPropagator class, which effectively is an umbrella over both the game and overlay
overlay interactions which call for actions on the game send instructions through the EventPropagator class
updates overlay based on game state
also contains the main event handler, which decides how events such as key presses are routed when multiple overlay components are showing

## scripts/index.js and scripts/main.js

entry point, creates and initializes components

## scripts/graphics.js

contains logic for rendering graphics

## scripts/load.js

contains function for loading graphics
contains glue code for interacting with sqlite database

## scripts/readMIDI.js

remnant file that is kept around in case it is useful in the future

# resources.json / src/resources.rs / graphics

each key in the resources.json file represents 1 graphic or animation. 
to add a graphic or animation, add the key to the resources.json file and create an array with the associated images
then, modify resources.rs such that: 
	keys have a corresponding name
	max count of the new graphic/animation and its size are as desired
animation frames are cycled through at 120 fps

graphics are displayed by passing the graphic id and frame number (along with transformation flags) to js
when the frame number is passed to js, the script does a modulus with the 
	total number of frames in the animation to determine which frame to present

render() functions within graphics.js:
graphic instructions are accessed by the script through a direct pointer to memory (may cause bugs later, but is performant)
graphics can be displayed with WebGL or with canvases. WebGL uses the gpu, so is better for some computers
