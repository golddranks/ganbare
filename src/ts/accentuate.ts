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

	function isYouon(i: number): boolean {
		return (word.charAt(i) === "ゃ" || word.charAt(i) === "ゅ" || word.charAt(i) === "ょ")
	};

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
	for (let i = 0, len = word.length; i < len; i++) {
		if (isYouon(i)) {
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
			let second = mora[1];
			let first_accent = accent;
			let second_accent = accent;
			if (accent == end) {
				first_accent = middle;
				second_accent = end;
			} else if (accent == start_end) {
				first_accent = start_flat;
				second_accent = end;
			} else if (accent == peak) {
				first_accent = start;
				second_accent = end;
			}
			accentuated.push(first_accent);
			accentuated.push(first);
			accentuated.push("</span>");
			accentuated.push(second_accent);
			accentuated.push(second);
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
