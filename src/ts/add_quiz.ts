/// <reference path="typings/globals/jquery/index.d.ts" />


$(function(){

var prototype_fieldset = $("#proto_fieldset").remove();
var prototype_q_variant = prototype_fieldset.find(".q_variant").remove();
var answer_button = $("#add_answer");

var lowest_fieldset_number_input = $("#lowest_fieldset");
var lowest_fieldset_number = 0;
var variants = { };

function change_upload_button_red(){
	$(this).parent().addClass("buttonHilight");
}

function add_q_audio() {
	var plus_button = $(this);
	var fieldset = plus_button.closest(".fieldset");
	var q_variations = fieldset.find(".q_variations");
	var new_variant = prototype_q_variant.clone();

	var choice_number = fieldset.prop("id");
	variants[choice_number]++;
	var variant_number = variants[choice_number];
	q_variations.val(variant_number);

	new_variant.children("input").prop("name", choice_number+"_q_variant_"+variant_number)
		.change(change_upload_button_red);
	new_variant.children("span").text("Kysymys (audio "+variant_number+")");

	new_variant.insertBefore(plus_button);
}

function add_answer_fieldset() {
	var lowest_fieldset = prototype_fieldset.clone();
	lowest_fieldset_number++;
	variants["choice_"+lowest_fieldset_number] = 0;
	lowest_fieldset_number_input.val(lowest_fieldset_number);

	lowest_fieldset.prop("id", "choice_"+lowest_fieldset_number );
	lowest_fieldset.children("label")
		.text(lowest_fieldset_number+". vastausvaihtoehto");
	lowest_fieldset.find(".answer_audio")
		.prop("name", "choice_"+lowest_fieldset_number+"_answer_audio" )
		.change(change_upload_button_red);
	lowest_fieldset.find(".answer_text")
		.prop("name", "choice_"+lowest_fieldset_number+"_answer_text" );
	lowest_fieldset.find(".q_variations")
		.prop("name", "choice_"+lowest_fieldset_number+"_q_variations" );
	var add_variant_button = lowest_fieldset.find(".addVariant");
	add_variant_button.click(add_q_audio);

	add_q_audio.call(add_variant_button);

	lowest_fieldset.insertBefore(answer_button.parent());
}

add_answer_fieldset();
add_answer_fieldset();
answer_button.click(add_answer_fieldset);

});
