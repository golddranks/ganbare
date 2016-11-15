/// <reference path="typings/globals/jquery/index.d.ts" />

$(function() {

var nuggetList = $("#skillNuggets");
var prototypeItem = $("#skillNuggets>li").remove();

$.getJSON("/api/get_nuggets", function(resp) {
	resp.forEach(function(nugget) {
		var newItem = prototypeItem.clone();
		newItem.appendTo(nuggetList).text(nugget.skill_summary);
		nugget.words.forEach(function(word) {
			var subItem = prototypeItem.clone();
			subItem.appendTo(newItem);
			subItem.text(word.word);
		});
	});
});

});
