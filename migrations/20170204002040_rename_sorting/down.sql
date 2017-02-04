UPDATE user_groups SET group_name = 'ready_for_sorting' WHERE group_name = 'sort';
UPDATE user_groups SET group_name = 'input_group' WHERE group_name = 'questions';
UPDATE user_groups SET group_name = 'output_group' WHERE group_name = 'exercises';
