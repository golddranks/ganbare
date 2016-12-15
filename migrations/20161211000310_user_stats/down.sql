DROP TABLE reset_email_secrets;
DROP TABLE user_stats;
ALTER TABLE q_answered_data DROP COLUMN full_spent_time_ms;
ALTER TABLE e_answered_data DROP COLUMN full_spent_time_ms;
ALTER TABLE e_answered_data DROP COLUMN reflected_time_ms;
UPDATE users SET email = random_id FROM (SELECT id, ('invalid' || md5(random()::text)) as random_id FROM users) as randoms WHERE email IS NULL and randoms.id=users.id;
ALTER TABLE users ALTER COLUMN email SET NOT NULL;

ALTER TABLE w_answered_data RENAME COLUMN full_spent_time_ms TO answer_time_ms;
ALTER TABLE w_answered_data DROP COLUMN active_answer_time_ms;
DELETE FROM group_memberships USING user_groups WHERE id=group_id AND group_name='show_accents';
DELETE FROM user_groups WHERE group_name='show_accents';
DELETE FROM group_memberships USING user_groups WHERE id=group_id AND group_name='nag_emails';
DELETE FROM user_groups WHERE group_name='nag_emails';
