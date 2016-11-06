/// <reference path="typings/globals/jquery/index.d.ts" />
$(function () {
    var prototypeAnswer = $(".answer").remove();
    var avatar = $("#quiz .avatar");
    var main = $("#main");
    var answerList = $(".answerList");
    var questionText = $(".questionText");
    var explanation = $("#quiz .explanation");
    var play_button = $("#quiz .avatar .imgbutton");
    var correct = document.getElementById('sfxCorrect');
    var wrong = document.getElementById('sfxWrong');
    var qAudio = null;
    var aAudio = [];
    var maru = document.createElement("img");
    maru.setAttribute("src", "/static/images/maru_red.png");
    $(maru).addClass("answerMark");
    $(maru).hide();
    $(maru).addClass("hidden");
    $(maru).appendTo(answerList);
    var batsu = document.createElement("img");
    batsu.setAttribute("src", "/static/images/batsu.png");
    $(batsu).addClass("answerMark");
    $(batsu).hide();
    $(batsu).addClass("hidden");
    $(batsu).appendTo(answerList);
    function ask_question() {
        answerList.hide();
        play_button.prop("disabled", false);
        avatar.show();
        play_button.click(function () {
            qAudio.play();
            play_button.off("click");
            play_button.prop("disabled", true);
            main.css("min-height", main.css("height"));
            avatar.fadeOut(400);
        });
    }
    function spawnAnswerButton(text, path, isCorrect) {
        var newAnswerButton = prototypeAnswer.clone();
        newAnswerButton.children("button")
            .text(text)
            .click(function (ev) {
            $(this).addClass("buttonHilight");
            var mark = null;
            if (isCorrect) {
                mark = maru;
                explanation.text("Oikein!");
                correct.play();
            }
            else {
                mark = batsu;
                explanation.text("Pieleen meni, kokeile uudestaan!");
                wrong.play();
            }
            $(mark).show();
            $(mark).removeClass("hidden");
            setTimeout(function () {
                $(mark).fadeOut();
                ask_question();
            }, 2000);
        });
        answerList.append(newAnswerButton);
    }
    ;
    $.getJSON("/api/new_quiz", function (result) {
        if (result === null) {
            explanation.text("Ei ole mitään kysyttävää ☹️");
            play_button.off("click");
            play_button.prop("disabled", true);
            main.css("min-height", main.css("height"));
            avatar.fadeOut(100);
            return;
        }
        explanation.text(result.explanation);
        qAudio = document.createElement('audio');
        qAudio.setAttribute("preload", "auto");
        $(qAudio).bind('ended', function () {
            questionText.text(result.question[0]);
            result.answers.forEach(function (a, i) {
                var isCorrect = (result.right_a === a[0]) ? true : false;
                spawnAnswerButton(a[1], a[2], isCorrect);
            });
            answerList.slideDown();
        });
        ask_question();
        qAudio.setAttribute('src', result.question[1]);
        aAudio.forEach(function (a, i) {
            var audio = document.createElement('audio');
            audio.setAttribute("preload", "auto");
            audio.setAttribute('src', result.answers[result.right_a][1]);
            aAudio[a[0]] = audio;
        });
    });
});
