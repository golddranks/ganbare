/// <reference path="typings/globals/jquery/index.d.ts" />
/// <reference path="typings/globals/howler/index.d.ts" />

$(function() {

function accentuate(word: string, showAccent) {

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
	questionSectionFlexContainer.show();
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


function setLoadError(audioElement, elementName, closureQuiz) {

	audioElement.on("loaderror", function (id, e) {

		if (closureQuiz !== null && currentQuiz !== closureQuiz) { this.off(); return false; };
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
	if (exercise.sent) { return; };
	exercise.sent = true;
	console.log("answerExercise! isCorrect: ", isCorrect, " exercise: ", exercise);
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
	var answeredInstant = Date.now();
	function postAnswerExercise() {
		console.log("postAnswerExercise", exercise);
		var jqxhr = $.post("/api/next_quiz", {
			type: "exercise",
			asked_id: exercise.asked_id,
			answer_level: isCorrect ? 1 : 0,
			times_audio_played: timesAudioPlayed,
			active_answer_time: exercise.pronouncedInstant - exercise.askedInstant,
			reflected_time: answeredInstant - exercise.pronouncedInstant,
			full_answer_time: answeredInstant - exercise.askedInstant,
			full_spent_time: answeredInstant - exercise.startedInstant,
		}, function(result) {
			clearError();
			console.log("postAnswerExercise: got result");
			nextQuestion(result);
		});
		jqxhr.fail(function(e) {
			console.log("postAnswerExercise: failed")
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
	var wordAnsweredInstant = Date.now();
	function postAnswerWord() {
		var jqxhr = $.post("/api/next_quiz", {
			type: "word",
			asked_id: word.asked_id,
			times_audio_played: timesAudioPlayed,
			active_answer_time: word.active_answer_time,
			full_spent_time: wordAnsweredInstant - word.wordShownInstant,
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
	var answeredInstant = Date.now();
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
			active_answer_time: answeredInstant - question.playbackEndedInstant,
			full_answer_time: answeredInstant - question.playbackStartedInstant,
			full_spent_time: answeredInstant - question.startedInstant,
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
	question.startedInstant = Date.now();

	question.answers.forEach(function(a, i) {
		var isCorrect = (question.right_a === a[0])?true:false;
		spawnAnswerButton(a[0], a[1], isCorrect, question);
	});
	var qAudio = new Howl({ src: ['/api/audio.mp3?'+question.asked_id]});

	play_button.one('click', function() {
		console.log("question started");
		question.playbackStartedInstant = Date.now();
	   	questionStatus.slideUp(normalSpeed);
		questionSection.css("min-height", questionSection.css("height")); // For mobile/xxsmall (questionSection is centered in a flexbox)
		main.css("min-height", main.css("height")); // For desktop (main changes size)
		avatar.fadeOut(quiteFast);

		qAudio.once('end', function() {
			question.playbackEndedInstant = Date.now();
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
/* pub struct WordJson {
    quiz_type: &'static str,
    asked_id: i32,
    word: String,
    explanation: String,
    show_accents: bool,
} */
function showWord(word) {
	wordSection.show();
	word_avatar.hide();
	console.log("showWord!");
	buttonSection.show();
	wordShowKana.html(accentuate(word.word, word.show_accents));
	wordExplanation.html(word.explanation);
	var wordAudio = new Howl({ src: ['/api/audio.mp3?'+word.asked_id]});

	word.wordShownInstant = Date.now();

	var activityStarted = Date.now();
	var activeNow = true;
	var activityThreshold_ms = 8000;
	word.active_answer_time = 0;

	function userInactivated() {
		activeNow = false;
		word.active_answer_time += Date.now() - activityStarted;
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
		word.active_answer_time += Date.now() - activityStarted;
		console.log("Active answer time was!", word.active_answer_time);
		clearTimeout(userInactiveTimer);
		answerWord(word);
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

/* pub struct ExerciseJson {
    quiz_type: &'static str,
    asked_id: i32,
    word: String,
    explanation: String,
} */

function showExercise(exercise) {
	wordSection.show();
	console.log("showExercise!");
	word_avatar.show();
	word_avatar.css('opacity', '0');
	wordStatus.text("Äännä parhaasi mukaan!").show();
	wordShowSection.hide();
	wordStatus.slideDown(normalSpeed, function() { word_avatar.fadeTo(normalSpeed, 1); });
	exercise.startedInstant = Date.now();
	word_play_button.one('click', function() {word_avatar.fadeOut(quiteFast, function() {

	console.log("exercise started");
	wordShowSection.slideDown();
	exerciseOkButton.show();
	buttonSection.show();
	wordShowKana.html(accentuate(exercise.word, false));
	wordExplanation.html(exercise.explanation);

	var exerciseAudio = new Howl({ src: ['/api/audio.mp3?'+exercise.asked_id]});

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

	exercise.answered = false;
	exerciseOkButton.one("click", function() {
		exercise.answered = true;
		wordShowKana.html(accentuate(exercise.word, true));
		exerciseAudio.play();
		timesAudioPlayed++;
		exercise.pronouncedInstant = Date.now();
		buttonSection.slideUp(normalSpeed, function() {
			exerciseOkButton.hide();
		});
		wordStatus.slideUp(normalSpeed);
	});

	topmessage.text("Vastausaikaa 8 s");
	topmessage.fadeIn();

	window.setTimeout(function() { if (exercise.answered) {return}; topmessage.text("Vastausaikaa 3 s"); }, 5000);
	window.setTimeout(function() { if (exercise.answered) {return}; topmessage.text("Vastausaikaa 2 s"); }, 6000);
	window.setTimeout(function() { if (exercise.answered) {return}; topmessage.text("Vastausaikaa 1 s"); }, 7000);
	window.setTimeout(function() {
		if (exercise.answered) {return};
		topmessage.fadeOut(); 
		exercise.pronouncedInstant = Date.now();
		answerExercise(false, exercise);
	}, 8000);
	
	exercise.askedInstant = Date.now();
	setTimeout(function() {
		wordExplanation.addClass("imageLoaded");
	}, 200);
	})});


	wordSectionSlideContainer.slideDown(normalSpeed);

}

function showQuiz(quiz) {
	console.log("showQuiz!");
	cleanState();

	if (quiz === null) {
		console.log("No cards!");
		questionSection.show();
		questionSectionFlexContainer.show();
		questionStatus.text("Ei ole mitään kysyttävää ☹️");
		questionStatus.slideDown(normalSpeed);
		avatar.fadeOut(superFast);
		return;
	} else if (new Date(quiz.due_date) > new Date()) {
		console.log("BreakTime! Breaking until: ", new Date(quiz.due_date));
		avatar.fadeOut(superFast);
		breakTime(quiz);
		breakTimeWaitHandle = window.setInterval(function() { breakTime(quiz); }, 1000);
		return;
	}
	currentQuiz = quiz;
	quiz.answered = false;

	if (quiz.quiz_type === "question") {
		showQuestion(quiz);
	} else if (quiz.quiz_type === "word") {
		showWord(quiz);
	} else if (quiz.quiz_type === "exercise") {
		showExercise(quiz);
	} else if (quiz.quiz_type === "future") {
		start();
	} else {
		bugMessage(quiz);
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
