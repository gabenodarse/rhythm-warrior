

export function renderAll(instructions, canvases, xFactor, yFactor, gameContext) {
	// TODO error handling: check if instructions is an array of PositionedGraphic objects
	instructions.forEach( instruction => {
		gameContext.drawImage(canvases[instruction.g],instruction.x * xFactor,instruction.y * yFactor); 
	});
}
