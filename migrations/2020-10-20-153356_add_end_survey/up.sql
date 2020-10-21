UPDATE events SET priority=10 WHERE id=(SELECT id FROM events WHERE name='posttest_info');
UPDATE events SET priority=11 WHERE id=(SELECT id FROM events WHERE name='posttest');
UPDATE events SET priority=12 WHERE id=(SELECT id FROM events WHERE name='posttest_retelling');
UPDATE events SET priority=13 WHERE id=(SELECT id FROM events WHERE name='posttest_done');
INSERT INTO events (name, published, priority, required_group) VALUES ('end_survey', false, 9, (SELECT id FROM user_groups WHERE group_name='posttest'));
