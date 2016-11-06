/// <reference path="typings/globals/jquery/index.d.ts" />
$(function () {
    var prototypeAnswer = $(".answer").remove();
    var avatar = $("#quiz .avatar");
    var main = $("#main");
    var answerList = $(".answerList");
    var questionText = $(".questionText");
    var correct = document.getElementById('sfxCorrect');
    var wrong = document.getElementById('sfxWrong');
    function spawnAnswerButton(text, path, isCorrect) {
        var newAnswerButton = prototypeAnswer.clone();
        newAnswerButton.children("button")
            .text(text)
            .click(function () {
            if (isCorrect) {
                correct.play();
                alert("CORRECT");
            }
            else {
                wrong.play();
                alert("WRONG");
            }
        });
        answerList.append(newAnswerButton);
    }
    ;
    $.getJSON("/api/new_quiz", function (result) {
        var play_button = $("#quiz .avatar .imgbutton");
        var explanation = $("#quiz .explanation");
        if (result === null) {
            explanation.text("Ei ole mitään kysyttävää ☹️");
            play_button.off("click");
            play_button.prop("disabled", true);
            main.css("min-height", main.css("height"));
            avatar.fadeOut(100);
            return;
        }
        explanation.text(result.explanation);
        var qAudio = document.createElement('audio');
        var aAudio = [];
        $(qAudio).bind('ended', function () {
            answerList.hide();
            questionText.text(result.question[0]);
            result.answers.forEach(function (a, i) {
                var isCorrect = (result.right_a === a[0]) ? true : false;
                spawnAnswerButton(a[1], a[2], isCorrect);
            });
            answerList.slideDown();
        });
        play_button.click(function () {
            qAudio.play();
            play_button.off("click");
            play_button.prop("disabled", true);
            main.css("min-height", main.css("height"));
            avatar.fadeOut(400);
        });
        qAudio.setAttribute('src', result.question[1]);
        aAudio.forEach(function (a, i) {
            var audio = document.createElement('audio');
            audio.setAttribute('src', result.answers[result.right_a][1]);
            aAudio[a[0]] = audio;
        });
    });
});
