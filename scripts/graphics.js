
import * as wasm from "../pkg/music_mercenary.js";
import {wasmMemory} from "./index.js";

// !!!
	// implement webGPU when it becomes an option
	// implement OffscreenCanvas when it becomes an option, or there a way to share contexts between canvases?
	// BG does not need to be refreshed. Separate, static canvas

function SizedTexture(texture, width, height){
	this.texture = texture;
	this.width = width;
	this.height = height;
}

function CanvasGroup(fullsizeCanvas, numCanvases, screenDiv){
	this.fullsize = fullsizeCanvas;
	this.nextCanvasIdx = 0; // the index of the next canvas of the group to make visible
	this.canvases = new Array(numCanvases);
	
	for(let i = 0; i < numCanvases; ++i){
		this.canvases[i] = document.createElement("canvas");
		this.canvases[i].width = fullsizeCanvas.width;
		this.canvases[i].height = fullsizeCanvas.height;
		this.canvases[i].getContext("2d").drawImage(fullsizeCanvas, 0, 0, fullsizeCanvas.width, fullsizeCanvas.height);
		this.canvases[i].style.visibility = "hidden"; // >:< visibility vs display performance
		screenDiv.appendChild( this.canvases[i] );
	}
}

let vertexShader;
let fragmentShader;

export function CanvasGraphics(images, screenDiv){
	this.canvases = new Array(images.length);
	// create canvases for each image
	images.forEach( (imgArray,gIdx) => {
		let dimensions = wasm.graphic_size(gIdx);
		let numCanvases = wasm.max_graphics(gIdx);
		this.canvases[gIdx] = [];
		imgArray.forEach( (img, gSubID) => {
			let fullsize = document.createElement("canvas");
			fullsize.width = dimensions.x;
			fullsize.height = dimensions.y;
			fullsize.getContext("2d").drawImage(img, 0, 0, dimensions.x, dimensions.y);
			
			this.canvases[gIdx][gSubID] = new CanvasGroup(fullsize, numCanvases, screenDiv);
		});
	});
}

CanvasGraphics.prototype.render = function(instructions, xFactor, yFactor){
	// clear old canvases
	this.canvases.forEach( graphicGroup => {
		graphicGroup.forEach( canvasGroup => {
			if(canvasGroup.nextCanvasIdx != 0){
				for(let i = canvasGroup.nextCanvasIdx - 1; i >= 0; --i){
					canvasGroup.canvases[i].style.visibility = "hidden";
					canvasGroup.canvases[i].style.opacity = 1.0;
				}
				canvasGroup.nextCanvasIdx = 0;
			}
		});
	});
	
	// move and make canvases visible
	let end = instructions.num_graphics * 3;
	let i32buf = new Int32Array(wasmMemory().buffer, instructions.graphics_ptr, end);
	let u8buf = new Uint8Array(wasmMemory().buffer, instructions.graphics_ptr, end*4);
	let i = 0;
	while(i < end){
		let x = i32buf[i];
		++i;
		let y = i32buf[i];
		++i;
		let graphicIdx = u8buf[i*4];
		let graphicFrame = u8buf[i*4 + 1];
		let graphicFlags = u8buf[i*4 + 2];
		let graphicArg = u8buf[i*4 + 3];
		++i;
		
		let numFrames = this.canvases[graphicIdx].length; // >:< 
		let graphicSubID = graphicFrame % numFrames;
		let canvasGroup = this.canvases[graphicIdx][graphicSubID];
		let idx = canvasGroup.nextCanvasIdx;
		
		// check if the canvas exists
		if(canvasGroup.canvases[idx] == undefined) {
			console.log("not enough canvases for graphicGroup indexed by: " + graphicIdx);
			return;
		}
		
		canvasGroup.canvases[idx].style.left = x * xFactor + "px";
		canvasGroup.canvases[idx].style.top = y * yFactor + "px";
		if(graphicFlags != 0){
			canvasGroup.canvases[idx].style.transform = "";
			if(graphicFlags & wasm.GraphicFlags.HorizontalFlip){
				canvasGroup.canvases[idx].style.transform += "scaleX(-1) ";
			}
			if(graphicFlags & wasm.GraphicFlags.VerticalFlip){
				canvasGroup.canvases[idx].style.transform += "scaleY(-1) ";
			}
			if(graphicFlags & wasm.GraphicFlags.Opacity){
				canvasGroup.canvases[idx].style.opacity = graphicArg / 255;
			}
		}
		else {
			canvasGroup.canvases[idx].style.transform = "";
		}
		
		canvasGroup.canvases[idx].style.visibility = "visible";
		++canvasGroup.nextCanvasIdx;
	}
}

CanvasGraphics.prototype.resize = function(xFactor, yFactor){
	this.canvases.forEach( (graphicGroup, gID) => {
		let dimensions = wasm.graphic_size(gID);
		graphicGroup.forEach( (canvasGroup, gIdx) => {
			let fullsize = canvasGroup.fullsize;
			canvasGroup.canvases.forEach( canvas => {
				canvas.width = dimensions.x * xFactor;
				canvas.height = dimensions.y * yFactor;
				canvas.getContext("2d").drawImage(fullsize, 0, 0, canvas.width, canvas.height);
			})
		});
	});
}

export function WebGLGraphics(images, screenDiv){
	// members
	this.canvas = document.createElement("canvas");;
	this.gl = this.canvas.getContext("webgl");;
	this.program;
	this.positionBuffer;
	this.textures = new Array(images.length);
	
	const canvas = this.canvas;
	const gl = this.gl;
	let gameDim = wasm.game_dimensions();
	canvas.width = gameDim.x;;
	canvas.height = gameDim.y;
	canvas.style.position = "absolute";
	screenDiv.appendChild(canvas);
	
	if(!gl){ // !!! move error and checking to when a choice between canvases and webGL is made
		alert("Unable to initialize WebGL. Your browser or machine may not support it.");
		throw Error("Unable to initialize WebGL. Your browser or machine may not support it.");
	}
	gl.clearColor(0.0, 0.0, 0.0, 0.0);
	gl.clear(gl.COLOR_BUFFER_BIT);
	
	const program = initShaderProgram(gl, vertexShader, fragmentShader);
	const positionLoc = gl.getAttribLocation(program, 'a_position');
	const texLoc = gl.getAttribLocation(program, 'a_texCoord');
	
	gl.useProgram(program);
	gl.enableVertexAttribArray(positionLoc);
	gl.enableVertexAttribArray(texLoc);
	
	// set view port to convert clip space to pixels
	gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);
	
	// enable alpha channel
	// TODO does it calculate 1-alpha for each pixel rendered? if so, how about setting alpha beforehand when loading texture?
	gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA); 
	gl.enable(gl.BLEND);
	
	// create a buffer to put the points of two triangles which will comprise the rectangle to be rendered
	this.positionBuffer = gl.createBuffer();
	const positionBuffer = this.positionBuffer;
	
	// create a buffer to put texture coordinates
	var texcoordBuffer = gl.createBuffer();
	gl.bindBuffer(gl.ARRAY_BUFFER, texcoordBuffer);
	gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([
		0.0,  0.0,
		1.0,  0.0,
		0.0,  1.0,
		0.0,  1.0,
		1.0,  0.0,
		1.0,  1.0,
	]), gl.STATIC_DRAW);
	
	// Tell the attribute how to get data out of positionBuffer (ARRAY_BUFFER)
	gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
	var size = 2;          // 2 components per iteration
	var type = gl.FLOAT;   // the data is 32bit floats
	var normalize = false; // don't normalize the data
	var stride = 0;        // 0 = move forward size * sizeof(type) each iteration to get the next position
	var offset = 0;        // start at the beginning of the buffer
	gl.vertexAttribPointer( positionLoc, size, type, normalize, stride, offset);
	
	// Tell the texcoord attribute how to get data out of texcoordBuffer (ARRAY_BUFFER)
	gl.bindBuffer(gl.ARRAY_BUFFER, texcoordBuffer);
	var size = 2;          // 2 components per iteration
	var type = gl.FLOAT;   // the data is 32bit floats
	var normalize = false; // don't normalize the data
	var stride = 0;        // 0 = move forward size * sizeof(type) each iteration to get the next position
	var offset = 0;        // start at the beginning of the buffer
	gl.vertexAttribPointer( texLoc, size, type, normalize, stride, offset);
	
	// create textures for each image
	images.forEach( (imgArray,gIdx) => {
		let dimensions = wasm.graphic_size(gIdx);
		this.textures[gIdx] = new Array(imgArray.length);
		imgArray.forEach( (img, gSubID) => {
			const texture = gl.createTexture();
			gl.bindTexture(gl.TEXTURE_2D, texture);

			// Set the parameters so we can render any size image (not only powers of 2)
			gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
			gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
			gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
			gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);

			// Upload the image into the texture.
			gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, img);
			
			// TODO dimensions can be stored less redundantly, because each texture in a graphic group has the same dimensions
			this.textures[gIdx][gSubID] = new SizedTexture(texture, dimensions.x, dimensions.y);
		});
	});
}

WebGLGraphics.prototype.resize = function(xFactor, yFactor){
	let gameDim = wasm.game_dimensions();
	this.canvas.width = gameDim.x * xFactor;
	this.canvas.height = gameDim.y * yFactor;
	this.gl.viewport(0, 0, this.canvas.width, this.canvas.height);
}

// >:< duplicate code this and other render function
WebGLGraphics.prototype.render = function(instructions, xFactor, yFactor){
	const gl = this.gl;
	const positionBuffer = this.positionBuffer;
	const textures = this.textures;
	const pointCount = 6; // Always drawing 2 triangles to make a rectangle (6 points)
	
	gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
	let positions = new Float32Array(12);
	
	// move and make textures visible
	let end = instructions.num_graphics * 3;
	let i32buf = new Int32Array(wasmMemory().buffer, instructions.graphics_ptr, end);
	let u8buf = new Uint8Array(wasmMemory().buffer, instructions.graphics_ptr, end*4);
	let i = 0;
	while(i < end){
		let x = i32buf[i];
		++i;
		let y = i32buf[i];
		++i;
		let graphicIdx = u8buf[i*4];
		let graphicFrame = u8buf[i*4 + 1];
		let graphicFlags = u8buf[i*4 + 2];
		++i;
		
		let numFrames = textures[graphicIdx].length;
		let graphicSubID = graphicFrame % numFrames;
		let sizedTexture = textures[graphicIdx][graphicSubID];
		
		gl.bindTexture(gl.TEXTURE_2D, sizedTexture.texture);
		let startX = x * xFactor / this.canvas.width * 2.0 - 1.0;
		let startY = -(y * yFactor / this.canvas.height * 2.0 - 1.0);
		let endX = startX + sizedTexture.width * xFactor / this.canvas.width * 2.0;
		let endY = startY - sizedTexture.height * yFactor / this.canvas.height * 2.0;
		
		if(graphicFlags != 0){
			// !!! have to use a different shader to flip?
			if(graphicFlags & wasm.GraphicFlags.HorizontalFlip){
			
			}
			if(graphicFlags & wasm.GraphicFlags.VerticalFlip){
				
			}
		}
		positions[0] = startX; positions[1] = startY;
		positions[2] = endX;   positions[3] = startY;
		positions[4] = startX; positions[5] = endY;
		positions[6] = startX; positions[7] = endY;
		positions[8] = endX;   positions[9] = startY;
		positions[10] = endX;  positions[11] = endY;
		
		
		gl.bufferData(gl.ARRAY_BUFFER, positions, gl.STATIC_DRAW);
		
		gl.drawArrays(gl.TRIANGLES, 0, pointCount);
	}
}

vertexShader = `
    attribute vec2 a_position;
	attribute vec2 a_texCoord;
	
	varying vec2 v_texCoord;
     
    void main() {
		gl_Position = vec4(a_position, 0.0, 1.0);
		
		// update v_texCoord for fragment shader
		v_texCoord = a_texCoord;
    }
`;
fragmentShader = `
	// fragment shaders don't have a default precision so we need to pick one
	precision mediump float;
	uniform sampler2D u_image;
	varying vec2 v_texCoord;

	void main() {
		gl_FragColor = texture2D(u_image, v_texCoord);
	}
`;

function initShaderProgram(gl, vsSource, fsSource) {
	const vertexShader = loadShader(gl, gl.VERTEX_SHADER, vsSource);
	const fragmentShader = loadShader(gl, gl.FRAGMENT_SHADER, fsSource);

	const shaderProgram = gl.createProgram();
	gl.attachShader(shaderProgram, vertexShader);
	gl.attachShader(shaderProgram, fragmentShader);
	gl.linkProgram(shaderProgram);

	if (!gl.getProgramParameter(shaderProgram, gl.LINK_STATUS)) {
		throw new Error(gl.getProgramParameter(shaderProgram));
	};

	return shaderProgram;
}

function loadShader(gl, type, source) {
  const shader = gl.createShader(type);
  gl.shaderSource(shader, source);
  gl.compileShader(shader);

  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    alert('An error occurred compiling the shaders: ' + gl.getShaderInfoLog(shader));
    gl.deleteShader(shader);
    return null;
  }

  return shader;
}