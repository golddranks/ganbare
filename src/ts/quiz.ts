/// <reference path="typings/globals/jquery/index.d.ts" />

$(function() {
$.getJSON("/api/new_quiz", function(result) {
		var play_button = $("#quiz .avatar .imgbutton");
		var qAudio = document.createElement('audio');
		var aAudio = document.createElement('audio');
		var test1Audio = document.createElement('audio');
		var test2Audio = document.createElement('audio');
    	play_button.click(function() {
			qAudio.play();
			play_button.off("click");
			play_button.prop("disabled", true);
			$("#quiz .avatar").fadeOut(400);
		});
		qAudio.setAttribute('src', result.q_line);
		aAudio.setAttribute('src', result.a_line);
		test1Audio.setAttribute('src', result.a_line);
		test2Audio.setAttribute('src', result.a_line);
});
});
