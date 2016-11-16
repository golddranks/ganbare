/// <reference path="typings/globals/jquery/index.d.ts" />

function accentuate_kana(word) {

	var empty = '<span class="accent">';
	var middle = '<span class="accent"><img src="/static/images/accent_middle.png">';
	var start = '<span class="accent"><img src="/static/images/accent_start.png">';
	var end = '<span class="accent"><img src="/static/images/accent_end.png" class="accent">';
	var flat_end = '<span class="accent"><img src="/static/images/accent_end_flat.png">';
	var start_end = '<span class="accent"><img src="/static/images/accent_start_end.png">';
	var start_end_flat = '<span class="accent"><img src="/static/images/accent_start_end_flat.png">';
	var start_end_flat_short = '<span class="accent"><img src="/static/images/accent_start_end_flat_short.png">';
	var peak = '<span class="accent"><img src="/static/images/accent_peak.png">';
	
	function isAccentMark(i) {
		return (word.charAt(i) === "*" || word.charAt(i) === "・")
	};

	var accentuated = [""];
	var ended = false;
	for (var i = 0, len = word.length; i < len; i++) {

		if (isAccentMark(i)) {
			continue;
		} else if (word.length === 1) {
			accentuated.push(start_end_flat_short);
		} else if (i === 0 && isAccentMark(i+1)) {
			accentuated.push(start_end);
			ended = true;
		} else if (i === 1 && !ended && isAccentMark(i+1)) {
			accentuated.push(peak);
			ended = true;
		} else if (i === 1 && !ended && i === len-1) {
			accentuated.push(start_end_flat);
		} else if (i === 1 && !ended) {
			accentuated.push(start);
		} else if (i > 1 && !ended && i === len-1) {
			accentuated.push(flat_end);
		} else if (i > 1 && !ended && isAccentMark(i+1)) {
			accentuated.push(end);
			ended = true;
		} else if (i > 1 && !ended && !isAccentMark(i+1)) {
			accentuated.push(middle);
		} else {
			accentuated.push(empty);
		}
		accentuated.push(word.charAt(i));
		accentuated.push("</span>");
	}
	return accentuated.join("");
}


$(function(){

var prototype_audio_variant = $(".audio_variant").remove();

var variants = 0;

function change_upload_button_red(){
	$(this).parent().addClass("buttonHilight");
}

function add_audio() {
	var plus_button = $(this);
	var audio_variations = $(".audio_variations");
	var new_variant = prototype_audio_variant.clone();

	variants++;
	audio_variations.val(variants);

	new_variant.children("input").prop("name", "audio_variant_"+variants)
		.change(change_upload_button_red);
	new_variant.children("span").text("Ääntämys "+variants);

	new_variant.insertBefore(plus_button);
}

var add_variant_button = $(".addVariant");
add_variant_button.click(add_audio);

add_audio.call(add_variant_button); // Make a single variant at first.

});
