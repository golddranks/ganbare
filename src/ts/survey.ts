/// <reference path="typings/globals/jquery/index.d.ts" />

$(function() {


/* menu */

var main = $("#main");
var settingsArea = $("#settings");
var menuButton = $("#menuButton");
var backButton = $("#backButton");

function toggleMenu(event: Event) : void {
	settingsArea.toggle();
	main.toggleClass("menuOn");
	event.stopPropagation(); 
}

function cancelMenu(event: Event): void {
	settingsArea.hide();
	main.removeClass("menuOn");
	event.stopPropagation(); 
}

settingsArea.hide();
settingsArea.click(cancelMenu);
$("body").click(cancelMenu);
menuButton.click(toggleMenu);
$("#settingsMenu").click(function( event ) { event.stopPropagation(); });

/* menu ends */

function surveyReady() {
	$("#questionText").text("Kiitos vastauksista!");
	$('<form action="/ok" method="POST">\
		<input type="hidden" value="survey" name="event_ok">\
		<input type="submit" value="Ok" style="min-width: 50%; margin-top: 1em;">\
	</form>').appendTo("#answerButtons");
};

function incrementFactory() {

	var answers_0 = [
		"Aloita kysely!",
	];

	var answers_1 = [
		'en ollenkaan tai satunnaisesti <br>(enintään joitakin kertoja vuodessa)',
		'silloin tällöin, mutta <br class="smallscreen">joskus on <br class="bigscreen">viikkojen <br class="smallscreen">tai kuukausien taukoja',
		'viikoittain tai ainakin<br>monta kertaa kuussa',
		'monta kertaa viikossa',
		'päivittäin',
	];

	var answers_2 = [
		"en ole käynyt",
		"olen käynyt kerran",
		"olen käynyt joitakin kertoja",
		"käyn vähintään kerran muutamassa vuodessa",
		"käyn kerran vuodessa tai useammin",
	];

	var answers_3 = [
		"käyn monta tuntia viikossa",
		"käyn kerran viikossa",
		"käyn satunnaisesti/<wbr>lyhytkestoisesti",
		"en tällä hetkellä",
	];

	var answers_4 = [
		"en kiinnitä huomiota",
		"välillä yritän kuunnella tarkkaavaisesti",
		"kuuntelen usein tarkkaavaisesti",
		"kiinnitän huomioni jatkuvasti ääntämiseen",
	];

	var answers_5 = [
		"asialla ei ole minulle suurta väliä",
		"olisi ihan kiva, mutta se ei ole prioriteettini",
		"haluan ääntää ainakin kohtalaisen hyvin",
		"hyvä ääntämys on minulle tärkeä asia",
		"haluaisin kuulostaa japanilaiselta",
	];

	var answers_6 = [
		"en ole koskaan opiskellut erityisesti ääntämistä",
		"olen kiinnittänyt johonkin yksittäiseen asiaan huomiota jos minulle on huomautettu siitä",
		"olen oma-aloitteisesti opiskellut ääntämistä",
		"käytän paljon aikaa ääntämisen opiskeluun",
	];

	var answers_7 = [
		"en ole asunut lainkaan",
		"olen asunut kuukauden tai vähemmän",
		"olen asunut puoli vuotta tai vähemmän",
		"olen asunut kaksi vuotta tai vähemmän",
		"olen asunut pidempään kuin kaksi vuotta",
	];

	var answers_8 = [
		"en ole juurikaan puhunut japaniksi (opetustilanteiden ulkopuolella)",
		"olen kokeillut jutustella, mutta keskustelu on tökkivää eikä siitä tule mitään",
		"pystyn välittämään mitä haluan sanoa, vaikka usein tapahtuu kommunikaatiokatkoksia",
		"pystyn juttelemaan kohtalaisen sujuvasti",
		"en koe, että minulla olisi mitään suurempia ongelmia jutella japaniksi",
	];

	var answers_9 = [
		"En ole.",
		"Olen läpäissyt N5-tason.",
		"Olen läpäissyt N4-tason.",
		"Olen läpäissyt N3-tason.",
		"Olen läpäissyt N2-tason.",
		"Olen läpäissyt N1-tason.",
	];

	var answers_10 = [
		"En ole.",
		"0 - 4 kk (lukukausi) viikoittaista opiskelua",
		"5 - 8 kk (lukuvuosi) viikoittaista opiskelua",
		"9 - 16 kk (2 lukuvuotta) viikoittaista opiskelua",
		"13 - 24 kk (3 lukuvuotta) viikoittaista opiskelua",
		"25 kk tai enemmän viikoittaista opiskelua",
	];

	var answers_11 = [
		"En ole.",
		"Olen käynyt lyhytkestoisessa vaihdossa (max 3 kk)",
		"Olen opiskellut 3 kk - vuoden",
		"Olen opiskellut 1 - 3 vuotta",
		"Olen opiskellut pidempään kuin 3 vuotta",
	];

	var textfield = "textfield";
	var fourfold = "fourfold";
	var languages = "languages";

	var questions = [

	{q: "<p>Kysymme alkuun siitä, millä tavalla olet yleensä tekemisissä japanin kielen kanssa. Kyselyssä oletetaan että olet ainakin jossain määrin aktiivinen japanin kielen opiskelija.</p><p>Kyselyn lopussa on mahdollisuus tarkentaa vastauksia omin sanoin, ja voit aina peruuttaa ja vastata uudelleen kysymyksiin.</p><p>Valitse vaihtoehto, joka kuvaa sinua parhaiten.</p>", a: answers_0},
	{q: "Juttelen ja/tai luen japaniksi sosiaalisessa mediassa, esim. Facebookissa, Twitterissä tai Linessä.", a: answers_1},
	{q: "Luen japanilaisia tekstipainotteisia web-sivuja (esim. blogit, Q&A-sivustot, reseptisivustot...)", a: answers_1},
	{q: "Katson YouTubesta ym. videopalveluista japaninkielisiä v-blogeja tai let's play -videoita.", a: answers_1},
	{q: "Kuuntelen japanilaisia radio-ohjelmia tai podcasteja.", a: answers_1},
	{q: "Kuuntelen japanilaista musiikkia niin että kuuntelen tai selvitän, mitä sanat tarkoittavat.", a: answers_1},
	{q: "Puhun japania livenä japanilaisten<br/>tuttavien kanssa", a: answers_1},
	{q: "Kuuntelen tarkkaavaisesti, miten japanilaiset ääntävät japania.", a: answers_4},
	{q: "Katson animea ilman tekstityksiä.", a: answers_1},
	{q: "Katson japanilaisia draamasarjoja tai näytelmäelokuvia ilman tekstityksiä.", a: answers_1},
	{q: "Katson japanilaisia ajankohtaisohjelmia, komediaa ym. TV-ohjelmia ilman tekstityksiä.", a: answers_1},
	{q: "Katson animea tekstitysten kanssa.", a: answers_1},
	{q: "Katson japanilaisia draamasarjoja tai näytelmäelokuvia tekstitysten kanssa.", a: answers_1},
	{q: "Katson japanilaisia ajankohtaisohjelmia, komediaa ym. TV-ohjelmia tekstitysten kanssa.", a: answers_1},
	{q: "Luen mangaa japaniksi.", a: answers_1},
	{q: "Luen japanilaisia romaaneja, nuortenkirjoja ym. proosaa japaniksi.", a: answers_1},
	{q: "Olen asunut Japanissa yksin tai ei-japaninkielisessä kodissa (asuntola, suomalaisen puolison kanssa tms.)", a: answers_7},
	{q: "Olen asunut Japanissa japaninkielisessä kodissa (vaihtoperhe, japanilainen puoliso tms.)", a: answers_7},
	{q: "Olen opiskellut japanilaisessa lukiossa, yliopistossa tai muussa koulussa.", a: answers_11},
	{q: "Olen matkustanut Japaniin.", a: answers_2},
	{q: "Rento rupattelu japaniksi sujuu minulta.", a: answers_8},
	{q: "Haluaisin osata ääntää japania todella hyvin.", a: answers_5},
	{q: "Olen suorittanut JLPT-kokeen.", a: answers_9},
	{q: "Mihin kohtaa nelikenttää sijoittaisit vahvuutesi japanin kielitaidossasi?", a: fourfold},
	{q: "Opiskelen tällä hetkellä japania käymällä kursseilla.", a: answers_3},
	{q: "Olen opiskellut japania elämäni varrella kursseilla. (Arvio riittää, mutta älä laske mukaan kesälomia yms. vaan varsinaiset opiskeluviikot)", a: answers_10},
	{q: "Olen opiskellut ääntämistä.", a: answers_6},
	{q: "Opiskelen japania jollain muulla tavalla, millä?", a: textfield},
	{q: "Mikä on äidinkielesi?", a: languages},
	{q: "Jos haluat tarkentaa aiempia vastauksia, sana on vapaa:", a: textfield},
	];

	var i = parseInt($("#answered_questions").val()) + 1 || 0;
	var main = $("#main");
	var surveyBox = $("#surveyBox");
	var answerButtons = $("#answerButtons");
	var questionText = $("#questionText");
	var progressMeter = $("#progressMeter");

	var answered = [];
	var alreadyAnswered = false;

	function answerQuestion(answerData, q_number) {
		if (alreadyAnswered === true) {
			return;
		}
		alreadyAnswered = true;
		main.css("min-height", main.css("height"));
		function postAnswer() {
		$.ajax({
			type: "POST",
			url: "/api/eventdata/survey",
			contentType : "application/json",
			data: JSON.stringify(answerData),
			success: function putQuestionNumber() {
				console.log("Successfully posted the answer.");
				$.ajax({
					type: "PUT",
					url: "/api/eventdata/survey/answered_questions",
					contentType : "application/json",
					data: JSON.stringify(q_number),
					success: function() {
						console.log("Successfully saved the answer! Next question!");
						alreadyAnswered = false;
						surveyBox.slideUp(400, function() {
							increment();
							surveyBox.css('opacity', '1.0');
							surveyBox.fadeIn();
						});
					},
					error: function() {
						console.log("connection error, trying again in 3 secs");
						setTimeout(putQuestionNumber, 3000);
					}
				});
			},
			error: function() {
				console.log("connection error, trying again in 3 secs");
				setTimeout(postAnswer, 3000);
			}
		});
		}
		postAnswer();
	}

	function renderQuestion() {

		answerButtons.empty();
		if (i > 0) {
			backButton.show();
		} else {
			backButton.hide();
		}

		if (i < 0) {
			i = 0;
		}

		if (i === questions.length) {
			return surveyReady();
		}

		progressMeter.text("("+(i+1)+"/"+questions.length+")");
		var question = questions[i].q;
		var answers = questions[i].a;
		questionText.html(question);
		if (Array.isArray(answers)) {

			answers.forEach(function(a, j) {
				var button = $('<button class="multilineButton">'+a+'</button>');

				button.appendTo($('<p></p>').appendTo(answerButtons));
				button.one('click', function() {
					surveyBox.css('opacity', '0.4');
					answerQuestion({q: question, a: button.html()}, i)}
				);
			});

		} else if (answers === "textfield") {

			var textarea = $('<textarea></textarea>');

			textarea.appendTo(answerButtons);

			$('<button style="min-width: 50%; margin-top: 1em;">Ok</button>')
				.appendTo(answerButtons)
				.one('click', function() { answerQuestion({q: question, a: textarea.val()}, i) });

		} else if (answers === "fourfold") {

			var suullinen_kirjallinen = 50;
			var ymmarrtaminen_tuottaminen = 50;

			var fourfold = $('<div style="border: 1px solid grey; position: relative; margin: auto; margin-bottom: 1em; width: 16em; height: 16em;">\
				<span style="position: absolute; left: -14%; top: -12%;">kuuntelu</span>\
				<span style="position: absolute; left: -14%; bottom: -12%;">puhuminen</span>\
				<span style="position: absolute; right: -14%; top: -12%;">lukeminen</span>\
				<span style="position: absolute; right: -14%; bottom: -12%;">kirjoittaminen</span>\
				<span style="position: absolute; text-align: center; right: 0; left: 0; top: 0.1em;">ymmärtäminen</span>\
				<span style="position: absolute; text-align: center; right: 0; left: 0; bottom: 0.1em;">tuottaminen</span>\
				<span style="position: absolute; transform: translate(-7em, 7em) rotate(90deg);text-align: center; left: 0; right: 0;">suullinen</span>\
				<span style="position: absolute; transform: translate(7em, 7em) rotate(-90deg);text-align: center; left: 0; right: 0;">kirjallinen</span>\
				<span id="fourfoldSpot" style="border: 1em solid red; border-radius: 1em; position: absolute; margin: -1em; left: 50%; top: 50%;"></span>\
				<div id="fourfoldTouchpad" style="position: absolute; left: 0; right: 0; top: 0; bottom: 0;"></div>\
				</div>').appendTo(answerButtons);
			$("#fourfoldTouchpad").click(function(ev) {
				suullinen_kirjallinen = ev.offsetX/$(this).width()*100;
				ymmarrtaminen_tuottaminen = ev.offsetY/$(this).height()*100;
				console.log(suullinen_kirjallinen, ymmarrtaminen_tuottaminen);
				$("#fourfoldSpot").css("left", suullinen_kirjallinen+"%");
				$("#fourfoldSpot").css("top", ymmarrtaminen_tuottaminen+"%");
			});
			var answerData = {q: question, a: {suullinen_kirjallinen: suullinen_kirjallinen, ymmarrtaminen_tuottaminen: ymmarrtaminen_tuottaminen} };

			$('<button style="min-width: 60%; margin-top: 1em;">Ok</button>')
				.appendTo(answerButtons)
				.one('click', function() { answerQuestion(answerData, i) });

		} else if (answers === "languages") {
			$(' <input type="checkbox" id="suomi"><label for="suomi">suomi</label>\
				<input type="checkbox" id="ruotsi"><label for="ruotsi">ruotsi</label>\
				<label>muu, mikä?</label>\
				<input type="text" id="muukieli">').appendTo(answerButtons);

			$('<button style="min-width: 50%; margin-top: 1em;">Ok</button>')
				.appendTo(answerButtons)
				.one('click', function() {
					var answerData = {q: question, a:
						{	suomi: $("#suomi").is(":checked"),
							ruotsi: $("#ruotsi").is(":checked"),
							muu: $("#muukieli").val(),
						}
					};
					answerQuestion(answerData, i);
				});
		};
	}

	function goBack() {
		i--;
		renderQuestion();
	}
	backButton.click(goBack);
	
	function increment() {
		i++;
		renderQuestion();
	};
	return renderQuestion;
}
var renderQuestion = incrementFactory();
renderQuestion();
})
