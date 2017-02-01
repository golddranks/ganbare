/// <reference path="typings/globals/jquery/index.d.ts" />

$(function() {

function moraize(word: string): {mora:string, accent:string}[] {

	function isYouon(i: number): boolean {
		return (word.charAt(i) === "ゃ" || word.charAt(i) === "ゅ" || word.charAt(i) === "ょ")
	};

	function isAccented(i: number): boolean {
		return (word.charAt(i) === "・" || word.charAt(i) === "／" || word.charAt(i) === "＝")
	}

	let moras: {mora:string, accent:string}[] = new Array();
	for (let i = 0, len = word.length; i < len; i++) {
		if (isYouon(i)) {
			moras[moras.length-1].mora += word.charAt(i);
		} else if (isAccented(i)) {
			moras[moras.length-1].accent = word.charAt(i);
		} else {
			moras.push({mora: word.charAt(i), accent: null});
		}
	}

	return moras;
}

function accentuate(word: string, showAccent: boolean): string {
	
	if (!showAccent) {		
		return word.replace("・", "").replace("*", "");		
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
	
	function isAccentMark(i: number): boolean {
		return (moras[i].accent === "・")
	};

	function isRisingAccentMark(i: number): boolean {
		return (moras[i].accent === "／")
	};

	function isFlatAccentMark(i: number): boolean {
		return (moras[i].accent === "＝")
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
			if (isRisingAccentMark(i-1) && isAccentMark(i)) {
				accent = peak;
				started = true;
				ended = true;
			} else if (isRisingAccentMark(i-1) && isFlatAccentMark(i)) {
				accent = start_end_flat;
				started = true;
				ended = true;
			} else if (isRisingAccentMark(i-1)) {
				accent = start;
				started = true;
			} else if (isAccentMark(i)) {
				accent = end;
				ended = true;
			} else if (isFlatAccentMark(i)) {
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
			if (i === 0 && isAccentMark(i)) {
				accent = start_end;
				ended = true;
			} else if (moras.length === 1) {
				accent = start_end_flat_short;
			} else if (i === 1 && !ended && isAccentMark(i)) {
				accent = peak;
				ended = true;
			} else if (i === 1 && !ended && i === len-1) {
				accent = start_end_flat;
			} else if (i === 1 && !ended) {
				accent = start;
			} else if (i > 1 && !ended && i === len-1) {
				accent = flat_end;
			} else if (i > 1 && !ended && isAccentMark(i)) {
				accent = end;
				ended = true;
			} else if (i > 1 && !ended && !isAccentMark(i)) {
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
