/// <reference path="typings/globals/jquery/index.d.ts" />
$(function () {
    var prototype_audio_variant = $(".audio_variant").remove();
    var variants = 0;
    function change_upload_button_red() {
        $(this).parent().addClass("buttonHilight");
    }
    function add_audio() {
        var plus_button = $(this);
        var audio_variations = $(".audio_variations");
        var new_variant = prototype_audio_variant.clone();
        variants++;
        audio_variations.val(variants);
        new_variant.children("input").prop("name", "audio_variant_" + variants)
            .change(change_upload_button_red);
        new_variant.children("span").text("Ääntämys " + variants);
        new_variant.insertBefore(plus_button);
    }
    var add_variant_button = $(".addVariant");
    add_variant_button.click(add_audio);
    add_audio.call(add_variant_button); // Make a single variant at first.
});
