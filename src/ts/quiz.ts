/// <reference path="typings/globals/jquery/index.d.ts" />
/// <reference path="typings/globals/howler/index.d.ts" />

declare type RecorderEventType = "streamError" | "streamReady" | "dataAvailable" | "start" | "pause" | "resume" | "stop";
interface RecordingDataAvailableEvent {
	detail: Uint8Array,
}

declare class Recorder {
	constructor(config?);
	initStream();
	start();
	stop();
	addEventListener( type: RecorderEventType, listener: (ev) => void, useCapture? );
	static isRecordingSupported(): boolean;
}

$(function() {

function accentuate(word: string, showAccent: boolean) : string {

	if (!showAccent) {
		return word.replace("・", "").replace("*", "");
	}

	var empty = '<span class="accent">';
	var middle = '<span class="accent" style="background-image: url(/static/images/accent_middle.png);">';
	var start = '<span class="accent" style="background-image: url(/static/images/accent_start.png);">';
	var end = '<span class="accent" style="background-image: url(/static/images/accent_end.png);">';
	var flat_end = '<span class="accent" style="background-image: url(/static/images/accent_end_flat.png);">';
	var start_end = '<span class="accent" style="background-image: url(/static/images/accent_start_end.png);">';
	var start_end_flat = '<span class="accent" style="background-image: url(/static/images/accent_start_end_flat.png);">';
	var start_end_flat_short = '<span class="accent" style="background-image: url(/static/images/accent_start_end_flat_short.png);">';
	var peak = '<span class="accent" style="background-image: url(/static/images/accent_peak.png);">';
	
	function isAccentMark(i) {
		return (word.charAt(i) === "*" || word.charAt(i) === "・")
	};

	var accentuated = [""];
	var ended = false;
	for (var i = 0, len = word.length; i < len; i++) {

		if (isAccentMark(i)) {
			continue;
		} else if (word.length === 1) {
			accentuated.push(start_end_flat_short);
		} else if (i === 0 && isAccentMark(i+1)) {
			accentuated.push(start_end);
			ended = true;
		} else if (i === 1 && !ended && isAccentMark(i+1)) {
			accentuated.push(peak);
			ended = true;
		} else if (i === 1 && !ended && i === len-1) {
			accentuated.push(start_end_flat);
		} else if (i === 1 && !ended) {
			accentuated.push(start);
		} else if (i > 1 && !ended && i === len-1) {
			accentuated.push(flat_end);
		} else if (i > 1 && !ended && isAccentMark(i+1)) {
			accentuated.push(end);
			ended = true;
		} else if (i > 1 && !ended && !isAccentMark(i+1)) {
			accentuated.push(middle);
		} else {
			accentuated.push(empty);
		}
		accentuated.push(word.charAt(i));
		accentuated.push("</span>");
	}
	return accentuated.join("");
}

function checkRecordingSupport(): boolean {
	if (testing && !Recorder.isRecordingSupported()) {
		errorMessage("Selaimesi ei tue äänen nauhoitusta!<br>Kokeile Firefoxia tai Chromea.");
		return false;
	}
	return true;
}

function startRecording(eventName: string, callback: (recording: boolean, startCB: ()=>void, finishedCB: ()=> void, doneCB: (afterDone: ()=>void)=> void)=> void) {
	if (Recorder.isRecordingSupported()) {
		let rec = new Recorder({encoderPath: "/static/js/encoderWorker.min.js"});

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

		rec.addEventListener( "streamReady", (ev) => {
			console.log("Recording stream ready! (Not recording yet.)");
			callback(true, startCB, finishedCB, doneCB);
		});
		rec.addEventListener( "dataAvailable", (ev: RecordingDataAvailableEvent) => {
			console.log("Recorded data is available!", ev);
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
		});
		rec.addEventListener( "streamError", (err: ErrorEvent) => {
			errorMessage("Virhe alustaessa nauhoitusta: "+err.error.message);
		});
		rec.initStream();
	} else {
		callback(false, ()=>{}, finishedCB, (afterDone: ()=>void)=>{ afterDone(); });
	}

}


type Quiz = FutureJson | QuestionJson | WordJson | ExerciseJson;

interface FutureJson {
    quiz_type: "future",
    due_date: string,
}

interface QuestionJson {
    quiz_type: "question",
    asked_id: number,
    explanation: string,
    question: string,
    right_a: number,
    answers: [number, string][],
}

interface AnsweredQuestion {
	type: "question",
	asked_id: number,
	answered_qa_id: number,
	active_answer_time: number,
	full_answer_time: number,
	full_spent_time: number,
}

interface WordJson {
    quiz_type: "word",
    asked_id: number,
    word: string,
    explanation: string,
    show_accents: boolean,
}

interface AnsweredWord {
	type: "word",
	asked_id: number,
	times_audio_played: number,
	active_answer_time: number,
	full_spent_time: number,
}

interface ExerciseJson {
    quiz_type: "exercise",
    event_name: string,
    asked_id: number,
    word: string,
    explanation: string,
    must_record: boolean,
}

interface AnsweredExercise {
	type: "exercise",
	asked_id: number,
	answer_level: number,
	times_audio_played: number,
	active_answer_time: number,
	reflected_time: number,
	full_answer_time: number,
	full_spent_time: number,
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


/* general things */
var bitSlow = 600;
var normalSlow = 500;
var normalSpeed = 400;
var quiteFast = 200;
var superFast = 100;

var testing = window["testing"];
var main = $("#main");
var errorSection = $("#errorSection");
var errorStatus = $("#errorStatus");
var breakTimeWaitHandle = null;
let currentQuiz = null;
var timesAudioPlayed = 0;
var correct = new Howl({ src: ['/static/sfx/correct.m4a', '/static/sfx/correct.mp3']});
var wrong = new Howl({ src: ['/static/sfx/wrong.m4a', '/static/sfx/wrong.mp3']});
var bell = new Howl({ src: ['/static/sfx/bell.m4a', '/static/sfx/bell.mp3']});
var speakerIconTeal = $("#speakerIconTeal");
var speakerIconPink = $("#speakerIconPink");

/* question-related things */
var prototypeAnswer = $(".answer").remove();
prototypeAnswer.show();
var avatar = $("#qAvatar");
var questionSection = $("#questionSection");
var questionSectionFlexContainer = $("#questionSectionFlexContainer");
var answerList = $(".answerList");
var questionText = $(".questionText");
var questionExplanation = $("#questionExplanation");
var questionStatus = $("#questionStatus");
var play_button = $("#qStartButton");
var maru = $("#maru");
var batsu = $("#batsu");
var answerMarks = $(".answerMark");
var topmessage = $(".topmessageparagraph");


/* word- and exercise-related things */
var wordSection = $("#wordSection");
var wordSectionSlideContainer = $("#wordSectionSlideContainer");
var wordShowButton = $("#wordShowButton");
var wordShowSection = $(".wordShowSection");
var wordShowKana = $("#wordShowKana");
var wordStatus = $("#wordStatus");
var word_avatar = $("#wordAvatar");
var word_play_button = $("#wordStartButton");
var wordExplanation = $("#wordExplanation");
var soundIcon = $(".soundicon");
var wordOkButton = $("#wordOkButton");
var exerciseOkButton = $("#exerciseOkButton");
var exerciseSuccessButton = $("#exerciseSuccessButton");
var exerciseFailureButton = $("#exerciseFailureButton");
var wordButtonLabel = $("#wordButtonLabel");
var buttonSection = $("#buttonSection");

/* errors */

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


/* app main logic */

function cleanState() : void {
	buttonSection.hide();
	questionSectionFlexContainer.hide();
	wordSectionSlideContainer.hide();
	wordExplanation.html("");
	wordExplanation.removeClass("imageLoaded");
	timesAudioPlayed = 0;
	wordSection.hide();
	questionSection.hide();
	answerMarks.hide();
	exerciseOkButton.hide();
	exerciseFailureButton.hide();
	exerciseSuccessButton.hide();
	wordShowButton.hide();
	wordButtonLabel.hide();
	wordOkButton.hide();
	avatar.hide();
	wordStatus.hide();
	answerMarks.addClass("hidden");
	questionExplanation.text("");
	questionExplanation.hide();
	topmessage.fadeOut();
	answerList.children(".answer")
		.remove();
	answerList.hide();
}

function breakTime(future: FutureJson) : void {
	var dur_seconds = (new Date(future.due_date).getTime() - Date.now())/1000;
	var dur_hours = Math.floor(dur_seconds/3600);
	var dur_minutes_remainder = Math.floor((dur_seconds % 3600) / 60);
	var dur_seconds_remainder = Math.floor((dur_seconds % 3600) % 60);

	if (dur_seconds < 0) {
		// The waiting has ended
		window.clearInterval(breakTimeWaitHandle);
		breakTimeWaitHandle = null;
		questionStatus.slideUp(normalSpeed);
		start();
		return;
	}

	if (dur_hours > 0) {
		questionStatus.html("Tauon paikka!<br>Seuraava kysymys avautuu<br>"
			+ dur_hours +" tunnin ja "+dur_minutes_remainder+" minuutin päästä");
	} else if (dur_hours === 0 && dur_minutes_remainder > 4) {
		questionStatus.html("Tauon paikka!<br>Seuraava kysymys avautuu<br>"
			+ dur_minutes_remainder+" minuutin päästä");
	} else if (dur_hours === 0 && dur_minutes_remainder > 0) {
		questionStatus.html("Tauon paikka!<br>Seuraava kysymys avautuu<br>"
			+ dur_minutes_remainder+" minuutin ja "+ dur_seconds_remainder +" sekunnin päästä");
	} else if (dur_hours === 0 && dur_minutes_remainder === 0 && dur_seconds_remainder > 0) {
		questionStatus.html("Tauon paikka!<br>Seuraava kysymys avautuu<br>"
			+ dur_seconds_remainder +" sekunnin päästä");
	}
	questionSection.show();
	questionSectionFlexContainer.show();
	questionStatus.slideDown(normalSpeed);
}

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


function setLoadError(audioElement: Howl, elementName: string, closureQuiz: Quiz) {

	audioElement.on("loaderror", function (id, e) {

		if (closureQuiz !== null && currentQuiz !== closureQuiz) { this.off(); return false; };
	    console.log("Error with "+elementName+" element! Trying again after 3 secs.");
		connectionFailMessage(e);
		audioElement.off("load").once("load", () => {
			console.log("Managed to load!", audioElement);
			clearError();
		});
		setTimeout(() => {
			audioElement.unload();
			audioElement.load();
		}, 3000);
	});
	
};

setLoadError(correct, "correctSfx", null);
setLoadError(wrong, "wrongSfx", null);

function setWordShowButton(audio: Howl) {

	wordShowButton.off('click').on('click', function() {
		timesAudioPlayed++;
		audio.play(); 
		speakerIconTeal.hide();
		speakerIconPink.show();
	});

	audio.on('end', function() {
		speakerIconTeal.show();
		speakerIconPink.hide();
	});

}

function answerExercise(isCorrect: boolean, exercise: ExerciseJson, quiz_data: quizData) {
	if (quiz_data.sent) { return; };
	quiz_data.sent = true;
	console.log("answerExercise! isCorrect: ", isCorrect, " exercise: ", exercise);
	wordShowButton.off('click');
	exerciseFailureButton.off('click');
	exerciseSuccessButton.off('click');
	var nextQuestion = createSemaphore(2);
	setTimeout(function() {
		wordExplanation.removeClass("imageLoaded");
		wordSectionSlideContainer.slideUp(normalSpeed, function() {
			nextQuestion();
		});
	}, bitSlow);
	if (!testing) {
		if (isCorrect) {
			correct.play();
		} else {
			bell.play();
		}
	}
	var answeredInstant = Date.now();
	function postAnswerExercise() {
		console.log("postAnswerExercise", exercise);
		let answered: AnsweredExercise = {
			type: "exercise",
			asked_id: exercise.asked_id,
			answer_level: isCorrect ? 1 : 0,
			times_audio_played: timesAudioPlayed,
			active_answer_time: quiz_data.pronouncedInstant - quiz_data.askedInstant,
			reflected_time: answeredInstant - quiz_data.pronouncedInstant,
			full_answer_time: answeredInstant - quiz_data.askedInstant,
			full_spent_time: answeredInstant - quiz_data.startedInstant,
		};
		var jqxhr = $.post("/api/next_quiz", answered, function(result) {
			clearError();
			console.log("postAnswerExercise: got result");
			nextQuestion(() => { showQuiz(result) });
		});
		jqxhr.fail(function(e) {
			console.log("postAnswerExercise: failed")
			connectionFailMessage(e);
			setTimeout(postAnswerExercise, 3000);
		});
	};
	postAnswerExercise();
};

function answerWord(word: WordJson, quiz_data: quizData) {
	wordExplanation.removeClass("imageLoaded");
	wordShowButton.off('click');
	var nextQuestion = createSemaphore(2);
	setTimeout(function() {
		wordSectionSlideContainer.slideUp(normalSpeed, function() {
			wordShowButton.focusout();
			nextQuestion();
		});
	}, normalSlow);
	var wordAnsweredInstant = Date.now();
	function postAnswerWord() {
		let answered: AnsweredWord = {
			type: "word",
			asked_id: word.asked_id,
			times_audio_played: timesAudioPlayed,
			active_answer_time: quiz_data.active_answer_time,
			full_spent_time: wordAnsweredInstant - quiz_data.wordShownInstant,
		};
		var jqxhr = $.post("/api/next_quiz", answered, function(result) {
			clearError();
			console.log("postAnswerWord: got result");
			nextQuestion(() => { showQuiz(result) });
		});
		jqxhr.fail(function(e) {
			connectionFailMessage(e);
			setTimeout(postAnswerWord, 3000);
		});
	};
	postAnswerWord();
};

function answerQuestion(ansId: number, isCorrect: boolean, question: QuestionJson, button: JQuery, quiz_data: quizData) {
	if (quiz_data.answered) { return; };
	quiz_data.answered = true;
	$(this).addClass("buttonHilight");
	var mark = null;
	var answeredInstant = Date.now();
	if (!testing) {
		if (isCorrect) {
			mark = maru;
			questionStatus.text("Oikein! Seuraava kysymys.");
			correct.play();
		} else if (ansId > 0) {
			mark = batsu;
			questionStatus.text("Pieleen meni, kokeile uudestaan!");
			wrong.play();
		} else if (ansId === -1) {
			mark = batsu;
			questionStatus.text("Aika loppui!");
			wrong.play();
		}
		mark.css("top", top + "px");
		mark.show();
		mark.removeClass("hidden");
		setTimeout(function() { mark.fadeOut(normalSpeed); }, 1700);
	} else {
		questionStatus.text("Vastattu!");
	}
	questionStatus.show();
	questionExplanation.hide();
	var nextQuestion = createSemaphore(2);
	var top = 0;
	if (button === null) {
		top = answerList.height()/2;
	} else {
		top = $(button).position().top + ($(button).height()/2);
	}
	var timeAfterClick = testing?500:2200; // If we are in testing mode, we don't have to give so much time to reflect on the answer
	setTimeout(function() { answerList.slideUp(normalSpeed, function() {
		topmessage.fadeOut();
		questionExplanation.text("Loading...");
		questionExplanation.slideDown(normalSpeed);
		nextQuestion();
	}); }, timeAfterClick);

	function postAnswerQuestion() {
		let answered: AnsweredQuestion = {
			type: "question",
			asked_id: question.asked_id,
			answered_qa_id: ansId,
			active_answer_time: answeredInstant - quiz_data.playbackEndedInstant,
			full_answer_time: answeredInstant - quiz_data.playbackStartedInstant,
			full_spent_time: answeredInstant - quiz_data.startedInstant,
		};
		var jqxhr = $.post("/api/next_quiz", answered, function(result) {
			clearError();
			console.log("postAnswerQuestion: got result");
			nextQuestion(() => { showQuiz(result) });
		});
		jqxhr.fail(function(e) {
			connectionFailMessage(e);
			setTimeout(postAnswerQuestion, 3000);
		});
	};
	postAnswerQuestion();
}

function spawnAnswerButton(ansId: number, text: string, isCorrect: boolean, question: QuestionJson, quiz_data: quizData) {
	var newAnswerButton = prototypeAnswer.clone();
	var aAudio = null;
	/*
	if (ansAudioId !== null) {
		aAudio = new Howl({ src: ['/api/audio/'+ansAudioId+'.mp3']});
		setLoadError(aAudio, "answerAudio", question);
	}*/
	newAnswerButton.children("button")
		.html(text)
		.one('click', function() {
			if (aAudio !== null) { aAudio.play() };
			answerQuestion(ansId, isCorrect, question, this, quiz_data);
		});
	answerList.append(newAnswerButton);
};


function showQuestion(question: QuestionJson) {
	console.log(question);
	questionSectionFlexContainer.show();
	questionSection.show();
	questionExplanation.text(question.explanation);
	avatar.show();
	avatar.css('opacity', '0');
	questionExplanation.slideDown(normalSpeed, function() { avatar.fadeTo(normalSpeed, 1); });
	let quiz_data: quizData = { startedInstant: Date.now(), answered: false, sent: false };
	quiz_data.startedInstant = Date.now();

	question.answers.forEach(function(a, i) {
		var isCorrect = (question.right_a === a[0])?true:false;
		spawnAnswerButton(a[0], a[1], isCorrect, question, quiz_data);
	});
	var qAudio = new Howl({ src: ['/api/audio.mp3?'+question.asked_id]});

	play_button.one('click', function() {
		console.log("question started");
		quiz_data.playbackStartedInstant = Date.now();
	   	questionStatus.slideUp(normalSpeed);
		questionSection.css("min-height", questionSection.css("height")); // For mobile/xxsmall (questionSection is centered in a flexbox)
		main.css("min-height", main.css("height")); // For desktop (main changes size)
		avatar.fadeOut(quiteFast);

		qAudio.once('end', function() {
			quiz_data.playbackEndedInstant = Date.now();
			topmessage.text("Vastausaikaa 8 s");
			topmessage.fadeIn();
			questionText.text(question.question);
		
			answerList.slideDown(normalSpeed);
			window.setTimeout(function() { if (quiz_data.answered) {return}; topmessage.text("Vastausaikaa 3 s"); }, 5000);
			window.setTimeout(function() { if (quiz_data.answered) {return}; topmessage.text("Vastausaikaa 2 s"); }, 6000);
			window.setTimeout(function() { if (quiz_data.answered) {return}; topmessage.text("Vastausaikaa 1 s"); }, 7000);
			window.setTimeout(function() {
				if (quiz_data.answered) {return};
				topmessage.fadeOut(); 
				answerQuestion(-1, false, question, null, quiz_data);
			}, 8000);
		});
		qAudio.play();
	});

	setLoadError(qAudio, "questionAudio", question);
	
}

function showWord(word: WordJson) {
	wordSection.show();
	word_avatar.hide();
	console.log("showWord!");
	buttonSection.show();
	wordShowKana.html(accentuate(word.word, word.show_accents));
	wordExplanation.html(word.explanation);
	var wordAudio = new Howl({ src: ['/api/audio.mp3?'+word.asked_id]});

	let quiz_data: quizData = { startedInstant: Date.now(), answered: false, sent: false };

	quiz_data.wordShownInstant = Date.now();

	var activityStarted = Date.now();
	var activeNow = true;
	var activityThreshold_ms = 8000;
	quiz_data.active_answer_time = 0;

	function userInactivated() {
		activeNow = false;
		quiz_data.active_answer_time += Date.now() - activityStarted;
	}

	var userInactiveTimer = setTimeout(userInactivated, activityThreshold_ms);
	$("body").mousemove( function() {
		if (!activeNow) {
			activeNow = true;
			activityStarted = Date.now();
		}
		clearTimeout(userInactiveTimer);
		userInactiveTimer = setTimeout(userInactivated, activityThreshold_ms);
	});

	wordOkButton.show()
	wordOkButton.one('click', function() {
		$("body").off('mousemove');
		quiz_data.active_answer_time += Date.now() - activityStarted;
		console.log("Active answer time was!", quiz_data.active_answer_time);
		clearTimeout(userInactiveTimer);
		answerWord(word, quiz_data);
	});

	setLoadError(wordAudio, "wordAudio", word);
	
	wordShowButton.show();
	
	setTimeout(function() { setWordShowButton(wordAudio); wordShowButton.trigger('click');}, 1100);

	timesAudioPlayed++;

	setTimeout(function() {
		wordExplanation.addClass("imageLoaded");
		wordSectionSlideContainer.slideDown(normalSpeed);
	}, 200);
}

function showExercise(exercise: ExerciseJson) {
	startRecording(exercise.event_name, (recording_supported, start_recording, finished_recording, when_recording_done) => {
		if (exercise.must_record && !recording_supported) {
			errorMessage("Selaimesi ei tue äänen nauhoitusta!<br>Kokeile Firefoxia tai Chromea.");
			return;
		}
		wordSection.show();
		console.log("showExercise!");
		word_avatar.show();
		word_avatar.css('opacity', '0');
		wordStatus.text("Äännä parhaasi mukaan!").show();
		if (testing) {
			exerciseOkButton.text("Ok, sana äännetty!");
		}
		wordShowSection.hide();
		wordStatus.slideDown(normalSpeed, function() { word_avatar.fadeTo(normalSpeed, 1); });
		let quiz_data: quizData = { startedInstant: Date.now(), answered: false, sent: false };
		word_play_button.one('click', function() {word_avatar.fadeOut(quiteFast, function() {
		
			start_recording();
			console.log("exercise started");
			wordShowSection.slideDown();
			exerciseOkButton.show();
			buttonSection.show();
			wordShowKana.html(accentuate(exercise.word, false));
			wordExplanation.html(exercise.explanation);
		
			var exerciseAudio = new Howl({ src: ['/api/audio.mp3?'+exercise.asked_id]});
		
			setLoadError(exerciseAudio, "exerciseAudio", exercise);
			setWordShowButton(exerciseAudio);
		
			exerciseSuccessButton.one('click', ()=> { answerExercise(true, exercise, quiz_data); });
		
			exerciseFailureButton.one('click', ()=> { answerExercise(false, exercise, quiz_data); });
		
			exerciseAudio.once('end', function(){

				var userAudio = new Howl({ src: ['/api/user_audio.ogg?event='+exercise.event_name+'&last']});
				userAudio.play();
				setTimeout(function() {
					wordButtonLabel.text("Itsearvio");
					wordButtonLabel.show();
					exerciseFailureButton.show();
					exerciseSuccessButton.show();
					wordShowButton.fadeIn();
					buttonSection.slideDown(normalSpeed);
				}, 1100);
			});
		
			quiz_data.answered = false;
			exerciseOkButton.one("click", function() {
				quiz_data.answered = true;
				finished_recording();
				quiz_data.pronouncedInstant = Date.now();
				buttonSection.slideUp(normalSpeed, function() {
					exerciseOkButton.hide();
				});
				wordStatus.slideUp(normalSpeed);

				if (!testing) {
					wordShowKana.html(accentuate(exercise.word, true));
					exerciseAudio.play();
					timesAudioPlayed++;
				} else {
					wordButtonLabel.text("Vastattu!");
					when_recording_done(() => { answerExercise(true, exercise, quiz_data) });
				}
			});
		
			topmessage.text("Vastausaikaa 8 s");
			topmessage.fadeIn();
		
			window.setTimeout(function() { if (quiz_data.answered) {return}; topmessage.text("Vastausaikaa 3 s"); }, 5000);
			window.setTimeout(function() { if (quiz_data.answered) {return}; topmessage.text("Vastausaikaa 2 s"); }, 6000);
			window.setTimeout(function() { if (quiz_data.answered) {return}; topmessage.text("Vastausaikaa 1 s"); }, 7000);
			window.setTimeout(function() {
				if (quiz_data.answered) {return};
				finished_recording();
				topmessage.fadeOut(); 
				quiz_data.pronouncedInstant = Date.now();
				when_recording_done(() => { answerExercise(false, exercise, quiz_data) });
			}, 8000);
			
			quiz_data.askedInstant = Date.now();
			setTimeout(function() {
				wordExplanation.addClass("imageLoaded");
			}, 200);
		})});
	
	
		wordSectionSlideContainer.slideDown(normalSpeed);

	});
}

function startBreak(quiz: FutureJson): void {
	if (new Date(quiz.due_date) > new Date()) {
		console.log("BreakTime! Breaking until: ", new Date(quiz.due_date));
		avatar.fadeOut(superFast);
		breakTime(quiz);
		breakTimeWaitHandle = window.setInterval(function() { breakTime(quiz); }, 1000);
	} else {
		start();
	}
}

function showQuiz(quiz: Quiz): void {
	console.log("showQuiz!");
	cleanState();

	if (quiz === null) {
		if (testing) {
			window.location.href = "/"; // Testing is over, reload the page.
		} else {
			console.log("No cards!");
			questionSection.show();
			questionSectionFlexContainer.show();
			questionStatus.text("Ei ole mitään kysyttävää ☹️");
			questionStatus.slideDown(normalSpeed);
			avatar.fadeOut(superFast);
			return;
		}
	} 

	currentQuiz = quiz;

	if (quiz.quiz_type === "question") {
		showQuestion(quiz);
	} else if (quiz.quiz_type === "word") {
		showWord(quiz);
	} else if (quiz.quiz_type === "exercise") {
		showExercise(quiz);
	} else if (quiz.quiz_type === "future") {
		startBreak(quiz);
	}

}

function start() {
	clearError();
	if (!checkRecordingSupport()) { return; }; // If required but not supported, abort.
	var jqxhr = $.getJSON("/api/new_quiz", showQuiz);
	jqxhr.fail(function(e) {
		console.log("Connection fails with getJSON. (/api/new_quiz)");
		connectionFailMessage(e);
		setTimeout(start, 3000);
	});
};
start();

});
