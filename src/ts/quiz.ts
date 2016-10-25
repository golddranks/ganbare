/// <reference path="typings/globals/jquery/index.d.ts" />

$(function() {
$.getJSON("/api/new_quiz", function(result) {
		var play_button = $("#quiz .avatar .imgbutton");
		var audioElement = document.createElement('audio');
    	play_button.click(function() {
			audioElement.play();
			play_button.off("click");
			play_button.prop("disabled", true);
			$("#quiz .avatar").fadeOut(400);
		});
		audioElement.setAttribute('src', result.lines);
});
});
