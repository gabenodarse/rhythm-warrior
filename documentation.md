# Documentation:

## file naming
scripts with an uppercase naming hold a class of the same name. Lower case holds multiple classes/functions. 
Game.js holds Game class and a subclass Editor

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

## src/slash.rs
contains the Slash class, initialized when the player slashes

## src/dash.rs
contains the Dash class, initialized when the player dashes

## resources.json / src/resources.rs / graphics
each key in the resources.json file represents 1 graphic or animation. 
to add a graphic or animation, add the key to the resources.json file and create an array with the associated images
then, modify resources.rs such that: 
	keys are in the corresponding order 
	max count of the new graphic/animation and its size are as desired

graphics are displayed by passing the graphic id and frame number (along with transformation flags) to js
when the frame number is passed to js, the script does a modulus with the total number of frames to determine which frame to present

render() functions within graphics.js:
graphic instructions are accessed by the script through a direct pointer to memory (may cause bugs later, but is performant)
graphics can be displayed with WebGL or with canvases. WebGL uses the gpu, so is better for some computers
