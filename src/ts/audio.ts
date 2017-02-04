/// <reference path="typings/globals/jquery/index.d.ts" />
/// <reference path="typings/globals/howler/index.d.ts" />

$(function() {

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


var narratorColumns = $("#narratorColumns");
var body = $("body");
var bundlesList = $("#bundlesList");
var narrators = [];
var bundles = [];
var globalNarratorMergingState = -1;
var globalBundleMergingState = -1;
let semaBothLoaded = createSemaphore(2);

$.getJSON("/api/narrators", function(resp){
	resp.forEach(function(narrator) { // These might be in any order
		narrators[narrator.id] = narrator;
	});
	narrators.forEach(function(narrator) {
		var narr_header = $('<th class="narratorHeaders narrator'+narrator.id+'"></th>').appendTo(narratorColumns);
		function initCell(editCell_cb) {
			narr_header.html('');
			var narr_vertical = $('<div class="vertical" scope="col"></div>').appendTo(narr_header);
			var published_button = $('<input type="checkbox" name="'+narrator.id+'_published" id="'+narrator.id+'_published"'+(narrator.published?' checked':'')+'>')
				.appendTo(narr_vertical);
			$('<label for="'+narrator.id+'_published"></label>')
				.appendTo(narr_vertical);
			var trash_button = $('<button class="compact narrDelButton"><i class="fa fa-trash" aria-hidden="true"></i></button>')
				.appendTo(narr_vertical);
			var merge_button = $('<button class="compact narrMergeButton"><i class="fa fa-compress" aria-hidden="true"></i></button>')
				.appendTo(narr_vertical);
			var narr_name = $('<span class="narrName">'+narrator.id+' '+narrator.name+'</span>')
				.appendTo(narr_vertical);
			narr_name.off('click').one('click', editCell_cb);
			published_button.change(function() {
				var json_data = JSON.stringify({id: narrator.id, name: narrator.name, published: (published_button.is(':checked'))?true:false});
				var request = {
					type: 'PUT',
					url: "/api/narrators/"+narrator.id,
					contentType: "application/json",
					data: json_data,
					success: function(resp) {
						narrator = resp;
					}, 
				};
				$.ajax(request);
			});
			trash_button.click(function() {
					var request = {
						type: 'DELETE',
						url: "/api/narrators/"+narrator.id,
						contentType: "application/json",
						data: "",
						success: function() {
							delete narrators[narrator.id];
							$(".narrator"+narrator.id).remove();
						}, 
					};
					$.ajax(request);
			});
			merge_button.click(function(ev) {
				ev.stopPropagation();
				if (globalNarratorMergingState === -1) {
					globalNarratorMergingState = narrator.id;
					body.one('click', function() {
						$(".narrMergeButton").removeClass("hilight");
						globalNarratorMergingState = -1;
					});
					merge_button.remove();
					$(".narrMergeButton").addClass("hilight");
				} else {
					$(".narrMergeButton").removeClass("hilight");
					var closureMergingState = globalNarratorMergingState;
					var request = {
						type: 'DELETE',
						url: "/api/narrators/"+globalNarratorMergingState+"?merge_with="+narrator.id,
						contentType: "application/json",
						data: "",
						success: function() {
							location.reload();
						}, 
					};
					$.ajax(request);
					globalNarratorMergingState = -1;
				}
			});
		}
		function editCell(ev) {
			var inputName = $('<input type="text" value="'+narrator.name+'">');
			narr_header.html('').append(inputName);
			inputName.focus();
			inputName.blur(function() {
				var request = {
					type: 'PUT',
					url: "/api/narrators/"+narrator.id,
					contentType: "application/json",
					data: JSON.stringify({id: narrator.id, name: inputName.val(), published: narrator.published}),
					success: function(resp) {
						narrator = resp;
						initCell(editCell);
					}, 
				};
				$.ajax(request);
			});
		}
		initCell(editCell);
	});
	semaBothLoaded();
});
	

$.getJSON("/api/bundles", function(resp) {
	bundles = resp;
	semaBothLoaded(function() {
	
		let bundle_index = 0;
		function drawBundleAsync() {
			for (let i = 0; i < 5; i++) {
				let tuple = bundles[bundle_index];
	
				if (tuple === undefined) {
					return; // Nothing left to render;
				}
	
				drawBundle(tuple);
				bundle_index += 1;
			}
			setTimeout(drawBundleAsync, 0);
		}
		drawBundleAsync();
	
	});
});


let currentlyDragged = null;

function dropHandler(event) {
    event.preventDefault();  
    event.stopPropagation();
    $(this).removeClass("dropTarget");
    console.log("drop:", event);
    this.append(currentlyDragged);
    let file = $(currentlyDragged).data("file");
    currentlyDragged = null;
    file.bundle_id = $(this).data("bundle_id");
    file.narrators_id = $(this).data("narrator_id");
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
	this.style.opacity = '0.4';
	currentlyDragged = this;
	setTimeout(() => {$(this).parent().addClass("dropTarget")}, 0);
}
function dragEndHandler(event) {
	this.style.opacity = '1.0';
}



let proto_speaker_button = $('<button class="compact" title="ID: undefined" draggable="true"><img src="/static/images/speaker_teal.png" draggable="false" class="soundicon"></button><br>');

function drawBundle(tuple) {
	var bundle = tuple[0];
	var files = tuple[1];
	var bundleRow = $('<tr></tr>').appendTo(bundlesList);
	var bundleCell = $('<th scope="row"></th>').appendTo(bundleRow);
	function initCell() {
		bundleCell.html(bundle.id+' '+accentuate(bundle.listname));
		var trash_button = $('<button class="compact"><i class="fa fa-trash" aria-hidden="true"></i></button>').appendTo(bundleCell);
		var merge_button = $('<button class="compact bundleMergeButton"><i class="fa fa-compress" aria-hidden="true"></i></button>').appendTo(bundleCell);
		trash_button.click(function() {
				var url = "/api/bundles/"+bundle.id;
				var request = {
					type: 'DELETE',
					url: url,
					contentType: "application/json",
					data: "",
					success: function() {
						bundleRow.remove();
					}, 
				};
				$.ajax(request);
		});
		merge_button.click(function(ev) {
			ev.stopPropagation();
			if (globalBundleMergingState === -1) {
				globalBundleMergingState = bundle.id;
				body.one('click', function() {
					$(".bundleMergeButton").removeClass("hilight");
					globalBundleMergingState = -1;
				});
				merge_button.remove();
				$(".bundleMergeButton").addClass("hilight");
			} else {
				$(".bundleMergeButton").removeClass("hilight");
				var closureMergingState = globalBundleMergingState;
				var request = {
					type: 'DELETE',
					url: "/api/bundles/"+globalBundleMergingState+"?merge_with="+bundle.id,
					contentType: "application/json",
					data: "",
					success: function() {
						location.reload();
					}, 
				};
				$.ajax(request);
				globalBundleMergingState = -1;
			}
		});
	}
	initCell();
	function editCell(ev) {
		ev.stopPropagation();
		var inputName = $('<input type="text" value="'+bundle.listname+'">');
		bundleCell.html('').append(inputName);
		inputName.click(function(inClickEvent) {
			inClickEvent.stopPropagation();
		});
		body.one('click', function() {
			var request = {
				type: 'PUT',
				url: "/api/bundles/"+bundle.id,
				contentType: "application/json",
				data: JSON.stringify({id: bundle.id, listname: inputName.val()}),
				success: function(resp) {
					bundle = resp;
					initCell();
					bundleCell.off('click').one('click', editCell);
				}, 
			};
			$.ajax(request);
		});
	}
	bundleCell.one('click', editCell);
	var narr_files = new Array();
	narrators.forEach(function(narrator) {
		narr_files[narrator.id] = [];
	});
	files.forEach(function (f) {
		narr_files[f.narrators_id].push(f);
	});

	narrators.forEach(function(narr) {
		var cell = $('<td class="narrator'+narr.id+'"></td>').appendTo(bundleRow);

		cell.on("drop", dropHandler);
		cell.on("dragenter", dragEnterHandler);
		cell.on("dragleave", dragLeaveHandler);
		cell.on("dragover", dragOverHandler);
		cell.data("bundle_id", bundle.id);
		cell.data("narrator_id", narr.id);

		narr_files[narr.id].forEach(function(f) {
			var speaker_button = proto_speaker_button.clone().appendTo(cell);
			speaker_button.prop('title', "ID: "+f.id);
			speaker_button.on('dragstart', dragStartHandler);
			speaker_button.on('dragend', dragEndHandler);
			speaker_button.data("file", f);
			speaker_button.click(function () {
				var audio = new Howl({ src: ['/api/audio/'+f.id+'.mp3']});
				audio.play();
			});
		});
	});
}

});
