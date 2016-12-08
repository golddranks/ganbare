/// <reference path="typings/globals/jquery/index.d.ts" />

$(function() {

	var groupHeader = $("#groupHeader");
	var usersList = $("#usersList");
	var pendingUsersList = $("#pendingUsersList");
	
	$.getJSON("/api/users", function(resp){
		var groups = resp[0];
		var users = resp[1];
		var pending_users = resp[2];

		groups.forEach(function(group) {
			var group_header = $('<th scope="col">'+group.group_name+'</th>').appendTo(groupHeader);
		});

		users.forEach(function(u) {
			var user = u[0];
			var group_memberships = u[1];

			var user_list_tr = $('<tr></tr>').appendTo(usersList);
			$('<th scope="row">'+user.email+'</th>').appendTo(user_list_tr);

			var user_groups = new Array();
			groups.forEach(function(group) {
				user_groups[group.id] = false;
			});

			group_memberships.forEach(function(group_membership) {
				user_groups[group_membership.group_id] = true;
			});

			user_groups.forEach(function(isMember, index) {
				var cell = $('<td></td>').appendTo(user_list_tr);
				var id = 'u'+user.id+'g'+index;
				var checkbox = $('<input type="checkbox" id="'+id+'">').appendTo(cell);
				var label = $('<label for="'+id+'"></label>').appendTo(cell);
				if (isMember) {
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
		});

		if (pending_users.length === 0) {
			var user_list = $('<li>(none)</li>').appendTo(pendingUsersList);
		}

		pending_users.forEach(function(pending_user) {
			var user_list = $('<li></li>').appendTo(pendingUsersList);
			user_list.text(pending_user.email);
		});
	});

})
