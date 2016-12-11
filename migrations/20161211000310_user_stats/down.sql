DROP TABLE user_stats;
ALTER TABLE q_answered_data DROP COLUMN full_spent_time_ms;
ALTER TABLE e_answered_data DROP COLUMN full_spent_time_ms;
ALTER TABLE e_answered_data DROP COLUMN reflected_time_ms;
UPDATE users SET email = 'invalid' WHERE email IS NULL;
ALTER TABLE users ALTER COLUMN email SET NOT NULL;
