/// <reference path="typings/globals/jquery/index.d.ts" />
/// <reference path="typings/globals/howler/index.d.ts" />

$(function() {

function getRandomColor() {
    var letters = '0123456789ABCDEF';
    var color = '#';
    for (var i = 0; i < 6; i++ ) {
        color += letters[Math.floor(Math.random() * 8 + 8)];
    }
    return color;
}

function stripAccents(word: string): string {
	return word.replace('・', '').replace('＝', '').replace('／', '');
}

function createSemaphore(count: number) : (argument?: any) =>void {

	var semaphore = count;
	var closureCallback: (argument: any)=>void = null;
	var closureArg: any = null;

	return (argument?: any) => {
		if (closureCallback === null && typeof argument === "function") {
			closureCallback = argument;
		} else if (closureArg === null) {
			closureArg = argument;
		};
		semaphore--;
		if (semaphore > 0) { return; };
		closureCallback(closureArg);
	};
}

function accentuate(word: string): string {

	var empty = '<span class="accent">';
	var middle = '<span class="accent" style="background-image: url(/static/images/accent_middle.png);">';
	var start = '<span class="accent" style="background-image: url(/static/images/accent_start.png);">';
	var end = '<span class="accent" style="background-image: url(/static/images/accent_end.png);">';
	var flat_end = '<span class="accent" style="background-image: url(/static/images/accent_end_flat.png);">';
	var start_end = '<span class="accent" style="background-image: url(/static/images/accent_start_end.png);">';
	var start_end_flat = '<span class="accent" style="background-image: url(/static/images/accent_start_end_flat.png);">';
	var start_end_flat_short = '<span class="accent" style="background-image: url(/static/images/accent_start_end_flat_short.png);">';
	var peak = '<span class="accent" style="background-image: url(/static/images/accent_peak.png);">';
	
	function isAccentMark(i: number): boolean {
		return (word.charAt(i) === "*" || word.charAt(i) === "・")
	};

	function isRisingAccentMark(i: number): boolean {
		return (word.charAt(i) === "／")
	};

	function isFlatAccentMark(i: number): boolean {
		return (word.charAt(i) === "＝")
	};

	var accentuated = [""];
	var ended = false;

	if (word.indexOf("／") >= 0) {
		var started = false;
		for (var i = 0, len = word.length; i < len; i++) {
			if (isAccentMark(i) || isFlatAccentMark(i) || isRisingAccentMark(i)) {
				continue;
			} else if (isRisingAccentMark(i-1) && isAccentMark(i+1)) {
				accentuated.push(peak);
				started = true;
				ended = true;
			} else if (isRisingAccentMark(i-1) && isFlatAccentMark(i+1)) {
				accentuated.push(start_end_flat);
				started = true;
				ended = true;
			} else if (isRisingAccentMark(i-1)) {
				accentuated.push(start);
				started = true;
			} else if (isAccentMark(i+1)) {
				accentuated.push(end);
				ended = true;
			} else if (isFlatAccentMark(i+1)) {
				accentuated.push(flat_end);
				ended = true;
			} else if (!ended && started) {
				accentuated.push(middle);
			} else {
				accentuated.push(empty);
			}
			accentuated.push(word.charAt(i));
			accentuated.push("</span>");
		}
	} else {
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
	}
	return accentuated.join("");
}



$("body").click(function() {
	var closeEditEvent = new Event('closeEdit');
	this.dispatchEvent(closeEditEvent);
});

var main = $("#main");
var n_list = $("#main ul");
var prifilter_value = $("#priorityFilterValue");
var prifilter_toggle = $("#priorityFilterToggle");
var pubstuff_toggle = $("#onlyPublishedStuffToggle");
var unpubstuff_toggle = $("#onlyUnPublishedStuffToggle");

let priority_filter_value = prifilter_value.val();
let priority_filter_toggle = prifilter_toggle.is(":checked");
let pubstuff_value = pubstuff_toggle.is(":checked");
let unpubstuff_value = unpubstuff_toggle.is(":checked");

var nugget_resp = null;
var bundle_resp = null;
var narrator_resp = null;

prifilter_value.change(() => {
	n_list.html("");
	priority_filter_value = prifilter_value.val();
	loading_msg.show();
	loading_msg.text("Loaded. Rendering content. ");
	drawList(nugget_resp, bundle_resp, narrator_resp);
});

prifilter_toggle.change(() => {
	n_list.html("");
	priority_filter_toggle = prifilter_toggle.is(":checked");
	loading_msg.show();
	loading_msg.text("Loaded. Rendering content. ");
	drawList(nugget_resp, bundle_resp, narrator_resp);
});

pubstuff_toggle.change(() => {
	n_list.html("");
	pubstuff_value = pubstuff_toggle.is(":checked");
	unpubstuff_value = false;
	unpubstuff_toggle.prop("checked", false);
	loading_msg.show();
	loading_msg.text("Loaded. Rendering content. ");
	drawList(nugget_resp, bundle_resp, narrator_resp);
});

unpubstuff_toggle.change(() => {
	n_list.html("");
	unpubstuff_value = unpubstuff_toggle.is(":checked");
	pubstuff_value = false;
	pubstuff_toggle.prop("checked", false);
	loading_msg.show();
	loading_msg.text("Loaded. Rendering content. ");
	drawList(nugget_resp, bundle_resp, narrator_resp);
});

let currentlyDragged = null;
let dropTargets = new Array();

function dropHandler(event) {
    event.preventDefault();  
    event.stopPropagation();
    $(this).removeClass("dropTarget");
    console.log("drop:", event);
    this.append(currentlyDragged);
    let file = $(currentlyDragged).data("file");
    currentlyDragged = null;
    file.bundle_id = $(this).data("bundle_id");

    while(dropTargets.length > 0) {
    	$(dropTargets.pop()).removeClass("dropTarget");
	}

	$.ajax({
		type: 'PUT',
		url: "/api/audio_files/"+file.id,
		contentType: "application/json",
		data: JSON.stringify(file),
		success: function(resp) {
			console.log("Updated audio_file!");
		},
	});
}

function dragEnterHandler(event) {
    event.preventDefault();  
    event.stopPropagation();
	dropTargets.push(this);
    $(this).addClass("dropTarget");
}

function dragOverHandler(event) {
    event.preventDefault();  
    event.stopPropagation();
}

function dragLeaveHandler(event) {
    event.preventDefault();  
    event.stopPropagation();
    $(this).removeClass("dropTarget");
}

function dragStartHandler(event) {
	dropTargets.push(this);
	this.style.opacity = '0.4';
	currentlyDragged = this;
	setTimeout(() => {$(this).parent().addClass("dropTarget")}, 0);
}
function dragEndHandler(event) {
	this.style.opacity = '1.0';
}

function drawList(nugget_resp, bundle_resp, narrator_resp) {

	var audio_bundles = {};

	bundle_resp.forEach(function(tuple) {
		var bundle = tuple[0];
		var files = tuple[1];
		bundle.files = files;
		audio_bundles[bundle.id] = bundle;
	});

	var narrators = {};

	narrator_resp.forEach(function(n) {
		n.color = getRandomColor();
		narrators[n.id] = n;
	});

	var proto_trash_button = $('<button class="compact narrDelButton" style="float: right;"><i class="fa fa-trash" aria-hidden="true"></i></button>');
	var proto_update_button = $('<button class="compact narrUpdateButton" style="float: right;"><i class="fa fa-arrow-down" aria-hidden="true"></i></button>');
	let proto_audio_button = $('<button class="compact" style="background-color: white;" draggable="true" title="undefined"><img draggable="false" src="/static/images/speaker_teal.png" class="soundicon"></button>');
	let proto_bundle = $('<div class="bordered weak" style="display: inline-block;" title="undefined"></div>');
	let proto_span = $('<span></span>');

	function createBundle(id, element, update_value_cb) {
		var listname = audio_bundles[id].listname;
		var bundle_html = proto_bundle.clone().appendTo(element).prop('title', 'Name: '+listname);

		function init_ui(id) {
			bundle_html.empty();
			var id_span = proto_span.clone()
				.appendTo(bundle_html)
				.text('ID '+id);

			function setup_editor() {
				id_span.one('click', function(ev) {
					id_span.empty();
					$('<input type="text" value="'+id+'" style="display: inline-block; width: 4em;">')
						.appendTo(id_span)
						.focus()
						.blur(function() {
							id_span.html('<i class="fa fa-spinner fa-spin fa-fw"></i>');
							setup_editor();
							update_value_cb($(this).val(), init_ui);
						});
				});
			}
			setup_editor();

			bundle_html.data("bundle_id", id);
			bundle_html.on("drop", dropHandler);
			bundle_html.on("dragenter", dragEnterHandler);
			bundle_html.on("dragleave", dragLeaveHandler);
			bundle_html.on("dragover", dragOverHandler);

			audio_bundles[id].files.forEach(function(file) {
				var narrator_published = narrators[file.narrators_id].published;
				if ( ! narrator_published) {
					return;
				}
				var narrator_name = narrators[file.narrators_id].name;
				var narrator_color = narrators[file.narrators_id].color;
				var audio_button = proto_audio_button.clone().appendTo(bundle_html);
				audio_button.data("file", file);
				audio_button.prop('title', 'ID: '+file.id+' Narrator: '+narrator_name);
				audio_button.css('background-color', narrator_color);
				audio_button.on('dragstart', dragStartHandler);
				audio_button.on('dragend', dragEndHandler);
				audio_button.click(function() {
					var audio = new Howl({ src: ['/api/audio/'+file.id+'.mp3']});
					audio.play();
				});
			});
		}

		init_ui(id);
	};

	if (nugget_resp.length === 0) {
		n_list.append("<h2>No skill nuggets exist at the moment.</h2>");
	}

	let shown_nuggets = new Array();
	for (let i = 0, len = nugget_resp.length; i < len; i++) {

		// Drawing only high-priority items
		let tuple = nugget_resp[i];
		let words = tuple[1][0];
		var questions = tuple[1][1];
		var exercises = tuple[1][2];

		let thereIsPriorityStuff = words.some((w) => { return w.priority == priority_filter_value });
		let thereIsPublishedStuff = words.some((i) => { return i.published })
				|| questions.some((i) => { return i.published })
				|| exercises.some((i) => { return i.published });

		if (( ! priority_filter_toggle || thereIsPriorityStuff )
			&& ( ! pubstuff_value || thereIsPublishedStuff )
			&& ( ! unpubstuff_value || ! thereIsPublishedStuff ) ) {
			shown_nuggets.push(tuple);
		}
	}
	$("#filteredAmount").show().text("Showing "+shown_nuggets.length+" of "+nugget_resp.length+" skills");

	let nugget_index = 0;
	function drawNuggetAsync() {
		for (let i = 0; i < 5; i++) {
			let tuple = shown_nuggets[nugget_index];

			if (tuple === undefined) {
				loading_msg.hide();
				return; // Nothing left to render;
			}
			drawNugget(tuple, nugget_index);

			nugget_index += 1;
		}
		setTimeout(drawNuggetAsync, 0);
	}
	drawNuggetAsync();

	function drawNugget(tuple, nugget_index) {

		var nugget = tuple[0];
		var words = tuple[1][0];
		var questions = tuple[1][1];
		var exercises = tuple[1][2];

		var n_item = $('<li style="width: 100%"><hr style="border-width: 3px"></li>').appendTo(n_list);
		var skill_nugget_header = $("<h2>Skill nugget: " + nugget.skill_summary + "</h2>")
			.appendTo(n_item);
		var trash_button = proto_trash_button.clone()
			.appendTo(skill_nugget_header)
			.click(function() {
				$.ajax({
					type: 'DELETE',
					url: "/api/skills/"+nugget.id,
					contentType: "application/json",
					data: "",
					success: function() {
						n_item.remove();
					}, 
					error: function() {
						alert("Can't remove this! (Try removing things that depend on it first.)");
					},
				});
			});

		var c_list = $("<ul></ul>").appendTo(n_item);
		
		words.forEach(function(word, index) {
			var c_item = $('<li style="width: 100%"></li>').appendTo(c_list);
			var c_header = $('<h3></h3>').html('Word ('+word.id+'): ' + accentuate(word.word)).appendTo(c_item);

			var trash_button = proto_trash_button.clone()
				.appendTo(c_header)
				.click(function() {
					$.ajax({
						type: 'DELETE',
						url: "/api/words/"+word.id,
						contentType: "application/json",
						data: "",
						success: function() {
							words.splice(index, 1);
							c_item.remove();
							check_init_autocreate_buttons();
						}, 
					});
				});


			var id = "n"+nugget_index+"w"+index;
			var c_info = $('<div><label for="'+id+'">public</label></div>').appendTo(c_item);

			var checkbox = $('<input type="checkbox" id="'+id+'">').prependTo(c_info);
			if (word.published) {
				checkbox.prop("checked", true);
			};
			checkbox.change(function() {
				var request= { type: 'PUT', url: null };
				if (this.checked) {
					request.url = '/api/words/'+word.id+'?publish';
				} else {
					request.url = '/api/words/'+word.id+'?unpublish';
				};
				$.ajax(request);
			});

			var c_edit = $('<input type="button" value="show" class="linklike">').appendTo(c_info);
			var skill_label = $('<label class="bordered weak" style="display: inline-block">Skill:</label>').appendTo(c_info);
			var skill_level = $('<input type="number" class="compact" value="'+word.skill_level+'" class="linklike">').appendTo(skill_label);
			skill_level.change(function() {
				let updated_value = $(this).val();
				$.ajax({
					type: 'PUT',
					url: "/api/words/"+word.id,
					contentType: "application/json",
					data: JSON.stringify({skill_level: updated_value}),
					success: function(resp) {
						word.skill_level = updated_value;
						console.log("Updated skill_level!");
					},
				});
			});

			var priority_label = $('<label class="bordered weak" style="display: inline-block">Priority:</label>').appendTo(c_info);
			var priority_level = $('<input type="number" class="compact" value="'+word.priority+'" class="linklike">').appendTo(priority_label);
			priority_level.change(function() {
				let updated_value = $(this).val();
				$.ajax({
					type: 'PUT',
					url: "/api/words/"+word.id,
					contentType: "application/json",
					data: JSON.stringify({priority: updated_value}),
					success: function(resp) {
						word.priority = updated_value;
						console.log("Updated priority!");
					},
				});
			});

			createBundle(word.audio_bundle, c_info, function(updated_value, update_bundle_cb) {
				$.ajax({
					type: 'PUT',
					url: "/api/words/"+word.id,
					contentType: "application/json",
					data: JSON.stringify({audio_bundle: updated_value}),
					success: function(resp) {
						word.audio_bundle = updated_value;
						update_bundle_cb(updated_value);
						console.log("Updated audio bundle!");
					},
				});
			});

			var wordLatestResp = word;

			var w_word_okayToUpdate = false;
			var c_body = $('<section class="bordered cardBody" style="margin-bottom: 3em;"></section>').appendTo(c_info).hide();
			var w_word = $('<p class="wordShowKana"></p>').appendTo(c_body).html(accentuate(word.word));
			w_word.click(function w_wordStartEdit(ev){
				ev.stopPropagation();
				w_word_okayToUpdate = false;
				var w_word_edit = $('<p></p>');
				var wordEdit = $('<input class="wordShowKana" type="text" value="'+word.word+'">').appendTo(w_word_edit);
				w_word.replaceWith(w_word_edit);
				$("body").one('click', function(ev){
					word.word = wordLatestResp.word;
					w_word_okayToUpdate = true;
					w_word.html(accentuate(word.word));

					w_word_edit.replaceWith(w_word);
					w_word.click(w_wordStartEdit);
				});
				w_word_edit.click(function(ev){ ev.stopPropagation(); });
				wordEdit.on('input', function() {
					word.word = wordEdit.val();
					c_header.html('Word: ' + accentuate(word.word));
					var request = {
						type: 'PUT',
						url: "/api/words/"+word.id,
						contentType: "application/json",
						data: JSON.stringify({word: word.word}),
						success: function(resp) {
							wordLatestResp = resp;
							if (w_word_okayToUpdate) {
								word.word = wordLatestResp.word;
								w_word.html(accentuate(word.word));
							}
						},
					};
					$.ajax(request);
				});
			});
			var w_explanation = $('<div class="wordExplanation" contenteditable="true"></div>').appendTo(c_body).html(word.explanation);

			var w_explanation_okayToUpdate = false;
			w_explanation.on('blur', function() {
				word.explanation = wordLatestResp.explanation;
				w_explanation_okayToUpdate = true;
				w_explanation.html(word.explanation);
			});
			w_explanation.on('input', function() {
				w_explanation_okayToUpdate = false;
				var request = {
					type: 'PUT',
					url: "/api/words/"+word.id,
					contentType: "application/json",
					data: JSON.stringify({explanation: w_explanation.html()}),
					success: function(resp) {
						wordLatestResp = resp;
						if (w_explanation_okayToUpdate) {
							word.explanation = wordLatestResp.explanation;
							w_explanation.html(word.explanation);
						}
					},
				};
				$.ajax(request);
			});

			function showBody() {
				c_edit.val("Hide").click(function() { c_body.hide(); c_edit.val("Show"); c_edit.click(showBody); });
				c_body.show();
			};

			c_edit.click(showBody);
		});

		function createQuestionEntry(tuple, index) {
			var question = tuple[0];
			var answers = tuple[1];

			var c_item = $('<li style="width: 100%"></li>').appendTo(c_list);
			var c_header = $('<h3>Question ('+question.id+'): ' + question.q_name + '</h3>').appendTo(c_item);
			var trash_button = proto_trash_button.clone()
				.appendTo(c_header)
				.click(function() {
					$.ajax({
						type: 'DELETE',
						url: "/api/questions/"+question.id,
						contentType: "application/json",
						data: "",
						success: function() {
							c_item.remove();
							let removeIndex = questions.indexOf(tuple);
							questions.splice(removeIndex, 1);
							check_init_autocreate_buttons();
						}, 
					});
				});


			let same_skill_words = new Array();
			words.forEach((w) => {
				let skill_level = Math.max(2, w.skill_level);
				if (skill_level === question.skill_level) {
					same_skill_words.push(w);
				}
			});

			if (same_skill_words.length == 2) {
				proto_update_button
					.clone()
					.appendTo(c_header)
					.click(function() {
	
						let a1_data = {
										question_id: question.id,
										a_audio_bundle: null,
										q_audio_bundle: same_skill_words[0].audio_bundle,
										answer_text: same_skill_words[0].explanation,
									};
						let a2_data = {
										question_id: question.id,
										a_audio_bundle: null,
										q_audio_bundle: same_skill_words[1].audio_bundle,
										answer_text: same_skill_words[1].explanation,
									};
						let sema = createSemaphore(3);
						sema(() => {
							c_item.remove();
							createQuestionEntry(tuple, index);
						})
						$.ajax({
							type: 'PUT',
							url: "/api/questions/answers/"+answers[0].id,
							contentType: "application/json",
							data: JSON.stringify(a1_data),
							success: function(resp) {
								answers[0] = resp;
								sema();
							}, 
						});
						$.ajax({
							type: 'PUT',
							url: "/api/questions/answers/"+answers[1].id,
							contentType: "application/json",
							data: JSON.stringify(a2_data),
							success: function(resp) {
								answers[1] = resp;
								sema();
							}, 
						});
					});
			}

			var id = "n"+nugget_index+"q"+index;
			var c_info = $("<div><label for=\""+id+"\">public</label></div>").appendTo(c_item);

			var checkbox = $('<input type="checkbox" id="'+id+'">').prependTo(c_info);
			if (question.published) {
				checkbox.prop("checked", true);
			};
			checkbox.change(function() {
				var request= { type: 'PUT', url: null };
				if (this.checked) {
					request.url = '/api/questions/'+question.id+'?publish';
				} else {
					request.url = '/api/questions/'+question.id+'?unpublish';
				};
				$.ajax(request);
			});


			var c_edit = $('<input type="button" value="show" class="linklike">').appendTo(c_info);

			var skill_label = $('<label class="bordered weak" style="display: inline-block">Skill:</label>').appendTo(c_info);
			var skill_level = $('<input type="number" class="compact" value="'+question.skill_level+'" class="linklike">').appendTo(skill_label);
			skill_level.change(function() {
				let updated_value = $(this).val();
				$.ajax({
					type: 'PUT',
					url: "/api/questions/"+question.id,
					contentType: "application/json",
					data: JSON.stringify({skill_level: updated_value}),
					success: function(resp) {
						question.skill_level = updated_value;
						console.log("Updated skill_level!");
					},
				});
			});

			answers.forEach(function(ans) {
				createBundle(ans.q_audio_bundle, c_info, function(updated_value, update_bundle_cb) {
					$.ajax({
						type: 'PUT',
						url: "/api/questions/answers/"+ans.id,
						contentType: "application/json",
						data: JSON.stringify({q_audio_bundle: updated_value}),
						success: function(resp) {
							ans.q_audio_bundle = updated_value;
							update_bundle_cb(updated_value);
							console.log("Updated audio bundle!");
						},
					});
				});
	
			});

			var c_body = $('<section class="bordered" style="margin-bottom: 3em;"></section>').appendTo(c_info).hide();
			var q_explanation = $('<p class="questionExplanation" contenteditable="true"></p>').appendTo(c_body).text(question.q_explanation);
			var question_latestResp = question;
			var q_explanation_okayToUpdate = false;
			q_explanation.on('blur', function() {
				question.q_explanation = question_latestResp.q_explanation;
				q_explanation_okayToUpdate = true;
				q_explanation.html(question.q_explanation);
			});
			q_explanation.on('input', function() {
				q_explanation_okayToUpdate = false;
				var request = {
					type: 'PUT',
					url: "/api/questions/"+question.id,
					contentType: "application/json",
					data: JSON.stringify({q_explanation: q_explanation.html()}),
					success: function(resp) {
						question_latestResp = resp;
						if (q_explanation_okayToUpdate) {
							question.q_explanation = question_latestResp.q_explanation;
							q_explanation.html(question.q_explanation);
						}
					},
				};
				$.ajax(request);
			});
			var question_text = $('<p class="questionText" contenteditable="true"></p>').appendTo(c_body).text(question.question_text);
			var question_text_okayToUpdate = false;
			question_text.on('blur', function() {
				question.question_text = question_latestResp.question_text;
				question_text_okayToUpdate = true;
				question_text.html(question.question_text);
			});
			question_text.on('input', function() {
				question_text_okayToUpdate = false;
				var request = {
					type: 'PUT',
					url: "/api/questions/"+question.id,
					contentType: "application/json",
					data: JSON.stringify({question_text: question_text.html()}),
					success: function(resp) {
						question_latestResp = resp;
						if (question_text_okayToUpdate) {
							question.question_text = question_latestResp.question_text;
							question_text.html(question.question_text);
						}
					},
				};
				$.ajax(request);
			});
			var a_list = $('<div class="answerList"></div>').appendTo(c_body);


			answers.forEach(function(ans) {
				var q_answer = $('<div class="answer bordered weak"></div>').appendTo(a_list);
				var q_bundle = $('<p></p>').appendTo(q_answer);
				createBundle(ans.q_audio_bundle, q_bundle, function(updated_value, update_bundle_cb) {
					$.ajax({
						type: 'PUT',
						url: "/api/questions/answers/"+ans.id,
						contentType: "application/json",
						data: JSON.stringify({q_audio_bundle: updated_value}),
						success: function(resp) {
							ans.q_audio_bundle = updated_value;
							update_bundle_cb(updated_value);
							console.log("Updated audio bundle!");
						},
					});
				});
				var qa_button = $('<div class="answerButton" contenteditable="true"></div>').appendTo(q_answer);
				qa_button.html(ans.answer_text);
				var answer_latestResp;
				var answer_text_okayToUpdate = false;
				qa_button.on('blur', function() {
					ans.answer_text = answer_latestResp.answer_text;
					answer_text_okayToUpdate = true;
					qa_button.html(ans.answer_text);
				});
				qa_button.on('input', function() {
					answer_text_okayToUpdate = false;
				var request = {
					type: 'PUT',
					url: "/api/questions/answers/"+ans.id,
					contentType: "application/json",
					data: JSON.stringify({answer_text: qa_button.html()}),
					success: function(resp) {
						answer_latestResp = resp;
						if (answer_text_okayToUpdate) {
							ans.answer_text = answer_latestResp.answer_text;
							qa_button.html(ans.answer_text);
						}
					},
				};
				$.ajax(request);
			});
			});

			function showBody() {
				c_edit.val("Hide").click(function() { c_body.hide(); c_edit.val("Show"); c_edit.click(showBody); });
				c_body.show();
			};

			c_edit.click(showBody);
		};
		questions.forEach(createQuestionEntry);

		function createExerciseEntry(tuple, index) {
			var exercise = tuple[0];
			var exercise_word_junction = tuple[1];

			let actual_words = new Array();
			words.forEach((w) => {
				if (w.id == exercise_word_junction[0].id || w.id == exercise_word_junction[1].id) {
					actual_words.push(w);
				}
			});

			let name = null;
			if (stripAccents(actual_words[0].word) == stripAccents(actual_words[1].word)) {
				name = stripAccents(actual_words[0].word);
			} else {
				name = nugget.skill_summary;
			}

			var c_item = $('<li style="width: 100%"></li>').appendTo(c_list);
			var c_header = $("<h3>Exercise ("+exercise.id+"): " + name + "</h3>").appendTo(c_item);
			var trash_button = proto_trash_button.clone()
				.appendTo(c_header)
				.click(function() {
					$.ajax({
						type: 'DELETE',
						url: "/api/exercises/"+exercise.id,
						contentType: "application/json",
						data: "",
						success: function() {
							c_item.remove();
							let removeIndex = exercises.indexOf(tuple);
							exercises.splice(removeIndex, 1);
							check_init_autocreate_buttons();
						}, 
					});
				});


			var id = "n"+nugget_index+"e"+index;
			var c_info = $("<div><label for=\""+id+"\">public</label></div>").appendTo(c_item);

			var checkbox = $('<input type="checkbox" id="'+id+'">').prependTo(c_info);
			if (exercise.published) {
				checkbox.prop("checked", true);
			};
			checkbox.change(function() {
				var request= { type: 'PUT', url: null };
				if (this.checked) {
					request.url = '/api/exercises/'+exercise.id+'?publish';
				} else {
					request.url = '/api/exercises/'+exercise.id+'?unpublish';
				};
				$.ajax(request);
			});

			var skill_label = $('<label class="bordered weak" style="display: inline-block">Skill:</label>').appendTo(c_info);
			var skill_level = $('<input type="number" class="compact" value="'+exercise.skill_level+'" class="linklike">').appendTo(skill_label);
			skill_level.change(function() {
				let updated_value = $(this).val();
				$.ajax({
					type: 'PUT',
					url: "/api/exercises/"+exercise.id,
					contentType: "application/json",
					data: JSON.stringify({skill_level: updated_value}),
					success: function(resp) {
						exercise.skill_level = updated_value;
						console.log("Updated skill_level!");
					},
				});
			});

		};
		exercises.forEach(createExerciseEntry);

		function check_init_autocreate_buttons() {
			n_item.find(".autocreate_q").remove();
			n_item.find(".autocreate_e").remove();

			let word_skill_levels = new Array();
			words.forEach((w) => {
				if ( word_skill_levels[w.skill_level] === undefined ) {
					word_skill_levels[w.skill_level] = new Array();
				}
				word_skill_levels[w.skill_level].push(w);
			});
	
			let q_skill_levels = new Array();
			questions.forEach((q_a) => {
				let q = q_a[0];
				let q_skill_level = Math.max(2, q.skill_level);
				if ( q_skill_levels[q_skill_level] === undefined ) {
					q_skill_levels[q_skill_level] = new Array();
				}
				q_skill_levels[q_skill_level].push(q);
			});
	
			let e_skill_levels = new Array();
			exercises.forEach((e_a) => {
				let e = e_a[0];
				let e_skill_level = Math.max(2, e.skill_level);
				if ( e_skill_levels[e_skill_level] === undefined ) {
					e_skill_levels[e_skill_level] = new Array();
				}
				e_skill_levels[e_skill_level].push(e);
			});
	
			word_skill_levels.forEach(function(words, skill_level) {
				let q_skill_level = Math.max(2, skill_level);
				let e_skill_level = Math.max(2, skill_level);

				if (words.length !== 2) {

					let pub_words = words.filter((w) => { return (w.published); });

					if (pub_words.length !== 2) {
						console.log("There isn't two words at skill level "+skill_level+" of skill "+nugget.skill_summary+", but ", words.length);
						return;
					}
					words = pub_words;
				}

				let name = null;
				if (stripAccents(words[0].word) == stripAccents(words[1].word)) {
					name = stripAccents(words[0].word);
				} else {
					name = nugget.skill_summary;
				}

				if (q_skill_levels[q_skill_level] === undefined) {
	
					let c_item = $('<li class="autocreate_q"></li>').appendTo(c_list);
					let c_body = $('<div></div>');
					c_body.appendTo(c_item);
					let c_button = $('<input type="button" value="Autocreate question '+name+', skill '+q_skill_level+'" class="linklike">');
					c_button.appendTo(c_body);

					c_button.click(function() {
						let question_data = [{
									q_name: name,
									q_explanation: "Kuuntele ja vastaa kysymykseen",
									question_text: "Mistä asiasta on kyse?",
									skill_id: nugget.id,
									published: false,
									skill_level: q_skill_level,
									},
									[{
										question_id: 0,
										a_audio_bundle: null,
										q_audio_bundle: words[0].audio_bundle,
										answer_text: words[0].explanation,
									},
									{
										question_id: 0,
										a_audio_bundle: null,
										q_audio_bundle: words[1].audio_bundle,
										answer_text: words[1].explanation,
									}]];
						$.ajax({
							url: "/api/questions",
							contentType: "application/json",
							type: "POST",
							data: JSON.stringify(question_data),
							success: function(resp) {
								c_item.remove();
								questions.push(resp);
								createQuestionEntry(resp, questions.length);
							},
						});
					});
				}

				if (e_skill_levels[e_skill_level] === undefined) {

					let c_item = $('<li class="autocreate_e"></li>').appendTo(c_list);
					let c_body = $('<div></div>');
					c_body.appendTo(c_item);
					let c_button = $('<input type="button" value="autocreate exercise '+name+', skill '+e_skill_level+'" class="linklike">');
					c_button.appendTo(c_body);
					c_button.click(function() {
						let exercise_data = [{
									skill_id: nugget.id,
									skill_level: e_skill_level,
									},
									[{
										exercise_id: 0,
										id: words[0].id,
									},
									{
										exercise_id: 0,
										id: words[1].id,
									}]];
						$.ajax({
							url: "/api/exercises",
							contentType: "application/json",
							type: "POST",
							data: JSON.stringify(exercise_data),
							success: function(resp) {
								c_item.remove();
								exercises.push(resp);
								createExerciseEntry(resp, exercises.length);
							},
						});
					});
				}
			});
		}

		check_init_autocreate_buttons();
		
	}; // function drawNugget ends
}; // function drawList ends

var loading_msg = $('#loadingMsg');

function tryDraw() {
	if (nugget_resp !== null && bundle_resp !== null && narrator_resp !== null) {
		$("#filteredAmount").hide();
		loading_msg.text("Loaded. Rendering content. ");
		setTimeout(() => { drawList(nugget_resp, bundle_resp, narrator_resp); }, 0);
	}
}

$.getJSON("/api/bundles", function(resp) {
	loading_msg.append("Bundles loaded. ");
	bundle_resp = resp;
	setTimeout(tryDraw, 0);
	
});

$.getJSON("/api/narrators", function(resp) {
	loading_msg.append("Narrators loaded. ");
	narrator_resp = resp;
	setTimeout(tryDraw, 0);
});

$.getJSON("/api/nuggets", function(resp) {
	loading_msg.append("Words loaded. ");
	nugget_resp = resp;
	setTimeout(tryDraw, 0);
});

});
