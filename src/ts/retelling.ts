/// <reference path="typings/globals/jquery/index.d.ts" />
/// <reference path="typings/globals/howler/index.d.ts" />
/// <reference path="recorder.d.ts" />

$(function() {

var bitSlow = 600;
var normalSlow = 500;
var normalSpeed = 400;
var quiteFast = 200;
var superFast = 100;

/* DOM */
var testing = window["testing"];
let event_name = window["event_name"];
var main = $("#main");
var errorSection = $("#errorSection");
var errorStatus = $("#errorStatus");
var avatar = $("#qAvatar");
var questionSection = $("#questionSection");
var questionSectionFlexContainer = $("#questionSectionFlexContainer");
var questionExplanation = $("#questionExplanation");
var questionStatus = $("#questionStatus");
var answerList = $(".answerList");
var questionText = $(".questionText");
var play_button = $("#qStartButton");
var questionText = $(".questionText");
var answerButton = $("#answerButton");
var retellingImage = $("#retellingImage");
var buttonSection = $("#buttonSection");

/* menu */

var settingsArea = $("#settings");
var menuButton = $("#menuButton");

function toggleMenu(event: Event) : void {
	settingsArea.toggle();
	main.toggleClass("menuOn");
	event.stopPropagation(); 
}

function cancelMenu(event: Event): void {
	settingsArea.hide();
	main.removeClass("menuOn");
	event.stopPropagation(); 
}

settingsArea.hide();
settingsArea.click(cancelMenu);
$("body").click(cancelMenu);
menuButton.click(toggleMenu);
$("#settingsMenu").click(function( event ) { event.stopPropagation(); });

/* helpers */

function createSemaphore(count: number) : (callback?: ()=>void) =>void {

	var semaphore = count;
	var closureCallback: ()=>void = null;

	return (callback?: ()=>void) => {
		if (closureCallback === null && callback !== undefined) {
			closureCallback = callback;
		};
		semaphore--;
		if (semaphore > 0) { return; };
		closureCallback();
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

/* recording */

function checkRecordingSupport(): boolean {
	if (testing && !Recorder.isRecordingSupported()) {
		errorMessage("Selaimesi ei tue äänen nauhoitusta!<br>Kokeile Firefoxia tai Chromea.");
		return false;
	}
	return true;
}

let global_rec = null;

function startRecording(eventName: string, callback: (recording: boolean, startCB: ()=>void, finishedCB: ()=> void, doneCB: (afterDone: ()=>void)=> void)=> void) {
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
			$(".recordIcon").removeClass("recordingNow");
			console.log("Stopping recording.");
			rec.stop();
		}

		function startCB() {
			$(".recordIcon").addClass("recordingNow");
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
			rec.removeEventListener("dataAvailable", dataAvailListener);
			console.log("Recorded data is available!", ev);

			if (ev.detail.length > 180000) {
				errorMessage("Ääni oli niin pitkä että sitä ei voida lähettää :(");
				setTimeout(function() { clearError() }, 3000);
				return;
			}

			$.ajax({
				type: 'POST',
				url: "/api/user_audio?event="+eventName,
				processData: false,
				contentType: 'application/octet-stream',
				data: ev.detail,
				success: function() {
					console.log("Recorded audio saved successfully!");
					doneSemaphore();
				}, 
				error: function(err) {
					connectionFailMessage(err);
					console.log("Error with saving recorded audio!");
				},
			});
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

interface quizData {
	startedInstant: number,
	pronouncedInstant?: number,
	askedInstant?: number,
	wordShownInstant?: number,
	active_answer_time?: number,
	playbackStartedInstant?: number,
	playbackEndedInstant?: number,
	answered: boolean,
	sent: boolean,
}

interface Retelling {
	img_src: string,
	audio_src: string,
}

function showRetelling(retelling: Retelling) {
	if (retelling === null) {
		if (testing) {
			window.location.href = "/"; // Testing is over, reload the page.
		} else {
			errorMessage("Bug: The server returned a null value.");
		}
	}
	startRecording(event_name, (recording_supported, start_recording, finished_recording, when_recording_done) => {
		answerList.hide();
		questionText.hide();
		questionText.html('Kerro, mitä kuvassa tapahtuu.<br>Aikaa on max 24 sekuntia.<br>Nauhoitus käynnissä.<img src="/static/images/record.png" class="recordIcon">');
		questionSectionFlexContainer.show();
		questionSection.show();
		questionExplanation.html("Kuuntele, mitä kuvassa tapahtuu.<br>Selitys loppuu äänimerkkiin. Selityksen loputtua<br>on sinun vuorosi kertoa kuulemasi uudestaan.<br>Äänesi nauhoitetaan.");
		avatar.show();
		avatar.css('opacity', '0');
		questionExplanation.slideDown(normalSpeed, function() { avatar.fadeTo(normalSpeed, 1); });
		let quiz_data: quizData = { startedInstant: Date.now(), answered: false, sent: false };
		retellingImage.attr('src', retelling.img_src);
		var retellingAudio = new Howl({ src: [retelling.audio_src]});
		var ping = new Howl({ src: ["/static/sfx/ping.mp3"]});
	
		play_button.one('click', function() {
			console.log("retelling started");
			quiz_data.playbackStartedInstant = Date.now();
		   	questionStatus.slideUp(normalSpeed);
			questionSection.css("min-height", questionSection.css("height")); // For mobile/xxsmall (questionSection is centered in a flexbox)
			main.css("min-height", main.css("height")); // For desktop (main changes size)
			avatar.fadeOut(quiteFast, () => {
				answerList.slideDown(quiteFast);
				retellingAudio.play();
			});
		});

		when_recording_done(() => {
			questionExplanation.text("Vastattu. Seuraava kysymys!");
			questionExplanation.fadeIn();
			setTimeout(() => { questionExplanation.slideUp(normalSpeed, ()=> {
				var jqxhr = $.getJSON("/api/next_retelling?event="+event_name, showRetelling);
				jqxhr.fail(function(e) {
					console.log("Connection fails with getJSON. (/api/next_retelling)");
					connectionFailMessage(e);
					setTimeout(start, 3000);
				});
			}); }, 1800);
		});

		let isRecordingOver = false;
		function recordingOver() {
			if (isRecordingOver) {
				return;
			}
			isRecordingOver = true;
			finished_recording();
			answerList.slideUp();
			buttonSection.slideUp();
		}

		retellingAudio.once('end', () => { setTimeout(() => {
			questionExplanation.slideUp(normalSpeed);
			questionText.slideDown(normalSpeed);
			ping.play();
			start_recording();
			buttonSection.slideDown(quiteFast);

			setTimeout(recordingOver, 24000);
	
		}, 1000);});

		answerButton.one('click', recordingOver);
	});
}


function start() {
	clearError();
	if (!checkRecordingSupport()) { return; }; // If required but not supported, abort.
	var jqxhr = $.getJSON("/api/new_retelling?event="+event_name, showRetelling);
	jqxhr.fail(function(e) {
		console.log("Connection fails with getJSON. (/api/new_retelling)");
		connectionFailMessage(e);
		setTimeout(start, 3000);
	});
};
start();


});
