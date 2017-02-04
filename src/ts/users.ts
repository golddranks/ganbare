/// <reference path="typings/globals/jquery/index.d.ts" />

$(function() {

	$("#show_anon").change(function() {
		groupHeader.html("<th>Users</th>");
		usersList.html("");
		pendingUsersList.html("");
		userActivityRows.html("");
		showUserTable(usersResp, $(this).is(":checked"));
	});

	var groupHeader = $("#groupHeader");
	var usersList = $("#usersList");
	var pendingUsersList = $("#pendingUsersList");
	var userActivityRows = $("#userActivityRows");

	function showUserTable(resp, showAnon: boolean) {
		var users = resp[0];
		var groups = resp[1];
		var pending_users = resp[2];

		groups.forEach(function(group) {
			var group_header = $('<th scope="col">'+group.group_name+'</th>').appendTo(groupHeader);
		});

		users.forEach(function(u) {
			var user = u[0];
			if (user.email === null) {
				return;
			}
			var user_metrics = u[1];
			var user_stats = u[2];
			var group_memberships = u[3];
			var overdues = u[4];
			var sessions = u[5];

			var user_list_tr = $('<tr></tr>').appendTo(usersList);
			$('<th scope="row">'+user.email+'</th>').appendTo(user_list_tr);

			var user_groups = new Array();
			groups.forEach(function(group) {
				user_groups[group.id] = false;
				if (group.anonymous && !showAnon) {
					user_groups[group.id] = null;
				}
			});

			group_memberships.forEach(function(group_membership) {
				if((showAnon || !group_membership.anonymous) && user_groups[group_membership.group_id] === false) {
					user_groups[group_membership.group_id] = true;
				}
			});

			user_groups.forEach(function(isMember, index) {
				var cell = $('<td></td>').appendTo(user_list_tr);
				var id = 'u'+user.id+'g'+index;
				var checkbox = $('<input type="checkbox" id="'+id+'">').appendTo(cell);
				var label = $('<label for="'+id+'"></label>').appendTo(cell);
				if (isMember === null) {
					checkbox.prop('disabled', true);
				} else if (isMember) {
					checkbox.prop('checked', 'true');
				}
				checkbox.change(function() {
					var url;
					if ($(this).prop('checked') === true) {
						url = "/api/users/"+user.id+"?add_group="+index;
					} else {
						url = "/api/users/"+user.id+"?remove_group="+index;
					}

					var request = {
						type: 'PUT',
						url: url,
						contentType: "application/json",
						data: "",
					};
					$.ajax(request);

				});
			});

			var user_item = $('<tr></tr>').appendTo(userActivityRows);
			var user_email_cell = $('<th>'+user.email+'</th>').appendTo(user_item);
			$('<button class="compact narrDelButton"><i class="fa fa-trash" aria-hidden="true"></i></button>')
				.appendTo(user_email_cell)
				.click(function() {
					var request = {
						type: 'DELETE',
						url: "/api/users/"+user.id,
						contentType: "application/json",
						data: "",
						success: function() {
							user_item.remove();
							user_list_tr.remove();
						}, 
					};
					$.ajax(request);
				});
			$('<button class="compact narrDelButton"><i class="fa fa-eraser" aria-hidden="true"></i></button>')
				.appendTo(user_email_cell)
				.click(function() {
					var request = {
						type: 'DELETE',
						url: "/api/users/"+user.id+'/due_and_pending_items',
						contentType: "application/json",
						data: "",
						success: function() {
							alert("User due and pending items removed!");
						}, 
					};
					$.ajax(request);
				});
			$('<td>'+user_stats.days_used+'</td>').appendTo(user_item);
			$('<td>'+Math.floor(Math.round(user_stats.all_active_time_ms/1000)/60)+' min '+Math.round(user_stats.all_active_time_ms/1000)%60+' s</td>').appendTo(user_item);
			$('<td>'+user_stats.all_words+'</td>').appendTo(user_item);
			$('<td>'+user_stats.quiz_all_times+'</td>').appendTo(user_item);
			$('<td>'+Math.round(user_stats.quiz_correct_times/user_stats.quiz_all_times*100)+' %</td>').appendTo(user_item);
			$('<td>'+overdues+'</td>').appendTo(user_item);

			function put_user_settings(type, settings, success_fn) {
					var url = "/api/users/"+user.id+"?settings="+type;
					settings.id = user.id;
					var request = {
						type: 'PUT',
						url: url,
						contentType: "application/json",
						data: JSON.stringify(settings),
						success: function() { console.log("Success!"); success_fn() }, 
					};
					$.ajax(request);
			}

			$('<td></td>').text(user_metrics.new_words_since_break+'/'+user_metrics.max_words_since_break).appendTo(user_item)
				.click(function(ev) {
					ev.stopPropagation();
					var cell = $(this);
					$("body").click(function(ev) {
						cell.text(user_metrics.new_words_since_break+'/'+user_metrics.max_words_since_break);
					})
					cell.empty().append(
						$('<input style="width: 5em;" type="text" value="'+user_metrics.max_words_since_break+'">')
							.change(function(){
								var new_val = $(this).val();
								put_user_settings("metrics", {max_words_since_break: new_val}, function() {
									user_metrics.max_words_since_break = new_val;
									cell.text(user_metrics.new_words_since_break+'/'+user_metrics.max_words_since_break);
								});
							})
							.click(function(ev) {
								ev.stopPropagation();
							})
					);
				});

			$('<td></td>').text(user_metrics.new_words_today+'/'+user_metrics.max_words_today).appendTo(user_item)
				.click(function(ev) {
					ev.stopPropagation();
					var cell = $(this);
					$("body").click(function(ev) {
						cell.text(user_metrics.new_words_today+'/'+user_metrics.max_words_today);
					})
					cell.empty().append(
						$('<input style="width: 5em;" type="text" value="'+user_metrics.max_words_today+'">')
							.change(function(){
								var new_val = $(this).val();
								put_user_settings("metrics", {max_words_today: new_val}, function() {
									user_metrics.max_words_today = new_val;
									cell.text(user_metrics.new_words_today+'/'+user_metrics.max_words_today);
								});
							})
							.click(function(ev) {
								ev.stopPropagation();
							})
					);
				});

			$('<td></td>').text(user_metrics.quizes_since_break+'/'+user_metrics.max_quizes_since_break).appendTo(user_item)
				.click(function(ev) {
					ev.stopPropagation();
					var cell = $(this);
					$("body").click(function(ev) {
						cell.text(user_metrics.quizes_since_break+'/'+user_metrics.max_quizes_since_break);
					})
					cell.empty().append(
						$('<input style="width: 5em;" type="text" value="'+user_metrics.max_quizes_since_break+'">')
							.change(function(){
								var new_val = $(this).val();
								put_user_settings("metrics", {max_quizes_since_break: new_val}, function() {
									user_metrics.max_quizes_since_break = new_val;
									cell.text(user_metrics.quizes_since_break+'/'+user_metrics.max_quizes_since_break);
								});
							})
							.click(function(ev) {
								ev.stopPropagation();
							})
					);
				});
			
			$('<td></td>').text(user_metrics.quizes_today+'/'+user_metrics.max_quizes_today).appendTo(user_item)
				.click(function(ev) {
					ev.stopPropagation();
					var cell = $(this);
					$("body").click(function(ev) {
						cell.text(user_metrics.quizes_today+'/'+user_metrics.max_quizes_today);
					})
					cell.empty().append(
						$('<input style="width: 5em;" type="text" value="'+user_metrics.max_quizes_today+'">')
							.change(function(){
								var new_val = $(this).val();
								put_user_settings("metrics", {max_quizes_today: new_val}, function() {
									user_metrics.max_quizes_today = new_val;
									cell.text(user_metrics.quizes_today+'/'+user_metrics.max_quizes_today);
								});
							})
							.click(function(ev) {
								ev.stopPropagation();
							})
					);
				});
			var break_button = $('<td></td>').appendTo(user_item);

			if (new Date(user_metrics.break_until) > new Date()) {
				$('<button>stop</button>')
					.appendTo(break_button)
					.click(function(){
						user_metrics.break_until = new Date().toISOString();
						put_user_settings("metrics", {break_until: user_metrics.break_until}, function() {
							location.reload();
						});
					});
			} else {
				break_button.text("no");
			}
			var reset_pw = $('<td></td>').appendTo(user_item);
			var reset_pw_button = $('<button>reset</button>')
				.appendTo(reset_pw)
				.click(function() {
					$.post("/send_password_reset_email", {email: user.email}, function() {
						alert("Done!");
					});
				});
			let last_seen_time = sessions[0].split("T");
			var last_seen = $('<td>'+last_seen_time[0]+'<br>'+last_seen_time[1].slice(0,8)+'</td>').appendTo(user_item);
		});

		if (pending_users.length === 0) {
			var user_list = $('<li>(none)</li>').appendTo(pendingUsersList);
		}

		pending_users.forEach(function(pending_user) {
			var user_list = $('<li></li>').appendTo(pendingUsersList);
			user_list.text(pending_user.email);
		});
	}

	let usersResp;
	
	$.getJSON("/api/users", (resp) => {
		usersResp = resp;
		showUserTable(resp, false);
	});

})
