/// <reference path="typings/globals/jquery/index.d.ts" />
$(function () {
    var lowest_fieldset = $("#choice_1");
    var lowest_fieldset_number_input = $("#lowest_fieldset");
    var lowest_fieldset_number = 1;
    var variants = { choice_1: 1 };
    function change_upload_button_red() {
        $(this).parent().css('background-color', '#F8DDDD').css('border-color', '#FF6666');
    }
    $("input[type=file]").change(change_upload_button_red);
    function add_answer_fieldset() {
        var old_lowest_fieldset = lowest_fieldset;
        lowest_fieldset = old_lowest_fieldset.clone();
        lowest_fieldset_number++;
        variants["choice_" + lowest_fieldset_number] = 1;
        lowest_fieldset_number_input.val(lowest_fieldset_number);
        lowest_fieldset.prop("id", "choice_" + lowest_fieldset_number);
        lowest_fieldset.children("label")
            .text(lowest_fieldset_number + ". vastausvaihtoehto");
        lowest_fieldset.find(".question")
            .prop("name", "choice_" + lowest_fieldset_number + "_q_variant_1")
            .change(change_upload_button_red);
        lowest_fieldset.find(".answer_audio")
            .prop("name", "choice_" + lowest_fieldset_number + "_answer_audio")
            .change(change_upload_button_red);
        lowest_fieldset.find(".answer_text")
            .prop("name", "choice_" + lowest_fieldset_number + "_answer_text");
        lowest_fieldset.find(".q_variations")
            .prop("name", "choice_" + lowest_fieldset_number + "_q_variations");
        lowest_fieldset.insertAfter(old_lowest_fieldset);
    }
    function add_q_audio(event) {
        var plus_button = $(this);
        var fieldset = plus_button.closest(".fieldset");
        var q_variations = fieldset.find(".q_variations");
        var old_variant = plus_button.prev();
        var new_variant = old_variant.clone();
        var choice_number = fieldset.prop("id");
        variants[choice_number]++;
        var variant_number = variants[choice_number];
        q_variations.val(variant_number);
        new_variant.children("input").prop("name", choice_number + "_q_variant_" + variant_number);
        new_variant.children("span").text("Kysymys (audio " + variant_number + ")");
        new_variant.insertAfter(old_variant);
    }
    add_answer_fieldset();
    $("#add_answer").click(add_answer_fieldset);
    $(".addVariant").click(add_q_audio);
});
