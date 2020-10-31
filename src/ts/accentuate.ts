/// <reference path="typings/globals/jquery/index.d.ts" />

$(function() {

/* menu */

var main = $("#main");
var settingsArea = $("#settings");
var menuButton = $("#menuButton");

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

// Accentuate 1.0

function moraize(word: string): {mora:string, rising: boolean, falling: boolean, flatEnd: boolean}[] {
	console.log(word);
	function isYouon(i: number): boolean {
		let c = word.charAt(i);
		return (c === "ゃ" || c === "ゅ" || c === "ょ" || c === "y" || c === "h" || c === "s")
	};

	function isVowel(i: number): boolean {
		let c = word.charAt(i);
		console.log("is vowel", c, (c === "a" || c === "i" || c === "u" || c === "e" || c === "o"));
		return (c === "a" || c === "i" || c === "u" || c === "e" || c === "o")
	}

	function isRising(i: number): boolean {
		return (word.charAt(i) === "／")
	}

	function isFalling(i: number): boolean {
		return (word.charAt(i) === "・")
	}

	function isFlatEnd(i: number): boolean {
		return (word.charAt(i) === "＝")
	}

	let moras: {mora:string, rising: boolean, falling: boolean, flatEnd: boolean}[] = new Array();
	let rising = false;
	let independent = true;
	for (let i = 0; i < word.length; i++) {
		if (isYouon(i) && !independent) {
			moras[moras.length-1].mora += word.charAt(i);
		} else if (isVowel(i) && !independent) {
			independent = true;
			moras[moras.length-1].mora += word.charAt(i);
		} else if (isRising(i)) {
			rising = true;
		} else if (isFalling(i)) {
			moras[moras.length-1].falling = true;
		} else if (isFlatEnd(i)) {
			moras[moras.length-1].flatEnd = true;
		} else {
			moras.push({mora: word.charAt(i), rising: rising, falling: false, flatEnd: false});
			rising = false;
			independent = false;
		}
	}

	console.log("moraized:", moras);

	return moras;
}

function accentuate(word: string, showAccent: boolean): string {

	if (!showAccent) {		
		return word.replace("・", "").replace("*", "").replace("＝", "").replace("／", "");		
	}
	let moras = moraize(word);

	var empty = '<span class="accent">';
	var middle = '<span class="accent" style="background-image: url(/static/images/accent_middle.png);">';
	var start = '<span class="accent" style="background-image: url(/static/images/accent_start.png);">';
	var start_flat = '<span class="accent" style="background-image: url(/static/images/accent_start_flat.png);">';
	var end = '<span class="accent" style="background-image: url(/static/images/accent_end.png);">';
	var flat_end = '<span class="accent" style="background-image: url(/static/images/accent_end_flat.png);">';
	var start_end = '<span class="accent" style="background-image: url(/static/images/accent_start_end.png);">';
	var start_end_flat = '<span class="accent" style="background-image: url(/static/images/accent_start_end_flat.png);">';
	var start_end_flat_short = '<span class="accent" style="background-image: url(/static/images/accent_start_end_flat_short.png);">';
	var peak = '<span class="accent" style="background-image: url(/static/images/accent_peak.png);">';
	
	function isFalling(i: number): boolean {
		return (moras[i].falling)
	};

	function isRising(i: number): boolean {
		return (moras[i].rising)
	};

	function isFlat(i: number): boolean {
		return (moras[i].flatEnd)
	};

	function pushAccents(accentuated: string[], accent: string, i: number): void {
		let mora = moras[i].mora;
		if (mora.length === 1) {
			accentuated.push(accent);
			accentuated.push(mora);
			accentuated.push("</span>");
		} else {
			let first = mora[0];
			let final = mora[mora.length-1];
			let first_accent = accent;
			let final_accent = accent;
			if (accent == end) {
				first_accent = middle;
				final_accent = end;
			} else if (accent == start_end) {
				first_accent = start_flat;
				final_accent = end;
			} else if (accent == peak) {
				first_accent = start;
				final_accent = end;
			} else if (accent == start_end_flat) {
				first_accent = start;
				final_accent = flat_end;
			} else if (accent == start) {
				first_accent = start;
				final_accent = flat_end;
			}
			accentuated.push(first_accent);
			accentuated.push(first);
			accentuated.push("</span>");
			for (var i = 1; i < mora.length-1; i++) {
				if (accent === empty) {
					accentuated.push(empty);
				} else {
					accentuated.push(middle);
				}
				accentuated.push(mora[i]);
				accentuated.push("</span>");
			}
			accentuated.push(final_accent);
			accentuated.push(final);
			accentuated.push("</span>");
		}
	}

	var accentuated = [""];
	var ended = false;

	if (word.indexOf("／") >= 0) {
		var started = false;
		for (var i = 0, len = moras.length; i < len; i++) {

			let accent = null;
			if (isRising(i) && isFalling(i)) {
				accent = peak;
				started = true;
				ended = true;
			} else if (isRising(i) && isFlat(i)) {
				accent = start_end_flat;
				started = true;
				ended = true;
			} else if (isRising(i)) {
				accent = start;
				started = true;
			} else if (isFalling(i)) {
				accent = end;
				ended = true;
			} else if (isFlat(i)) {
				accent = flat_end;
				ended = true;
			} else if (!ended && started) {
				accent = middle;
			} else {
				accent = empty;
			}

			pushAccents(accentuated, accent, i);
		}
	} else {

		for (var i = 0, len = moras.length; i < len; i++) {

			let accent = null;
			if (i === 0 && isFalling(i)) {
				accent = start_end;
				ended = true;
			} else if (moras.length === 1) {
				accent = start_end_flat_short;
			} else if (i === 1 && !ended && isFalling(i)) {
				accent = peak;
				ended = true;
			} else if (i === 1 && !ended && i === len-1) {
				accent = start_end_flat;
			} else if (i === 1 && !ended) {
				accent = start;
			} else if (i > 1 && !ended && i === len-1) {
				accent = flat_end;
			} else if (i > 1 && !ended && isFalling(i)) {
				accent = end;
				ended = true;
			} else if (i > 1 && !ended && !isFalling(i)) {
				accent = middle;
			} else {
				accent = empty;
			}
			pushAccents(accentuated, accent, i);
		}
	}
	return accentuated.join("");
}

$(".accentuate").html(function(i, text) { return accentuate(text, true); });

})

function romanize(word: string): string {
	let output = [];

	let kana = "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんがぎぐげござじずぜぞだぢづでどぱぴぷぺぽばびぶべぼ";
	let romaji = ["a","i","u","e","o","ka","ki","ku","ke","ko","sa","shi","su","se","so","ta","chi","tsu","te","to","na","ni","nu","ne","no","ha","hi","fu","he","ho","ma","mi","mu","me","mo","ya","yu","yo","ra","ri","ru","re","ro","wa","o","n","ga","gi","gu","ge","go","za","ji","zu","ze","zo","da","ji","dzu","de","do","pa","pi","pu","pe","po","ba","bi","bu","be","bo"];

	let youon = "ゃゅょ";
	let youon_romaji = "auo";

	for (let i = 0; i < word.length; i++) {
		let j = kana.indexOf(word.charAt(i));
		let y = youon.indexOf(word.charAt(i));
		if (j >= 0) {
			output.push(romaji[j]);
		} else if (y >= 0) {
			output[output.length-1] = output[output.length-1].replace("i", youon_romaji[y]);
		} else {
			output.push(word.charAt(i));
		}
	}
	return output.join("")
}