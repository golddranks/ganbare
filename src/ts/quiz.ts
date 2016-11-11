/// <reference path="typings/globals/jquery/index.d.ts" />

$(function() {

/* init the static machinery */

var prototypeAnswer = $(".answer").remove();
prototypeAnswer.show();
var avatar = $("#quiz .avatar");
var main = $("#main");
var answerList = $(".answerList");
var questionText = $(".questionText");
var explanation = $("#quiz .explanation");
var status = $("#quiz .status");
var play_button = $("#quiz .avatar .imgbutton");
var maru = $("#maru");
var batsu = $("#batsu");
var answerMarks = $(".answerMark");
var semaphore = 0;
var topmessage = $(".topmessageparagraph");
var breakTimeWaitHandle = null;

var qAudio = <HTMLAudioElement>document.getElementById('questionAudio');
var correct = <HTMLAudioElement>document.getElementById('sfxCorrect');
var wrong = <HTMLAudioElement>document.getElementById('sfxWrong');

var currentQuestion = null;
var aAudio = [];
var timeAudioEnded = null;

$(qAudio).bind('ended', function() {
	timeAudioEnded = Date.now();
	topmessage.text("Vastausaikaa 8 s");
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
		topmessage.text(""); 
		answerQuestion(-1, false, thisQ);
	}, 8000);
});

play_button.click(function() {
   	if (play_button.prop("disabled")) {
   		return;
   	};
   	status.slideUp();
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

/* dynamics */

function nextQuestion() {
	semaphore--;
	if (semaphore > 0) { return; };
	askQuestion(currentQuestion);
};

function answerQuestion(ansId, isCorrect, question) {
	question.answered = true;
	var mark = null;
	var time = Date.now() - timeAudioEnded;
	if (isCorrect) {
		mark = maru;
		status.text("Oikein! Seuraava kysymys.");
		correct.play();
	} else if (ansId > 0) {
		mark = batsu;
		status.text("Pieleen meni, kokeile uudestaan!");
		wrong.play();
	} else if (ansId === -1) {
		mark = batsu;
		status.text("Aika loppui!");
		wrong.play();
	}
	status.show();
	explanation.hide();
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
	window.setTimeout(function() { answerList.slideUp(400, function() { explanation.text("Loading..."); nextQuestion(); }); }, 2200);
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
	aAudio = [];
	answerMarks.hide();
	answerMarks.addClass("hidden");
	currentQuestion = null;
	explanation.text("");
	explanation.show();
	topmessage.text("");
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
		askQuestion(question);
		return;
	}

	if (dur_hours > 0) {
		explanation.html("Tauon paikka!<br>Seuraava kysymys avautuu<br>"
			+ dur_hours +" tunnin ja "+dur_minutes_remainder+" minuutin päästä");
	} else if (dur_hours === 0 && dur_minutes_remainder > 4) {
		explanation.html("Tauon paikka!<br>Seuraava kysymys avautuu<br>"
			+ dur_minutes_remainder+" minuutin päästä");
	} else if (dur_hours === 0 && dur_minutes_remainder > 0) {
		explanation.html("Tauon paikka!<br>Seuraava kysymys avautuu<br>"
			+ dur_minutes_remainder+" minuutin ja "+ dur_seconds_remainder +" sekunnin päästä");
	} else if (dur_hours === 0 && dur_minutes_remainder === 0 && dur_seconds_remainder > 0) {
		explanation.html("Tauon paikka!<br>Seuraava kysymys avautuu<br>"
			+ dur_seconds_remainder +" sekunnin päästä");
	}
}


function askQuestion(question) {

	cleanState();

	if (question === null) {
		explanation.text("Ei ole mitään kysyttävää ☹️");
		play_button.prop("disabled", true);
		avatar.fadeOut(100);
		return;
	} else if (new Date(question.due_date) > new Date()) {
		play_button.prop("disabled", true);
		avatar.fadeOut(100);
		breakTime(question);
		breakTimeWaitHandle = window.setInterval(function() { breakTime(question); }, 1000);
		return;
	} else {
		avatar.fadeIn();
		explanation.slideDown();
		play_button.prop("disabled", false);
	}

	currentQuestion = question;
	question.answered = false;

	explanation.text(question.explanation);

	question.answers.forEach(function(a, i) {
		var isCorrect = (question.right_a === a[0])?true:false;
		spawnAnswerButton(a[0], a[1], a[2], isCorrect, question);
	});

	qAudio.setAttribute('src', "/api/get_line/"+question.question[1]);
}

$.getJSON("/api/new_quiz", askQuestion);

});
