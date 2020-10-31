INSERT INTO user_groups (group_name) VALUES ('simple_test');
INSERT INTO user_groups (group_name) VALUES ('pretest');
INSERT INTO user_groups (group_name) VALUES ('romaji');
INSERT INTO user_groups (group_name) VALUES ('training');

INSERT INTO group_memberships
WITH pretest_group AS (SELECT * FROM user_groups WHERE group_name='pretest'),
	subjects_group AS (SELECT * FROM user_groups WHERE group_name='subjects')
SELECT user_id, pretest_group.id AS group_id FROM group_memberships CROSS JOIN pretest_group CROSS JOIN subjects_group WHERE group_id=subjects_group.id;

INSERT INTO group_memberships
WITH training_group AS (SELECT * FROM user_groups WHERE group_name='training')
SELECT users.id as user_id, training_group.id AS group_id FROM users CROSS JOIN training_group;

WITH training_group AS (SELECT * FROM user_groups WHERE group_name='training')
UPDATE events SET required_group = training_group.id FROM training_group WHERE name = 'training';

INSERT INTO events (name, published, priority, required_group) VALUES ('welcome2', true, 0, (SELECT id FROM user_groups WHERE group_name='simple_test'));
INSERT INTO events (name, published, priority, required_group) VALUES ('mini_survey', false, 2, (SELECT id FROM user_groups WHERE group_name='simple_test'));
INSERT INTO events (name, published, priority, required_group) VALUES ('simple_test_info', false, 14, (SELECT id FROM user_groups WHERE group_name='simple_test'));
INSERT INTO events (name, published, priority, required_group) VALUES ('simple_test', false, 15, (SELECT id FROM user_groups WHERE group_name='simple_test'));
INSERT INTO events (name, published, priority, required_group) VALUES ('simple_test_done', false, 16, (SELECT id FROM user_groups WHERE group_name='simple_test'));
INSERT INTO events (name, published, priority) VALUES ('over', true, 100);


WITH pretest_group AS (SELECT * FROM user_groups WHERE group_name='pretest')
UPDATE events SET required_group = pretest_group.id FROM pretest_group WHERE name IN ('pretest_info', 'pretest', 'pretest_retelling', 'pretest_done');

WITH pretest_group AS (SELECT * FROM user_groups WHERE group_name='pretest')
UPDATE events SET required_group = pretest_group.id FROM pretest_group WHERE name IN ('welcome');
