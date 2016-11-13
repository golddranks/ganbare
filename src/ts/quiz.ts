/// <reference path="typings/globals/jquery/index.d.ts" />

$(function() {
/* init the static machinery */

var main = $("#main");

/* question-related things */
var prototypeAnswer = $(".answer").remove();
prototypeAnswer.show();
var avatar = $("#quiz .avatar");
var questionSection = $("#questionSection");
var wordSection = $("#wordSection");
var answerList = $(".answerList");
var questionText = $(".questionText");
var questionExplanation = $("#questionExplanation");
var questionStatus = $("#questionStatus");
var play_button = $("#quiz .avatar .imgbutton");
var maru = $("#maru");
var batsu = $("#batsu");
var answerMarks = $(".answerMark");
var semaphore = 0;
var topmessage = $(".topmessageparagraph");
var breakTimeWaitHandle = null;
var aAudio = [];
var currentQuestion = null;
var timeUsedForAnswering = null;
var timesAudioPlayed = 0;

var qAudio = <HTMLAudioElement>document.getElementById('questionAudio');
var correct = <HTMLAudioElement>document.getElementById('sfxCorrect');
var wrong = <HTMLAudioElement>document.getElementById('sfxWrong');

/* word-related things */
var wordShow = $("#wordShow");
var wordItself = $("#word");
var wordExplanation = $("#wordExplanation");
var soundIcon = $(".soundicon");
var wordOkButton = $("#wordOkButton");

wordOkButton.click(function() {
	semaphore = 2;
	nextQuestion();
	$.post("/api/next_quiz", {
		type: "word",
		word_id: currentQuestion.id,
		timesAudioPlayed: timesAudioPlayed,
		time: Date.now() - timeUsedForAnswering,
	}, function(result) {
		currentQuestion = result;
		nextQuestion();
	});
});

var wordAudio = <HTMLAudioElement>document.getElementById('wordAudio');
wordShow.click(function() {
	timesAudioPlayed++;
	wordAudio.play(); 
	soundIcon.prop("src", "/static/images/speaker_pink.png");
});

$(wordAudio).bind('ended', function() {
	soundIcon.prop("src", "/static/images/speaker_teal.png");
});

$(qAudio).bind('ended', function() {
	timeUsedForAnswering = Date.now();
	topmessage.text("Vastausaikaa 8 s");
	topmessage.fadeIn();
	questionText.text(currentQuestion.question[0]);
	answerList.slideDown(400, function() {	
		main.css("min-height", main.css("height"));
	});
	var thisQ = currentQuestion; // Let the closures capture a local variable, not global
	window.setTimeout(function() { if (thisQ.answered) {return}; topmessage.text("Vastausaikaa 3 s"); }, 5000);
	window.setTimeout(function() { if (thisQ.answered) {return}; topmessage.text("Vastausaikaa 2 s"); }, 6000);
	window.setTimeout(function() { if (thisQ.answered) {return}; topmessage.text("Vastausaikaa 1 s"); }, 7000);
	window.setTimeout(function() {
		if (thisQ.answered) {return};
		topmessage.fadeOut(); 
		answerQuestion(-1, false, thisQ);
	}, 8000);
});

play_button.click(function() {
   	if (play_button.prop("disabled")) {
   		return;
   	};
   	questionStatus.slideUp();
	play_button.prop("disabled", true);
	qAudio.play();
	main.css("min-height", main.css("height"));
	avatar.fadeOut(400);
});

/* menu */

var settingsArea = $("#settings");
var menuButton = $("#menuButton");

function toggleMenu() {
	settingsArea.toggle();
}

settingsArea.hide();
settingsArea.click(toggleMenu);
menuButton.click(toggleMenu);
$("#settingsMenu").click(function( event ) { event.stopPropagation(); });

/* dynamics */

function nextQuestion() {
	semaphore--;
	if (semaphore > 0) { return; };
	showQuiz(currentQuestion);
};

function answerQuestion(ansId, isCorrect, question) {
	question.answered = true;
	var mark = null;
	var time = Date.now() - timeUsedForAnswering;
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
	semaphore = 2;

	$.post("/api/next_quiz", {
		answered_id: ansId,
		right_a_id: currentQuestion.right_a,
		question_id: currentQuestion.question_id,
		q_audio_id: currentQuestion.question[1],
		time: time,
		due_delay: currentQuestion.due_delay,
	}, function(result) {
		currentQuestion = result;
		nextQuestion();
	});
	mark.show();
	mark.removeClass("hidden");
	window.setTimeout(function() { mark.fadeOut(400); }, 1700);
	window.setTimeout(function() { answerList.slideUp(400, function() {
		topmessage.fadeOut();
		questionExplanation.text("Loading...");
		questionExplanation.slideDown();
		nextQuestion();
	}); }, 2200);
}

function spawnAnswerButton(ansId, text, ansAudioId, isCorrect, question) {
	var newAnswerButton = prototypeAnswer.clone();
	newAnswerButton.children("button")
		.text(text)
		.click(function(){
			$(this).addClass("buttonHilight");
			answerQuestion(ansId, isCorrect, question);
		});

	if (ansAudioId !== null) {
		var audio = document.createElement('audio');
		audio.setAttribute("preload", "auto");
		audio.setAttribute('src', "/api/get_line/"+ansAudioId);
		aAudio[ansId] = audio;
	}
	answerList.append(newAnswerButton);
};

function cleanState() {
	timesAudioPlayed = 0;
	wordSection.hide();
	questionSection.hide();
	aAudio = [];
	answerMarks.hide();
	answerMarks.addClass("hidden");
	currentQuestion = null;
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
		questionStatus.slideUp();
		showQuiz(question);
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
	questionStatus.slideDown();
}

function askQuestion(question) {
	questionSection.show();
	questionExplanation.text(question.explanation);
	avatar.fadeIn();
	questionExplanation.slideDown();
	play_button.prop("disabled", false);

	question.answers.forEach(function(a, i) {
		var isCorrect = (question.right_a === a[0])?true:false;
		spawnAnswerButton(a[0], a[1], a[2], isCorrect, question);
	});

	qAudio.setAttribute('src', "/api/get_line/"+question.question[1]);
}

function accentuate(word) {

	var empty = '<span class="accent">';
	var middle = '<span class="accent"><img src="/static/images/accent_middle.png">';
	var start = '<span class="accent"><img src="/static/images/accent_start.png">';
	var end = '<span class="accent"><img src="/static/images/accent_end.png" class="accent">';
	var flat_end = '<span class="accent"><img src="/static/images/accent_end_flat.png">';
	var start_end = '<span class="accent"><img src="/static/images/accent_start_end.png">';
	var start_end_flat = '<span class="accent"><img src="/static/images/accent_start_end_flat.png">';
	var start_end_flat_short = '<span class="accent"><img src="/static/images/accent_start_end_flat_short.png">';
	var peak = '<span class="accent"><img src="/static/images/accent_peak.png">';
	

	var accentuated = [];
	var ended = false;
	for (var i = 0, len = word.length; i < len; i++) {

		if (word.charAt(i) === "*") {
			continue;
		} else if (word.length === 1) {
			accentuated.push(start_end_flat_short);
		} else if (i === 0 && word.charAt(i+1) === "*") {
			accentuated.push(start_end);
			ended = true;
		} else if (i === 1 && !ended && word.charAt(i+1) === "*") {
			accentuated.push(peak);
			ended = true;
		} else if (i === 1 && !ended && i === len-1) {
			accentuated.push(start_end_flat);
		} else if (i === 1 && !ended && word.charAt(i+1) !== "*") {
			accentuated.push(start);
		} else if (i > 1 && !ended && i === len-1) {
			accentuated.push(flat_end);
		} else if (i > 1 && !ended && word.charAt(i+1) !== "*") {
			accentuated.push(middle);
		} else if (i > 1 && !ended && word.charAt(i+1) === "*") {
			accentuated.push(end);
			ended = true;
		} else {
			accentuated.push(empty);
		}
		accentuated.push(word.charAt(i));
		accentuated.push("</span>");
	}
	return accentuated.join("");
}

function showWord(word) {

	var word_html = word.word;
	if (word.show_accents) {
		word_html = accentuate(word.word);
	} 
	wordItself.html(word_html);
	wordExplanation.html(word.explanation);
	wordAudio.setAttribute('src', "/api/get_line/"+word.audio_bundle);
	wordAudio.play();
	timesAudioPlayed++;
	soundIcon.prop("src", "/static/images/speaker_pink.png");
	wordSection.show();
}

function showQuiz(question) {

	cleanState();

	if (question === null) {
		questionStatus.text("Ei ole mitään kysyttävää ☹️");
		questionStatus.slideDown();
		play_button.prop("disabled", true);
		avatar.fadeOut(100);
		return;
	} else if (new Date(question.due_date) > new Date()) {
		play_button.prop("disabled", true);
		avatar.fadeOut(100);
		breakTime(question);
		breakTimeWaitHandle = window.setInterval(function() { breakTime(question); }, 1000);
		return;
	}
	currentQuestion = question;
	question.answered = false;

	if (question.quiz_type === "question") {
		askQuestion(question);
	} else if (question.quiz_type === "word") {
		showWord(question);
	} else {
		questionStatus.text("Oops, there seems to be a bug :(");
		questionStatus.show();
	}

}

$.getJSON("/api/new_quiz", showQuiz);

});
