/// <reference path="typings/globals/jquery/index.d.ts" />
/// <reference path="typings/globals/howler/index.d.ts" />
/// <reference path="recorder.d.ts" />

$(function() {

function createSemaphore(count: number) : (argument?: any) =>void {

	var semaphore = count;
	var closureCallback: (argument: any)=>void = null;
	var closureArg: any = null;

	return (argument?: any) => {
		if (closureCallback === null && typeof argument === "function") {
			closureCallback = argument;
		} else if (closureArg === null) {
			closureArg = argument;
		};
		semaphore--;
		if (semaphore > 0) { return; };
		closureCallback(closureArg);
	};
}

function connectionFailMessage(e) : void {
	console.log("Bug?", e);
	errorSection.show();
	errorStatus.text("Palvelimeen ei saada yhteyttä :(");
	setTimeout(function() { errorStatus.html("Palvelimeen ei saada yhteyttä :(<br>Kokeillaan uudestaan..."); },2000);
	main.addClass("errorOn");
}

function errorMessage(e) : void {
	errorSection.show();
	errorStatus.html(e);
	main.addClass("errorOn");
}

function clearError() : void {

	errorSection.hide();
	main.removeClass("errorOn");
}

var errorSection = $("#errorSection");
var errorStatus = $("#errorStatus");
var main = $("#main");
let global_rec = null;

function startRecording(eventName: string, callback: (recording: boolean, startCB: ()=>void, finishedCB: ()=> void, doneCB: (afterDone: (argument: any)=>void)=> void)=> void) {
	if (Recorder.isRecordingSupported()) {
		let rec;
		if (global_rec === null) {
			console.log("Starting a new recorder stream.");
			rec = new Recorder({encoderPath: "/static/js/encoderWorker.min.js", leaveStreamOpen: true });
		} else {
			console.log("Using an already started recorder stream.");
			rec = global_rec;
		}

		function finishedCB() {
			console.log("Stopping recording.");
			rec.stop();
		}

		function startCB() {
			console.log("Start recording.");
			rec.start();
		}

		let doneSemaphore = createSemaphore(2);

		function doneCB(afterDone: ()=>void) {
			doneSemaphore(afterDone);
		}

		function readyListener(ev) {
			rec.removeEventListener("streamReady", readyListener);
			clearError();
			console.log("Recording stream ready! (Not recording yet.)");
			callback(true, startCB, finishedCB, doneCB);
		}

		function dataAvailListener(ev: RecordingDataAvailableEvent) {
			let random_token = Math.random().toString().slice(2);
			rec.removeEventListener("dataAvailable", dataAvailListener);
			console.log("Recorded data is available!", ev);

			function sendAudioData() {
				$.ajax({
					type: 'POST',
					url: "/api/mic_check?"+random_token,
					processData: false,
					contentType: 'application/octet-stream',
					data: ev.detail,
					success: function() {
						clearError();
						console.log("Recorded audio saved successfully!");
						doneSemaphore(random_token);
					}, 
					error: function(err) {
						connectionFailMessage(err);
						console.log("Error with saving recorded audio!");
						setTimeout(sendAudioData, 2000);
					},
				});
			}
			sendAudioData();
		}

		rec.addEventListener("streamReady", readyListener);
		rec.addEventListener("dataAvailable", dataAvailListener);
	
		if (global_rec === null) {
			console.log("Init stream");

			rec.addEventListener( "streamError", (err: ErrorEvent) => {
				errorMessage("Virhe alustaessa nauhoitusta: "+err.error.message);
			});

			errorMessage("Tarvitsemme selaimesi nauhoitusominaisuutta!<br>Ole hyvä ja myönnä lupa nauhoitukselle.");
			global_rec = rec;
			rec.initStream();
		} else {
			callback(true, startCB, finishedCB, doneCB);
		}
	} else {
		callback(false, ()=>{}, finishedCB, (afterDone: ()=>void)=>{ afterDone(); });
	}

}

function checkMic() {
	startRecording("miccheck", (recording, start_rec, finished_rec, after_done_rec) => {
		if ( ! recording) {
			errorMessage("Selaimesi ei tue äänen nauhoitusta!<br>Kokeile Firefoxia tai Chromea.");
		}
		start_rec();

		console.log("Setting up the rec done button.");

		$("#recDone").prop('disabled', false);
		$("#recDone").one('click',function() {
			$("#recDone").prop('disabled', true);
			finished_rec();
			after_done_rec((random_token) => {
				console.log("Recording done. Random token:", random_token);
				$("#micCheckExplanation").hide();
				$("#micCheckOk").show();
				// HTML5 is required because Chrome doesn't support audio/ogg; codecs=opus without it
				let recording = new Howl({ src: ["/api/mic_check.ogg?"+random_token], format: ['opus'], html5: true});
				recording.play();
			});
		});
	})
}

$("#checkMic").click(function() {
	$("#pretestExplanation").hide();
	$("#micCheckOk").hide();
	$("#micCheckExplanation").show();
	$("#recStart").prop('disabled', false);
	$("#recDone").prop('disabled', true);
});

$("#checkMicAgain").click(function() {
	$("#pretestExplanation").hide();
	$("#micCheckOk").hide();
	$("#micCheckExplanation").show();
	$("#recStart").prop('disabled', false);
	$("#recDone").prop('disabled', true);
});

$("#recStart").click(function() {
	$("#recStart").prop('disabled', true);
	checkMic();
});

});

