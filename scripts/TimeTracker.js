"use strict";

export function TimeTracker(numIncrementsPerCalc){
	this.numIncrementsPerCalc = numIncrementsPerCalc; // number of increments for each average calculation
	this.incrementStartTime; // the start time of the current increment
	this.incrementEndTime; // the end time of the current increment
	this.counter = 0; // counts number of increments since last average calculation
	this.totalTime = 0; // total time for the average calculation
	this.minTime = 1000; // minimum time in this average time calculation
	this.maxTime = 0; // maximum time in this average time calculation
	this.record = []; // record of times
}

TimeTracker.prototype.startTime = function(time){
	this.incrementStartTime = time;
}

TimeTracker.prototype.endTime = function(time){
	this.incrementEndTime = time;
	let timeTaken = this.incrementEndTime - this.incrementStartTime;

	this.counter += 1;	
	this.totalTime += timeTaken;
	
	if(this.minTime > timeTaken){
		this.minTime = timeTaken;
	}

	if(this.maxTime < timeTaken){
		this.maxTime = timeTaken;
	}

	if(this.counter == this.numIncrementsPerCalc){
		this.record.push({time: time, average: this.totalTime / this.numIncrementsPerCalc, min: this.minTime, max: this.maxTime});
		
		this.counter = 0;
		this.totalTime = 0;
		this.minTime = 1000;
		this.maxTime = 0;
	}
}

TimeTracker.prototype.getRecord = function(){
	if(this.record.length == 0){
		return null;
	}

	else{
		return this.record;
	}
}

TimeTracker.prototype.getAggregates = function(){
	if(this.record.length == 0){
		return null;
	}

	let record = this.record;

	let avgMin;
	let medianMin;
	let minMin;
	let maxMin;
	let avgMax;
	let medianMax;
	let minMax;
	let maxMax;
	let avgAvg;
	let medianAvg;
	
	let mins = [];
	let maxes = [];
	let averages = [];
	let minsTotal = 0;
	let maxesTotal = 0;
	let averagesTotal = 0;
	
	for(let i = 0; i < record.length; ++i){
		mins.push(record[i].min);
		minsTotal += record[i].min;

		maxes.push(record[i].max);
		maxesTotal += record[i].max;

		averages.push(record[i].average);
		averagesTotal += record[i].average;
	}

	mins = mins.sort((a, b) => {return a - b;});
	maxes = maxes.sort((a, b) => {return a - b;});
	averages = averages.sort((a, b) => {return a - b;});

	let findMedian = sortedArr => {
		let idx =  Math.floor(sortedArr.length / 2);

		if (sortedArr.length % 2 === 0) {
			return (sortedArr[idx - 1] + sortedArr[idx]) / 2;
		} else {
			return sortedArr[idx];
		}
	}

	
	avgMin = minsTotal / mins.length;
	medianMin = findMedian(mins);
	minMin = mins[0];
	maxMin = mins[mins.length - 1];

	avgMax = maxesTotal / maxes.length;
	medianMax = findMedian(maxes);
	minMax = maxes[0];
	maxMax = maxes[maxes.length - 1];

	avgAvg = averagesTotal / averages.length;
	medianAvg = findMedian(averages);

	return {avgMin: avgMin, medianMin: medianMin, minMin: minMin, maxMin: maxMin,
		avgMax: avgMax, medianMax: medianMax, minMax: minMax, maxMax: maxMax,
		avgAvg: avgAvg, medianAvg: medianAvg,
		length: record.length};
}

TimeTracker.prototype.getMostRecent = function(){
	if(this.record.length == 0){
		return [{}]; // dummy value so function works even if no record is present
	}

	else{
		return this.record.slice(-1);
	}
}

TimeTracker.prototype.reset = function(){
	this.counter = 0;
	this.totalTime = 0;
	this.minTime = 1000;
	this.maxTime = 0;
	this.record = [];
}