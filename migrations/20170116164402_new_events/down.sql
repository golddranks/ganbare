DELETE FROM event_experiences WHERE event_id=(SELECT id FROM events WHERE name='agreement');
DELETE FROM event_experiences WHERE event_id=(SELECT id FROM events WHERE name='info');
DELETE FROM event_experiences WHERE event_id=(SELECT id FROM events WHERE name='pretest_info');
DELETE FROM event_experiences WHERE event_id=(SELECT id FROM events WHERE name='pretest_done');
DELETE FROM event_experiences WHERE event_id=(SELECT id FROM events WHERE name='posttest_info');
DELETE FROM event_experiences WHERE event_id=(SELECT id FROM events WHERE name='posttest_done');
DELETE FROM event_userdata WHERE event_id=(SELECT id FROM events WHERE name='agreement');
DELETE FROM event_userdata WHERE event_id=(SELECT id FROM events WHERE name='info');
DELETE FROM event_userdata WHERE event_id=(SELECT id FROM events WHERE name='pretest_info');
DELETE FROM event_userdata WHERE event_id=(SELECT id FROM events WHERE name='pretest_done');
DELETE FROM event_userdata WHERE event_id=(SELECT id FROM events WHERE name='posttest_info');
DELETE FROM event_userdata WHERE event_id=(SELECT id FROM events WHERE name='posttest_done');
DELETE FROM events WHERE name='agreement';
DELETE FROM events WHERE name='info';
DELETE FROM events WHERE name='pretest_info';
DELETE FROM events WHERE name='pretest_done';
DELETE FROM events WHERE name='posttest_info';
DELETE FROM events WHERE name='posttest_done';