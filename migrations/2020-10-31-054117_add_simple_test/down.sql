WITH subjects_group AS (SELECT * FROM user_groups WHERE group_name='subjects')
UPDATE events SET required_group = subjects_group.id FROM subjects_group WHERE name IN ('pretest_info', 'pretest', 'pretest_retelling', 'pretest_done');
UPDATE events SET required_group = NULL WHERE name = 'training';
UPDATE events SET required_group = NULL WHERE name = 'welcome';

DELETE FROM event_userdata WHERE event_id in (SELECT id FROM events WHERE name IN ('simple_test', 'simple_test_info', 'simple_test_done', 'mini_survey', 'welcome2', 'over'));
DELETE FROM event_experiences WHERE event_id in (SELECT id FROM events WHERE name IN ('simple_test', 'simple_test_info', 'simple_test_done', 'mini_survey', 'welcome2', 'over'));
DELETE FROM events WHERE name IN ('simple_test', 'simple_test_info', 'simple_test_done', 'mini_survey', 'welcome2', 'over');
DELETE FROM group_memberships WHERE group_id IN (SELECT id FROM user_groups WHERE group_name IN ('pretest', 'simple_test', 'training', 'romaji'));
DELETE FROM user_groups WHERE group_name IN ('simple_test', 'pretest', 'romaji', 'training');
