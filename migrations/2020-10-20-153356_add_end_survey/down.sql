DELETE FROM events where name = 'end_survey';
UPDATE events SET priority=9 WHERE id=(SELECT id FROM events WHERE name='posttest_info');
UPDATE events SET priority=10 WHERE id=(SELECT id FROM events WHERE name='posttest');
UPDATE events SET priority=11 WHERE id=(SELECT id FROM events WHERE name='posttest_retelling');
UPDATE events SET priority=12 WHERE id=(SELECT id FROM events WHERE name='posttest_done');
