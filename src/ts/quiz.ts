/// <reference path="typings/globals/jquery/index.d.ts" />

$(function() {

var prototypeAnswer = $(".answer").remove();

function spawnAnswerButton(text, path) {
	var newAnswerButton = prototypeAnswer.clone();
	newAnswerButton.children("button").text(text);
	$(".answerList").append(newAnswerButton);
};

spawnAnswerButton("a", "b");
spawnAnswerButton("aa", "bb");
spawnAnswerButton("aaa", "bbb");
spawnAnswerButton("aaaa", "bbbb");

$.getJSON("/api/new_quiz", function(result) {
		var play_button = $("#quiz .avatar .imgbutton");
		var explanation = $("#quiz .explanation");

		explanation.text(result.explanation);

		var qAudio = document.createElement('audio');
		var aAudio = document.createElement('audio');

		$(qAudio).bind('ended', function() {
			alert("kysymys!");
		});

    	play_button.click(function() {
			qAudio.play();
			play_button.off("click");
			play_button.prop("disabled", true);
			$("#quiz .avatar").fadeOut(400);
		});

		qAudio.setAttribute('src', result.question[1]);
		aAudio.setAttribute('src', result.right_answer[1]);
});

});
