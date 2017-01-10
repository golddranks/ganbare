DELETE FROM event_experiences WHERE event_id = (SELECT id FROM events WHERE name='pretest_retelling');
DELETE FROM event_experiences WHERE event_id = (SELECT id FROM events WHERE name='posttest_retelling');
DELETE FROM events WHERE name = 'pretest_retelling';
DELETE FROM events WHERE name = 'posttest_retelling';
