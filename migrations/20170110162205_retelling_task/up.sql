UPDATE events SET priority = 1 WHERE name = 'survey';
UPDATE events SET priority = 2 WHERE name = 'pretest';
INSERT INTO events (name, published, required_group, priority) VALUES ('pretest_retelling', false, (SELECT id FROM user_groups WHERE group_name='subjects'), 3);
UPDATE events SET priority = 4 WHERE name = 'sorting_ceremony';
UPDATE events SET priority = 5 WHERE name = 'posttest';
INSERT INTO events (name, published, required_group, priority) VALUES ('posttest_retelling', false, (SELECT id FROM user_groups WHERE group_name='subjects'), 6);
UPDATE events SET published = true WHERE name = 'pretest';
