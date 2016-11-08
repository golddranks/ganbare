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
var play_button = $("#quiz .avatar .imgbutton");
var maru = $("#maru");
var batsu = $("#batsu");
var answerMarks = $(".answerMark");
var semaphore = 0;

var qAudio = <HTMLAudioElement>document.getElementById('questionAudio');
var correct = <HTMLAudioElement>document.getElementById('sfxCorrect');
var wrong = <HTMLAudioElement>document.getElementById('sfxWrong');

var currentQuestion = null;
var aAudio = [];
var timeAudioEnded = null;

$(qAudio).bind('ended', function() {
	timeAudioEnded = Date.now();
	questionText.text(currentQuestion.question[0]);
	answerList.slideDown();
});

play_button.click(function() {
   	if (play_button.prop("disabled")) {
   		return;
   	};
	play_button.prop("disabled", true);
	qAudio.play();
	main.css("min-height", main.css( "height" ));
	avatar.fadeOut(400);
});

/* dynamics */

function nextQuestion() {
	semaphore--;
	if (semaphore > 0) { return; };
	askQuestion(currentQuestion);
};

function spawnAnswerButton(ansId, text, path, isCorrect) {
	var newAnswerButton = prototypeAnswer.clone();
	newAnswerButton.children("button")
		.text(text)
		.click(function(){
			$(this).addClass("buttonHilight");
			var mark = null;
			var time = Date.now() - timeAudioEnded;
			if (isCorrect) {
				mark = maru;
				explanation.text("Oikein! Seuraava kysymys.");
				correct.play();
			} else {
				mark = batsu;
				explanation.text("Pieleen meni, kokeile uudestaan!");
				wrong.play();
			}
			semaphore = 2;

			$.post("/api/next_quiz", {
				answer_id: ansId,
				right_a_id: currentQuestion.right_a,
				question_id: currentQuestion.question_id,
				time: time,
			}, function(result) {
				currentQuestion = result;
				nextQuestion();
			});
			mark.show();
			mark.removeClass("hidden");
			setTimeout(function() { mark.fadeOut(400); }, 1700);
			setTimeout(function() { answerList.slideUp(400); }, 2200);
			setTimeout(function() { explanation.text("Loading..."); nextQuestion(); }, 4000);
		});

	if (path !== null) {
		var audio = document.createElement('audio');
		audio.setAttribute("preload", "auto");
		audio.setAttribute('src', path);
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
	answerList.children(".answer")
		.remove();
	avatar.fadeIn();
	play_button.prop("disabled", false);
	answerList.hide();
}


function askQuestion(question) {

	cleanState();
	currentQuestion = question;

	explanation.text(question.explanation);

	question.answers.forEach(function(a, i) {
		var isCorrect = (question.right_a === a[0])?true:false;
		spawnAnswerButton(a[0], a[1], a[2], isCorrect);
	});

	qAudio.setAttribute('src', question.question[1]);
}

$.getJSON("/api/new_quiz", function(result) {

	if (result === null) {
		explanation.text("Ei ole mitään kysyttävää ☹️");
		play_button.off("click");
		play_button.prop("disabled", true);
		main.css("min-height", main.css( "height" ));
		avatar.fadeOut(100);
		return;
	}

	askQuestion(result);
});

});
