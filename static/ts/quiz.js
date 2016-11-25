/// <reference path="typings/globals/jquery/index.d.ts" />
/// <reference path="typings/globals/howler/index.d.ts" />
$(function () {
    function accentuate(word) {
        var empty = '<span class="accent">';
        var middle = '<span class="accent"><img src="/static/images/accent_middle.png" style="display:none;">';
        var start = '<span class="accent"><img src="/static/images/accent_start.png" style="display:none;">';
        var end = '<span class="accent"><img src="/static/images/accent_end.png" class="accent" style="display:none;">';
        var flat_end = '<span class="accent"><img src="/static/images/accent_end_flat.png" style="display:none;">';
        var start_end = '<span class="accent"><img src="/static/images/accent_start_end.png" style="display:none;">';
        var start_end_flat = '<span class="accent"><img src="/static/images/accent_start_end_flat.png" style="display:none;">';
        var start_end_flat_short = '<span class="accent"><img src="/static/images/accent_start_end_flat_short.png" style="display:none;">';
        var peak = '<span class="accent"><img src="/static/images/accent_peak.png" style="display:none;">';
        function isAccentMark(i) {
            return (word.charAt(i) === "*" || word.charAt(i) === "・");
        }
        ;
        var accentuated = [""];
        var ended = false;
        for (var i = 0, len = word.length; i < len; i++) {
            if (isAccentMark(i)) {
                continue;
            }
            else if (word.length === 1) {
                accentuated.push(start_end_flat_short);
            }
            else if (i === 0 && isAccentMark(i + 1)) {
                accentuated.push(start_end);
                ended = true;
            }
            else if (i === 1 && !ended && isAccentMark(i + 1)) {
                accentuated.push(peak);
                ended = true;
            }
            else if (i === 1 && !ended && i === len - 1) {
                accentuated.push(start_end_flat);
            }
            else if (i === 1 && !ended) {
                accentuated.push(start);
            }
            else if (i > 1 && !ended && i === len - 1) {
                accentuated.push(flat_end);
            }
            else if (i > 1 && !ended && isAccentMark(i + 1)) {
                accentuated.push(end);
                ended = true;
            }
            else if (i > 1 && !ended && !isAccentMark(i + 1)) {
                accentuated.push(middle);
            }
            else {
                accentuated.push(empty);
            }
            accentuated.push(word.charAt(i));
            accentuated.push("</span>");
        }
        return accentuated.join("");
    }
    /* general things */
    var main = $("#main");
    var errorSection = $("#errorSection");
    var errorStatus = $("#errorStatus");
    var semaphore = 0;
    var breakTimeWaitHandle = null;
    var currentQuestion = null;
    var activeAnswerTime = null;
    var fullAnswerTime = null;
    var timesAudioPlayed = 0;
    var correct = new Howl({ src: ['/static/sfx/correct.m4a', '/static/sfx/correct.mp3'] });
    var wrong = new Howl({ src: ['/static/sfx/wrong.m4a', '/static/sfx/wrong.mp3'] });
    var bell = new Howl({ src: ['/static/sfx/bell.m4a', '/static/sfx/bell.mp3'] });
    var speakerIconTeal = $("#speakerIconTeal");
    var speakerIconPink = $("#speakerIconPink");
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
    var topmessage = $(".topmessageparagraph");
    /* word- and exercise-related things */
    var wordShowButton = $("#wordShowButton");
    var wordShowKana = $("#wordShowKana");
    var wordStatus = $("#wordStatus");
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
        setTimeout(function () { errorStatus.html("Server is down or there is a bug :(<br>Trying to connect again..."); }, 2000);
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
    $("#settingsMenu").click(function (event) { event.stopPropagation(); });
    /* app main logic */
    function cleanState() {
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
        var dur_seconds = (new Date(question.due_date).getTime() - Date.now()) / 1000;
        var dur_hours = Math.floor(dur_seconds / 3600);
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
                + dur_hours + " tunnin ja " + dur_minutes_remainder + " minuutin päästä");
        }
        else if (dur_hours === 0 && dur_minutes_remainder > 4) {
            questionStatus.html("Tauon paikka!<br>Seuraava kysymys avautuu<br>"
                + dur_minutes_remainder + " minuutin päästä");
        }
        else if (dur_hours === 0 && dur_minutes_remainder > 0) {
            questionStatus.html("Tauon paikka!<br>Seuraava kysymys avautuu<br>"
                + dur_minutes_remainder + " minuutin ja " + dur_seconds_remainder + " sekunnin päästä");
        }
        else if (dur_hours === 0 && dur_minutes_remainder === 0 && dur_seconds_remainder > 0) {
            questionStatus.html("Tauon paikka!<br>Seuraava kysymys avautuu<br>"
                + dur_seconds_remainder + " sekunnin päästä");
        }
        questionSection.show();
        questionStatus.slideDown();
    }
    function nextQuestion() {
        semaphore--;
        if (semaphore > 0) {
            return;
        }
        ;
        showQuiz(currentQuestion);
    }
    ;
    function setLoadError(audioElement, elementName, closureQuestion) {
        audioElement.on("loaderror", function (id, e) {
            if (closureQuestion !== null && currentQuestion !== closureQuestion) {
                this.off();
                return false;
            }
            ;
            console.log("Error with " + elementName + " element! Trying again after 3 secs.");
            bugMessage(e);
            console.log("element", audioElement);
            audioElement.off("load").once("load", function () {
                console.log(audioElement);
                clearError();
            });
            setTimeout(function () {
                audioElement.unload();
                audioElement.load();
            }, 3000);
        });
    }
    ;
    setLoadError(correct, "correctSfx", null);
    setLoadError(wrong, "wrongSfx", null);
    function setWordShowButton(audio) {
        wordShowButton.off('click').on('click', function () {
            timesAudioPlayed++;
            audio.play();
            speakerIconTeal.hide();
            speakerIconPink.show();
        });
        audio.on('end', function () {
            speakerIconTeal.show();
            speakerIconPink.hide();
        });
    }
    function answerExercise(isCorrect, exercise) {
        wordShowButton.off('click');
        exerciseFailureButton.off('click');
        exerciseSuccessButton.off('click');
        buttonSection.slideUp(100);
        setTimeout(function () {
            wordSection.slideUp(400, function () {
                nextQuestion();
            });
        }, 600);
        if (isCorrect) {
            correct.play();
        }
        else {
            bell.play();
        }
        semaphore = 2;
        function postAnswerExercise() {
            var jqxhr = $.post("/api/next_quiz", {
                type: "exercise",
                word_id: exercise.id,
                correct: isCorrect,
                times_audio_played: timesAudioPlayed,
                active_answer_time: activeAnswerTime - fullAnswerTime,
                full_answer_time: Date.now() - fullAnswerTime
            }, function (result) {
                clearError();
                console.log("postAnswerExercise: got result");
                currentQuestion = result;
                nextQuestion();
            });
            jqxhr.fail(function (e) {
                bugMessage(e);
                setTimeout(postAnswerExercise, 3000);
            });
        }
        ;
        postAnswerExercise();
    }
    ;
    function answerWord() {
        wordShowButton.off('click');
        setTimeout(function () {
            wordSection.slideUp(400, function () {
                nextQuestion();
            });
        }, 500);
        semaphore = 2;
        function postAnswerWord() {
            var jqxhr = $.post("/api/next_quiz", {
                type: "word",
                word_id: currentQuestion.id,
                times_audio_played: timesAudioPlayed,
                time: Date.now() - activeAnswerTime
            }, function (result) {
                clearError();
                console.log("postAnswerWord: got result");
                currentQuestion = result;
                nextQuestion();
            });
            jqxhr.fail(function (e) {
                bugMessage(e);
                setTimeout(postAnswerWord, 3000);
            });
        }
        ;
        postAnswerWord();
    }
    ;
    function answerQuestion(ansId, isCorrect, question, button) {
        if (question.answered) {
            return;
        }
        ;
        question.answered = true;
        $(this).addClass("buttonHilight");
        var mark = null;
        var activeATime = Date.now() - activeAnswerTime;
        var fullATime = Date.now() - fullAnswerTime;
        if (isCorrect) {
            mark = maru;
            questionStatus.text("Oikein! Seuraava kysymys.");
            correct.play();
        }
        else if (ansId > 0) {
            mark = batsu;
            questionStatus.text("Pieleen meni, kokeile uudestaan!");
            wrong.play();
        }
        else if (ansId === -1) {
            mark = batsu;
            questionStatus.text("Aika loppui!");
            wrong.play();
        }
        questionStatus.show();
        questionExplanation.hide();
        semaphore = 2;
        if (button === null) {
            mark.css("top", "55%;");
        }
        else {
            var top = $(button).position().top + ($(button).height() / 2);
            mark.css("top", top + "px");
        }
        mark.show();
        mark.removeClass("hidden");
        setTimeout(function () { mark.fadeOut(400); }, 1700);
        setTimeout(function () {
            answerList.slideUp(400, function () {
                topmessage.fadeOut();
                questionExplanation.text("Loading...");
                questionExplanation.slideDown();
                nextQuestion();
            });
        }, 2200);
        function postAnswerQuestion() {
            var jqxhr = $.post("/api/next_quiz", {
                type: "question",
                answered_id: ansId,
                right_a_id: question.right_a,
                question_id: question.question_id,
                q_audio_id: question.question[1],
                active_answer_time: activeATime,
                full_answer_time: fullATime
            }, function (result) {
                clearError();
                console.log("postAnswerQuestion: got result");
                currentQuestion = result;
                nextQuestion();
            });
            jqxhr.fail(function (e) {
                bugMessage(e);
                setTimeout(postAnswerQuestion, 3000);
            });
        }
        ;
        postAnswerQuestion();
    }
    function spawnAnswerButton(ansId, text, ansAudioId, isCorrect, question) {
        var newAnswerButton = prototypeAnswer.clone();
        var aAudio = null;
        if (ansAudioId !== null) {
            aAudio = new Howl({ src: ['/api/audio/' + ansAudioId + '.mp3'] });
            setLoadError(aAudio, "answerAudio", question);
        }
        newAnswerButton.children("button")
            .html(text)
            .one('click', function () {
            if (aAudio !== null) {
                aAudio.play();
            }
            ;
            answerQuestion(ansId, isCorrect, question, this);
        });
        answerList.append(newAnswerButton);
    }
    ;
    function showQuestion(question) {
        questionSection.show();
        questionExplanation.text(question.explanation);
        avatar.show();
        avatar.css('opacity', '0');
        questionExplanation.slideDown(400, function () { avatar.fadeTo(400, 1); });
        fullAnswerTime = Date.now();
        question.answers.forEach(function (a, i) {
            var isCorrect = (question.right_a === a[0]) ? true : false;
            spawnAnswerButton(a[0], a[1], a[2], isCorrect, question);
        });
        var qAudio = new Howl({ src: ['/api/audio/' + question.question[1] + '.mp3'] });
        play_button.one('click', function () {
            questionStatus.slideUp();
            qAudio.play();
            console.log(qAudio);
            main.css("min-height", main.css("height"));
            avatar.fadeOut(400);
        });
        qAudio.on('play', function () { console.log("Playing!"); });
        qAudio.once('end', function () {
            activeAnswerTime = Date.now();
            topmessage.text("Vastausaikaa 8 s");
            topmessage.fadeIn();
            questionText.text(currentQuestion.question[0]);
            answerList.slideDown(400);
            window.setTimeout(function () { if (question.answered) {
                return;
            } ; topmessage.text("Vastausaikaa 3 s"); }, 5000);
            window.setTimeout(function () { if (question.answered) {
                return;
            } ; topmessage.text("Vastausaikaa 2 s"); }, 6000);
            window.setTimeout(function () { if (question.answered) {
                return;
            } ; topmessage.text("Vastausaikaa 1 s"); }, 7000);
            window.setTimeout(function () {
                if (question.answered) {
                    return;
                }
                ;
                topmessage.fadeOut();
                answerQuestion(-1, false, question, null);
            }, 8000);
        });
        setLoadError(qAudio, "questionAudio", question);
    }
    function showWord(word) {
        console.log("showWord!");
        wordOkButton.show();
        wordOkButton.one('click', answerWord);
        buttonSection.show();
        wordShowKana.html(accentuate(word.word));
        if (word.show_accents) {
            $(".accent img").show();
        }
        wordExplanation.html(word.explanation);
        var wordAudio = new Howl({ src: ['/api/audio/' + word.audio_id + '.mp3'] });
        setLoadError(wordAudio, "wordAudio", word);
        wordShowButton.show();
        setTimeout(function () { setWordShowButton(wordAudio); wordShowButton.trigger('click'); }, 1500);
        timesAudioPlayed++;
        activeAnswerTime = Date.now();
        setTimeout(function () { wordSection.slideDown(); }, 100);
    }
    function showExercise(exercise) {
        console.log("showExercise!");
        exerciseOkButton.show();
        wordStatus.text("Äännä parhaasi mukaan:").show();
        buttonSection.show();
        wordShowKana.html(accentuate(exercise.word));
        $(".accent img").hide();
        wordExplanation.html(exercise.explanation);
        var exerciseAudio = new Howl({ src: ['/api/audio/' + exercise.audio_id + '.mp3'] });
        setLoadError(exerciseAudio, "exerciseAudio", exercise);
        setWordShowButton(exerciseAudio);
        exerciseSuccessButton.one('click', function () { answerExercise(true, exercise); });
        exerciseFailureButton.one('click', function () { answerExercise(false, exercise); });
        exerciseAudio.once('end', function () {
            wordButtonLabel.text("Itsearvio");
            wordButtonLabel.show();
            exerciseFailureButton.show();
            exerciseSuccessButton.show();
            setTimeout(function () {
                wordShowButton.fadeIn();
                buttonSection.slideDown();
            }, 1100);
        });
        exerciseOkButton.one("click", function () {
            $(".accent img").fadeIn();
            exerciseAudio.play();
            timesAudioPlayed++;
            activeAnswerTime = Date.now();
            buttonSection.slideUp(200, function () {
                exerciseOkButton.hide();
            });
            wordStatus.slideUp();
        });
        fullAnswerTime = Date.now();
        setTimeout(function () { wordSection.slideDown(); }, 100);
    }
    function showQuiz(question) {
        console.log("showQuiz!");
        cleanState();
        if (question === null) {
            console.log("No cards!");
            questionSection.show();
            questionStatus.text("Ei ole mitään kysyttävää ☹️");
            questionStatus.slideDown();
            avatar.fadeOut(100);
            return;
        }
        else if (new Date(question.due_date) > new Date()) {
            console.log("BreakTime!");
            avatar.fadeOut(100);
            breakTime(question);
            breakTimeWaitHandle = window.setInterval(function () { breakTime(question); }, 1000);
            return;
        }
        currentQuestion = question;
        question.answered = false;
        if (question.quiz_type === "question") {
            showQuestion(question);
        }
        else if (question.quiz_type === "word") {
            showWord(question);
        }
        else if (question.quiz_type === "exercise") {
            showExercise(question);
        }
        else {
            bugMessage(question);
        }
    }
    function start() {
        clearError();
        var jqxhr = $.getJSON("/api/new_quiz", showQuiz);
        jqxhr.fail(function (e) {
            console.log("Connection fails with getJSON. (/api/new_quiz)");
            bugMessage(e);
            setTimeout(start, 3000);
        });
    }
    ;
    start();
});
//# sourceMappingURL=quiz.js.map