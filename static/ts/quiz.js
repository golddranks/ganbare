/// <reference path="typings/globals/jquery/index.d.ts" />
$(function () {
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
    var qAudio = document.getElementById('questionAudio');
    var correct = document.getElementById('sfxCorrect');
    var wrong = document.getElementById('sfxWrong');
    var currentQuestion = null;
    var aAudio = [];
    var timeAudioEnded = null;
    $(qAudio).bind('ended', function () {
        timeAudioEnded = Date.now();
        questionText.text(currentQuestion.question[0]);
        answerList.slideDown();
    });
    play_button.click(function () {
        if (play_button.prop("disabled")) {
            return;
        }
        ;
        play_button.prop("disabled", true);
        qAudio.play();
        main.css("min-height", main.css("height"));
        avatar.fadeOut(400);
    });
    /* dynamics */
    function nextQuestion() {
        semaphore--;
        if (semaphore > 0) {
            return;
        }
        ;
        askQuestion(currentQuestion);
    }
    ;
    function spawnAnswerButton(ansId, text, ansAudioId, isCorrect, question) {
        var newAnswerButton = prototypeAnswer.clone();
        newAnswerButton.children("button")
            .text(text)
            .click(function () {
            $(this).addClass("buttonHilight");
            var mark = null;
            var time = Date.now() - timeAudioEnded;
            if (isCorrect) {
                mark = maru;
                explanation.text("Oikein! Seuraava kysymys.");
                correct.play();
            }
            else {
                mark = batsu;
                explanation.text("Pieleen meni, kokeile uudestaan!");
                wrong.play();
            }
            semaphore = 2;
            $.post("/api/next_quiz", {
                answer_id: ansId,
                right_a_id: currentQuestion.right_a,
                question_id: currentQuestion.question_id,
                q_audio_id: currentQuestion.question[1],
                time: time,
                due_delay: question.due_delay
            }, function (result) {
                currentQuestion = result;
                nextQuestion();
            });
            mark.show();
            mark.removeClass("hidden");
            setTimeout(function () { mark.fadeOut(400); }, 1700);
            setTimeout(function () { answerList.slideUp(400); }, 2200);
            setTimeout(function () { explanation.text("Loading..."); nextQuestion(); }, 4000);
        });
        if (ansAudioId !== null) {
            var audio = document.createElement('audio');
            audio.setAttribute("preload", "auto");
            audio.setAttribute('src', "/api/get_line/" + ansAudioId);
            aAudio[ansId] = audio;
        }
        answerList.append(newAnswerButton);
    }
    ;
    function cleanState() {
        aAudio = [];
        answerMarks.hide();
        answerMarks.addClass("hidden");
        currentQuestion = null;
        explanation.text("");
        answerList.children(".answer")
            .remove();
        answerList.hide();
    }
    function askQuestion(question) {
        cleanState();
        if (question === null) {
            explanation.text("Ei ole mitään kysyttävää ☹️");
            play_button.prop("disabled", true);
            avatar.fadeOut(100);
            return;
        }
        else if (new Date(question.due_date) > new Date()) {
            var dur_minutes = (new Date(question.due_date).getTime() - Date.now()) / 1000 / 60;
            var dur_hours = Math.floor(dur_minutes / 60);
            var dur_minutes_remainder = Math.floor(dur_minutes % 60);
            explanation.html("Tauon paikka!<br>Seuraava kysymys avautuu<br>"
                + dur_hours + " tunnin ja " + dur_minutes_remainder + " minuutin päästä");
            play_button.prop("disabled", true);
            avatar.fadeOut(100);
            return;
        }
        else {
            avatar.fadeIn();
            play_button.prop("disabled", false);
        }
        currentQuestion = question;
        explanation.text(question.explanation);
        question.answers.forEach(function (a, i) {
            var isCorrect = (question.right_a === a[0]) ? true : false;
            spawnAnswerButton(a[0], a[1], a[2], isCorrect, question);
        });
        qAudio.setAttribute('src', "/api/get_line/" + question.question[1]);
    }
    $.getJSON("/api/new_quiz", askQuestion);
});
