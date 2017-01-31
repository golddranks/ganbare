DELETE FROM event_experiences WHERE event_id=(SELECT id FROM events WHERE name='training');
DELETE FROM event_userdata WHERE event_id=(SELECT id FROM events WHERE name='training');
DELETE FROM events WHERE name='training';
