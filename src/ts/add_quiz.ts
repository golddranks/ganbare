/// <reference path="typings/globals/jquery/index.d.ts" />


$(function(){

var lowest_fieldset = $("#choice_1");
var lowest_fieldset_number_input = $("#lowest_fieldset");
var lowest_fieldset_number = 1;

function change_upload_button_red(){
	$(this).parent().css('background-color', '#F8DDDD').css('border-color', '#FF6666');
}

$("input[type=file]").change(change_upload_button_red);

function add_answer_fieldset() {
	var old_lowest_fieldset = lowest_fieldset;
	lowest_fieldset = old_lowest_fieldset.clone();
	lowest_fieldset_number++;
	lowest_fieldset_number_input.val(lowest_fieldset_number);

	lowest_fieldset.prop("id", "choice_"+lowest_fieldset_number )
		.insertAfter(old_lowest_fieldset);
	lowest_fieldset.children("label")
		.text(lowest_fieldset_number+". vastausvaihtoehto");
	lowest_fieldset.find(".question")
		.prop("name", "choice_"+lowest_fieldset_number+"_question" )
		.change(change_upload_button_red);
	lowest_fieldset.find(".answer_audio")
		.prop("name", "choice_"+lowest_fieldset_number+"_answer_audio" )
		.change(change_upload_button_red);
	lowest_fieldset.find(".answer_text")
		.prop("name", "choice_"+lowest_fieldset_number+"_answer_text" );
}
add_answer_fieldset();

$("#add_answer").click(add_answer_fieldset);
});
