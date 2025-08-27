
import * as wasm from "../pkg/rhythm_warrior.js";

// TODO
	// implement webGPU when it becomes an option
// TODO (if canvas graphics are still being used)
	// implement OffscreenCanvas when it becomes an option, or there a way to share contexts between canvases?
	// BG does not need to be refreshed. Separate, static canvas

function SizedTexture(texture, width, height){
	this.texture = texture;
	this.width = width;
	this.height = height;
}

let vertexShaderSource;
let fragementShaderSource;

export function WebGLGraphics(images, screenDiv){
	// members
	this.canvas;
	this.gl;
	this.program;
	this.positionBuffer;
	this.texCoordBuffer;
	this.vao;
	this.textures;

	this.canvas = document.createElement("canvas");;
	this.gl = this.canvas.getContext("webgl2", { premultipliedAlpha: false });
	this.program;
	this.positionBuffer;
	this.textures = new Array(images.length);
	
	const canvas = this.canvas;
	const gl = this.gl;
	let gameDim = wasm.game_dimensions();
	canvas.width = gameDim.x;
	canvas.height = gameDim.y;
	screenDiv.appendChild(canvas);
	
	if(!gl){ // !!! move error and checking to when a choice between canvases and webGL is made
		alert("Unable to initialize WebGL. Your browser or machine may not support it.");
		throw Error("Unable to initialize WebGL. Your browser or machine may not support it.");
	}
	gl.clearColor(0.0, 0.0, 0.0, 0.0);
	gl.clear(gl.COLOR_BUFFER_BIT);
	
	const program = initShaderProgram(gl, vertexShaderSource, fragementShaderSource);
	const positionLoc = gl.getAttribLocation(program, 'a_position');
	const texLoc = gl.getAttribLocation(program, 'a_texCoord');
	
	gl.useProgram(program);

	// set view port and resolution
	let resolutionUniformLocation = gl.getUniformLocation(program, "u_resolution");
	gl.uniform2f(resolutionUniformLocation, gameDim.x, gameDim.y);
	gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);
	
	// enable alpha channel
	// TODO does it calculate 1-alpha for each pixel rendered? if so, how about setting alpha beforehand when loading texture?
	gl.clearColor(1.0, 1.0, 1.0, 1.0);
	gl.clear(gl.COLOR_BUFFER_BIT);
	gl.enable(gl.BLEND);
	gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA); 
	
	// create a position buffer to put the points of two triangles which will comprise the rectangle to be rendered
	// create a texture coordinate buffer to hold the coordinates of where color is being extracted from
	this.positionBuffer = gl.createBuffer();
	this.texCoordBuffer = gl.createBuffer();
	
	// set up the vertex array object (which holds the state of attributes, records which buffer each attribute uses, and how to pull data out of those buffers)
		// enable the attributes
	this.vao = gl.createVertexArray();
	gl.bindVertexArray(this.vao);
	gl.enableVertexAttribArray(positionLoc);
	gl.enableVertexAttribArray(texLoc);
	
	// Tell the attribute how to get data out of positionBuffer (ARRAY_BUFFER)
	gl.bindBuffer(gl.ARRAY_BUFFER, this.positionBuffer);
	var size = 2;          // 2 components per iteration
	var type = gl.FLOAT;   // the data is 32bit floats
	var normalize = false; // don't normalize the data
	var stride = 0;        // 0 = move forward size * sizeof(type) each iteration to get the next position
	var offset = 0;        // start at the beginning of the buffer
	gl.vertexAttribPointer( positionLoc, size, type, normalize, stride, offset);
	
	// Tell the texcoord attribute how to get data out of texcoordBuffer (ARRAY_BUFFER)
	gl.bindBuffer(gl.ARRAY_BUFFER, this.texCoordBuffer);
	var size = 3;          // 2 components per iteration
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

WebGLGraphics.prototype.preRender = function(instructions, wasmMemoryObj){
	const gl = this.gl;
	const positionBuffer = this.positionBuffer;
	const textures = this.textures;
	const pointCount = 6; // Always drawing 2 triangles to make a rectangle (6 points)
	
	// set up for the draws
	gl.bindVertexArray(this.vao);
	let positions = new Float32Array(12);
	let texCoordinates = new Float32Array(18);

	// move and make textures visible
	let len = instructions.num_graphics * 3;
	let f32buf = new Float32Array(wasmMemoryObj.buffer, instructions.graphics_ptr, len);
	let u8buf = new Uint8Array(wasmMemoryObj.buffer, instructions.graphics_ptr, len*4);
	let i = 0;
	while(i < len){
		let graphicIdx = u8buf[i*4];
		let graphicFrame = u8buf[i*4 + 1];
		let graphicFlags = u8buf[i*4 + 2];
		let graphicArg = u8buf[i*4 + 3];

		++i;
		let x = f32buf[i];
		++i;
		let y = f32buf[i];
		++i;
		
		let numFrames = textures[graphicIdx].length;
		let graphicSubID = graphicFrame % numFrames;
		let sizedTexture = textures[graphicIdx][graphicSubID];
		
		let startX = x;
		let startY = y;
		let endX = x + sizedTexture.width;
		let endY = y + sizedTexture.height;
		
		if(graphicFlags & wasm.GraphicFlags.HorizontalFlip){
			startX = endX;
			endX = x;
		}
		if(graphicFlags & wasm.GraphicFlags.VerticalFlip){
			startY = endY;
			endY = y;
		}

		// set the data of the texture coordinate buffer
		let alpha = 1.0;
		if(graphicFlags & wasm.GraphicFlags.Opacity){
			alpha = graphicArg / 255;
		}
		gl.bindBuffer(gl.ARRAY_BUFFER, this.texCoordBuffer);
		texCoordinates[0] = 0.0; texCoordinates[1] = 0.0; texCoordinates[2] = alpha;
		texCoordinates[3] = 1.0; texCoordinates[4] = 0.0; texCoordinates[5] = alpha;
		texCoordinates[6] = 0.0; texCoordinates[7] = 1.0; texCoordinates[8] = alpha;
		texCoordinates[9] = 0.0; texCoordinates[10] = 1.0; texCoordinates[11] = alpha;
		texCoordinates[12] = 1.0; texCoordinates[13] = 0.0; texCoordinates[14] = alpha;
		texCoordinates[15] = 1.0; texCoordinates[16] = 1.0; texCoordinates[17] = alpha;
		gl.bufferData(gl.ARRAY_BUFFER, texCoordinates, gl.STATIC_DRAW);

		// set the data of the position buffer
		gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
		positions[0] = startX; positions[1] = startY;
		positions[2] = endX;   positions[3] = startY;
		positions[4] = startX; positions[5] = endY;
		positions[6] = startX; positions[7] = endY;
		positions[8] = endX;   positions[9] = startY;
		positions[10] = endX;  positions[11] = endY;
		gl.bufferData(gl.ARRAY_BUFFER, positions, gl.STATIC_DRAW);

		gl.bindTexture(gl.TEXTURE_2D, sizedTexture.texture);
		
		gl.drawArrays(gl.TRIANGLES, 0, pointCount);
	}
}

vertexShaderSource = `#version 300 es
    in vec2 a_position;
	in vec3 a_texCoord;
	
	out vec3 v_texCoord;

	uniform vec2 u_resolution;
     
    void main() {
		// convert from pixels to clipspace
		vec2 tmp = a_position / u_resolution;
		vec2 tmp2 = tmp * 2.0;
		vec2 clipSpacePos = tmp2 - 1.0;
	  
		gl_Position = vec4(clipSpacePos * vec2(1, -1), 0, 1);
		
		// update varyings for fragment shader
		v_texCoord = a_texCoord;
    }
`;
fragementShaderSource = `#version 300 es
	// set a default precision
	precision highp float;

	uniform sampler2D u_image;

	in vec3 v_texCoord;

	out vec4 outColor;

	void main() {
		vec4 texture = texture(u_image, v_texCoord.xy);
		outColor = vec4(texture.rgb,texture.a * v_texCoord.z);
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