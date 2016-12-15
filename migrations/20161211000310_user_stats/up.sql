
ALTER TABLE w_answered_data RENAME COLUMN answer_time_ms TO full_spent_time_ms;

ALTER TABLE w_answered_data ADD COLUMN active_answer_time_ms INTEGER NOT NULL DEFAULT 0;

INSERT INTO user_groups (group_name) VALUES ('show_accents');
INSERT INTO user_groups (group_name) VALUES ('nag_emails');

CREATE TABLE user_stats (
	id SERIAL PRIMARY KEY REFERENCES users,
	days_used INTEGER NOT NULL DEFAULT 0,
	all_active_time_ms BIGINT NOT NULL DEFAULT 0,
	all_spent_time_ms BIGINT NOT NULL DEFAULT 0,
	all_words INTEGER NOT NULL DEFAULT 0,
	quiz_all_times INTEGER NOT NULL DEFAULT 0,
	quiz_correct_times INTEGER NOT NULL DEFAULT 0,
	last_nag_email TIMESTAMPTZ
);

INSERT INTO user_stats (id, all_spent_time_ms, all_active_time_ms, days_used, all_words, quiz_all_times, quiz_correct_times) (
	SELECT
		id,
		COALESCE(all_spent_time_ms, 0) AS all_spent_time_ms,
		COALESCE(all_active_time_ms, 0) AS all_active_time_ms,
		COALESCE(days_used, 0) AS days_used,
		COALESCE(all_words, 0) AS all_words,
		COALESCE(quiz_all_times, 0) AS quiz_all_times,
		(COALESCE(e_correct_times, 0) + COALESCE(q_correct_times, 0)) AS quiz_correct_times
	FROM
		users
		LEFT JOIN (
			SELECT	user_id,
					(
						COALESCE(SUM(q_ans.full_answer_time_ms), 0) +
						COALESCE(SUM(e_ans.full_answer_time_ms), 0) +
						COALESCE(SUM(w_ans.full_spent_time_ms), 0)
					) AS all_spent_time_ms
				FROM pending_items AS items
				LEFT JOIN q_answered_data AS q_ans ON q_ans.id=items.id
				LEFT JOIN e_answered_data AS e_ans ON e_ans.id=items.id
				LEFT JOIN w_answered_data AS w_ans ON w_ans.id=items.id
				GROUP BY user_id
			) AS all_spent_time ON all_spent_time.user_id=id
		LEFT JOIN (
			SELECT	user_id,
					(
						COALESCE(SUM(q_ans.active_answer_time_ms), 0) +
						COALESCE(SUM(e_ans.active_answer_time_ms), 0) +
						COALESCE(SUM(w_ans.active_answer_time_ms), 0)
					) AS all_active_time_ms
				FROM pending_items AS items
				LEFT JOIN q_answered_data AS q_ans ON q_ans.id=items.id
				LEFT JOIN e_answered_data AS e_ans ON e_ans.id=items.id
				LEFT JOIN w_answered_data AS w_ans ON w_ans.id=items.id
				GROUP BY user_id
			) AS all_active_time ON all_active_time.user_id=id
		LEFT JOIN (
			SELECT	user_id, COUNT(DISTINCT a_date) AS days_used
				FROM pending_items AS items
				LEFT JOIN (
							SELECT id, answered_date::date AS a_date FROM q_answered_data
							UNION
							SELECT id, answered_date::date AS a_date FROM e_answered_data
							UNION
							SELECT id, checked_date::date AS a_date FROM w_answered_data
						) AS dates ON dates.id=items.id
				WHERE pending=false
				GROUP BY user_id
			) AS all_days ON all_days.user_id=id
		LEFT JOIN (
			SELECT user_id, COUNT(*) AS all_words
				FROM pending_items
				WHERE item_type='word'
				GROUP BY user_id
			) AS words ON words.user_id=id
		LEFT JOIN (
			SELECT user_id, COUNT(*) AS quiz_all_times
				FROM pending_items
				WHERE item_type='question' OR item_type='exercise'
				GROUP BY user_id
			) AS quizes ON quizes.user_id=id
		LEFT JOIN (
			SELECT user_id, COUNT(*) AS q_correct_times
				FROM pending_items AS items
				JOIN q_answered_data AS ans ON ans.id=items.id
				JOIN q_asked_data AS ask ON ask.id=items.id
				WHERE ans.answered_qa_id=ask.correct_qa_id
				GROUP BY user_id
			) AS q_correct ON q_correct.user_id=id
		LEFT JOIN (
			SELECT user_id, COUNT(*) AS e_correct_times
				FROM pending_items AS items
				JOIN e_answered_data AS ans ON ans.id=items.id
				WHERE answer_level>0
				GROUP BY user_id
			) AS e_correct ON e_correct.user_id=id
);


ALTER TABLE q_answered_data ADD COLUMN full_spent_time_ms INTEGER NOT NULL DEFAULT 0;

ALTER TABLE e_answered_data ADD COLUMN full_spent_time_ms INTEGER NOT NULL DEFAULT 0;

ALTER TABLE e_answered_data ADD COLUMN reflected_time_ms INTEGER NOT NULL DEFAULT 0;

ALTER TABLE users ALTER COLUMN email DROP NOT NULL;
UPDATE users SET email=NULL WHERE email LIKE 'invalid________________________________';

CREATE TABLE reset_email_secrets (
	user_id SERIAL PRIMARY KEY REFERENCES users,
	email VARCHAR NOT NULL,
	secret VARCHAR NOT NULL,
	added TIMESTAMPTZ NOT NULL DEFAULT current_timestamp
)

