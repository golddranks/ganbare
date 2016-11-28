/// <reference path="typings/globals/jquery/index.d.ts" />
/// <reference path="typings/globals/howler/index.d.ts" />

$(function() {

function accentuate(word: string) {

	var empty = '<span class="accent">';
	var middle = '<span class="accent"><img src="/static/images/accent_middle.png" style="display:none;">';
	var start = '<span class="accent"><img src="/static/images/accent_start.png" style="display:none;">';
	var end = '<span class="accent"><img src="/static/images/accent_end.png" class="accent" style="display:none;">';
	var flat_end = '<span class="accent"><img src="/static/images/accent_end_flat.png" style="display:none;">';
	var start_end = '<span class="accent"><img src="/static/images/accent_start_end.png" style="display:none;">';
	var start_end_flat = '<span class="accent"><img src="/static/images/accent_start_end_flat.png" style="display:none;">';
	var start_end_flat_short = '<span class="accent"><img src="/static/images/accent_start_end_flat_short.png" style="display:none;">';
	var peak = '<span class="accent"><img src="/static/images/accent_peak.png" style="display:none;">';
	
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



/* general things */
var bitSlow = 600;
var normalSlow = 500;
var normalSpeed = 400;
var quiteFast = 200;
var superFast = 100;

var main = $("#main");
var errorSection = $("#errorSection");
var errorStatus = $("#errorStatus");
var breakTimeWaitHandle = null;
let currentQuestion = null;
var activeAnswerTime = null;
var fullAnswerTime = null;
var timesAudioPlayed = 0;
var correct = new Howl({ src: ['/static/sfx/correct.m4a', '/static/sfx/correct.mp3']});
var wrong = new Howl({ src: ['/static/sfx/wrong.m4a', '/static/sfx/wrong.mp3']});
var bell = new Howl({ src: ['/static/sfx/bell.m4a', '/static/sfx/bell.mp3']});
var speakerIconTeal = $("#speakerIconTeal");
var speakerIconPink = $("#speakerIconPink");

/* question-related things */
var prototypeAnswer = $(".answer").remove();
prototypeAnswer.show();
var avatar = $("#quiz .avatar");
var questionSection = $("#questionSection");
var questionSectionFlexContainer = $("#questionSectionFlexContainer");
var answerList = $(".answerList");
var questionText = $(".questionText");
var questionExplanation = $("#questionExplanation");
var questionStatus = $("#questionStatus");
var play_button = $("#quiz .avatar .imgbutton");
var maru = $("#maru");
var batsu = $("#batsu");
var answerMarks = $(".answerMark");
var topmessage = $(".topmessageparagraph");


/* word- and exercise-related things */
var wordSection = $("#wordSection");
var wordSectionSlideContainer = $("#wordSectionSlideContainer");
var wordShowButton = $("#wordShowButton");
var wordShowKana = $("#wordShowKana");
var wordStatus = $("#wordStatus");
var wordExplanation = $("#wordExplanation");
var soundIcon = $(".soundicon");
var wordOkButton = $("#wordOkButton");
var exerciseOkButton = $("#exerciseOkButton");
var exerciseSuccessButton = $("#exerciseSuccessButton");
var exerciseFailureButton = $("#exerciseFailureButton");
var wordButtonLabel = $("#wordButtonLabel");
var buttonSection = $("#buttonSection");

/* errors */

function bugMessage(e) {
	console.log("Bug?", e);
	errorSection.show();
	errorStatus.text("Server is down or there is a bug :(");
	setTimeout(function() { errorStatus.html("Server is down or there is a bug :(<br>Trying to connect again..."); },2000);
	main.addClass("errorOn");
}

function clearError() {

	errorSection.hide();
	main.removeClass("errorOn");
}

/* menu */

var settingsArea = $("#settings");
var menuButton = $("#menuButton");

function toggleMenu(event) {
	settingsArea.toggle();
	main.toggleClass("menuOn");
	event.stopPropagation(); 
}

function cancelMenu(event) {
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

function cleanState() {
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

function breakTime(question) {
	var dur_seconds = (new Date(question.due_date).getTime() - Date.now())/1000;
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
	questionStatus.slideDown(normalSpeed);
}

function createTwiceSemaphoreToNextQuestion() {

	var semaphore = 2;
	var closureNextOne = null;

	var nextQuestion = function(nextOne) {
		if (closureNextOne === null && nextOne !== null) {
			closureNextOne = nextOne;
		};
		semaphore--;
		if (semaphore > 0) { return; };
		showQuiz(closureNextOne);
	};
	return nextQuestion;
}


function setLoadError(audioElement, elementName, closureQuestion) {

	audioElement.on("loaderror", function (id, e) {

		if (closureQuestion !== null && currentQuestion !== closureQuestion) { this.off(); return false; };
	    console.log("Error with "+elementName+" element! Trying again after 3 secs.");
		bugMessage(e);
		audioElement.off("load").once("load", function() {
			console.log("Managed to load!", audioElement);
			clearError();
		});
		setTimeout(function() {
			audioElement.unload();
			audioElement.load();
		}, 3000);
	});
	
};

setLoadError(correct, "correctSfx", null);
setLoadError(wrong, "wrongSfx", null);

function setWordShowButton(audio) {

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

function answerExercise(isCorrect, exercise) {
	wordShowButton.off('click');
	exerciseFailureButton.off('click');
	exerciseSuccessButton.off('click');
	var nextQuestion = createTwiceSemaphoreToNextQuestion();
	setTimeout(function() {
		wordExplanation.removeClass("imageLoaded");
		wordSectionSlideContainer.slideUp(normalSpeed, function() {
			nextQuestion(null);
		});
	}, bitSlow);
	if (isCorrect) {
		correct.play();
	} else {
		bell.play();
	}
	function postAnswerExercise() {
		var jqxhr = $.post("/api/next_quiz", {
			type: "exercise",
			word_id: exercise.id,
			correct: isCorrect,
			times_audio_played: timesAudioPlayed,
			active_answer_time: activeAnswerTime - fullAnswerTime,
			full_answer_time: Date.now() - fullAnswerTime,
		}, function(result) {
			clearError();
			console.log("postAnswerExercise: got result");
			nextQuestion(result);
		});
		jqxhr.fail(function(e) {
			bugMessage(e);
			setTimeout(postAnswerExercise, 3000);
		});
	};
	postAnswerExercise();
};

function answerWord(word) {
	wordExplanation.removeClass("imageLoaded");
	wordShowButton.off('click');
	var nextQuestion = createTwiceSemaphoreToNextQuestion();
	setTimeout(function() {
		wordSectionSlideContainer.slideUp(normalSpeed, function() {
			wordShowButton.focusout();
			nextQuestion(null);
		});
	}, normalSlow);
	function postAnswerWord() {
		var jqxhr = $.post("/api/next_quiz", {
			type: "word",
			word_id: word.id,
			times_audio_played: timesAudioPlayed,
			time: Date.now() - activeAnswerTime,
		}, function(result) {
			clearError();
			console.log("postAnswerWord: got result");
			nextQuestion(result);
		});
		jqxhr.fail(function(e) {
			bugMessage(e);
			setTimeout(postAnswerWord, 3000);
		});
	};
	postAnswerWord();
};

function answerQuestion(ansId, isCorrect, question, button) {
	if (question.answered) { return; };
	question.answered = true;
	$(this).addClass("buttonHilight");
	var mark = null;
	var activeATime = Date.now() - activeAnswerTime;
	var fullATime = Date.now() - fullAnswerTime;
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
	questionStatus.show();
	questionExplanation.hide();
	var nextQuestion = createTwiceSemaphoreToNextQuestion();
	var top = 0;
	if (button === null) {
		top = answerList.height()/2;
	} else {
		top = $(button).position().top + ($(button).height()/2);
	}
	mark.css("top", top + "px");
	mark.show();
	mark.removeClass("hidden");
	setTimeout(function() { mark.fadeOut(normalSpeed); }, 1700);
	setTimeout(function() { answerList.slideUp(normalSpeed, function() {
		topmessage.fadeOut();
		questionExplanation.text("Loading...");
		questionExplanation.slideDown(normalSpeed);
		nextQuestion(null);
	}); }, 2200);

	function postAnswerQuestion() {
		var jqxhr = $.post("/api/next_quiz", {
			type: "question",
			asked_id: question.asked_id,
			answered_qa_id: ansId,
			active_answer_time: activeATime,
			full_answer_time: fullATime,
		}, function(result) {
			clearError();
			console.log("postAnswerQuestion: got result");
			nextQuestion(result);
		});
		jqxhr.fail(function(e) {
			bugMessage(e);
			setTimeout(postAnswerQuestion, 3000);
		});
	};
	postAnswerQuestion();
}

function spawnAnswerButton(ansId, text, isCorrect, question) {
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
			answerQuestion(ansId, isCorrect, question, this);
		});
	answerList.append(newAnswerButton);
};


/* pub struct QuestionJson {
    quiz_type: &'static str,
    asked_id: i32,
    explanation: String,
    question: String,
    right_a: i32,
    answers: Vec<(i32, String)>,
} */

function showQuestion(question) {
	console.log(question);
	questionSectionFlexContainer.show();
	questionSection.show();
	questionExplanation.text(question.explanation);
	avatar.show();
	avatar.css('opacity', '0');
	questionExplanation.slideDown(normalSpeed, function() { avatar.fadeTo(normalSpeed, 1); });
	fullAnswerTime = Date.now();

	question.answers.forEach(function(a, i) {
		var isCorrect = (question.right_a === a[0])?true:false;
		spawnAnswerButton(a[0], a[1], isCorrect, question);
	});
	var qAudio = new Howl({ src: ['/api/audio.mp3?'+question.asked_id]});

	play_button.one('click', function() {
	   	questionStatus.slideUp(normalSpeed);
		questionSection.css("min-height", questionSection.css("height")); // For mobile/xxsmall (questionSection is centered in a flexbox)
		main.css("min-height", main.css("height")); // For desktop (main changes size)
		avatar.fadeOut(quiteFast);

		qAudio.once('end', function() {
			activeAnswerTime = Date.now();
			topmessage.text("Vastausaikaa 8 s");
			topmessage.fadeIn();
			questionText.text(question.question);
		
			answerList.slideDown(normalSpeed);
			window.setTimeout(function() { if (question.answered) {return}; topmessage.text("Vastausaikaa 3 s"); }, 5000);
			window.setTimeout(function() { if (question.answered) {return}; topmessage.text("Vastausaikaa 2 s"); }, 6000);
			window.setTimeout(function() { if (question.answered) {return}; topmessage.text("Vastausaikaa 1 s"); }, 7000);
			window.setTimeout(function() {
				if (question.answered) {return};
				topmessage.fadeOut(); 
				answerQuestion(-1, false, question, null);
			}, 8000);
		});
		qAudio.play();
	});

	setLoadError(qAudio, "questionAudio", question);
	
}

function showWord(word) {
	wordSection.show();
	console.log("showWord!");
	wordOkButton.show()
	wordOkButton.one('click', function() { answerWord(word); });
	buttonSection.show();
	wordShowKana.html(accentuate(word.word));
	if (word.show_accents) {
		$(".accent img").show();
	}
	wordExplanation.html(word.explanation);
	var wordAudio = new Howl({ src: ['/api/audio/'+word.audio_id+'.mp3']});

	setLoadError(wordAudio, "wordAudio", word);
	
	wordShowButton.show();
	
	setTimeout(function() { setWordShowButton(wordAudio); wordShowButton.trigger('click');}, 1100);

	timesAudioPlayed++;
	activeAnswerTime = Date.now();

	setTimeout(function() {
		wordExplanation.addClass("imageLoaded");
		wordSectionSlideContainer.slideDown(normalSpeed);
	}, 200);
}

function showExercise(exercise) {
	wordSection.show();
	console.log("showExercise!");
	exerciseOkButton.show();
	wordStatus.text("Äännä parhaasi mukaan:").show();
	buttonSection.show();
	wordShowKana.html(accentuate(exercise.word));
	$(".accent img").hide();
	wordExplanation.html(exercise.explanation);

	var exerciseAudio = new Howl({ src: ['/api/audio/'+exercise.audio_id+'.mp3']});

	setLoadError(exerciseAudio, "exerciseAudio", exercise);
	setWordShowButton(exerciseAudio);

	exerciseSuccessButton.one('click', function() { answerExercise(true, exercise); });

	exerciseFailureButton.one('click', function() { answerExercise(false, exercise); });

	exerciseAudio.once('end', function(){
		setTimeout(function() {
			wordButtonLabel.text("Itsearvio");
			wordButtonLabel.show();
			exerciseFailureButton.show();
			exerciseSuccessButton.show();
			wordShowButton.fadeIn();
			buttonSection.slideDown(normalSpeed);
		}, 1100);
	});

	exerciseOkButton.one("click", function() {
		$(".accent img").fadeIn();
		exerciseAudio.play();
		timesAudioPlayed++;
		activeAnswerTime = Date.now();
		buttonSection.slideUp(normalSpeed, function() {
			exerciseOkButton.hide();
		});
		wordStatus.slideUp(normalSpeed);
	});
	
	
	fullAnswerTime = Date.now();

	setTimeout(function() {
		wordExplanation.addClass("imageLoaded");
		wordSectionSlideContainer.slideDown(normalSpeed);
	}, 200);
}

function showQuiz(question) {
	console.log("showQuiz!");
	cleanState();

	if (question === null) {
		console.log("No cards!");
		questionSection.show();
		questionSectionFlexContainer.show();
		questionStatus.text("Ei ole mitään kysyttävää ☹️");
		questionStatus.slideDown(normalSpeed);
		avatar.fadeOut(superFast);
		return;
	} else if (new Date(question.due_date) > new Date()) {
		console.log("BreakTime!");
		avatar.fadeOut(superFast);
		breakTime(question);
		breakTimeWaitHandle = window.setInterval(function() { breakTime(question); }, 1000);
		return;
	}
	currentQuestion = question;
	question.answered = false;

	if (question.quiz_type === "question") {
		showQuestion(question);
	} else if (question.quiz_type === "word") {
		showWord(question);
	} else if (question.quiz_type === "exercise") {
		showExercise(question);
	} else if (question.quiz_type === "future") {
		start();
	} else {
		bugMessage(question);
	}

}

function start() {
	clearError();
	var jqxhr = $.getJSON("/api/new_quiz", showQuiz);
	jqxhr.fail(function(e) {
		console.log("Connection fails with getJSON. (/api/new_quiz)");
		bugMessage(e);
		setTimeout(start, 3000);
	});
};
start();

});
