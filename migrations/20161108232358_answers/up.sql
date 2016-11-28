CREATE TABLE due_items (
	id SERIAL PRIMARY KEY,
	user_id SERIAL REFERENCES users,
	due_date TIMESTAMPTZ NOT NULL,
	due_delay INTEGER NOT NULL,
	correct_streak INTEGER NOT NULL DEFAULT 0,
	item_type VARCHAR NOT NULL
);

CREATE TABLE pending_items (
	id SERIAL PRIMARY KEY,
	user_id SERIAL REFERENCES users,
	audio_file_id SERIAL REFERENCES audio_files,
	asked_date TIMESTAMPTZ NOT NULL DEFAULT current_timestamp,
	pending BOOLEAN NOT NULL DEFAULT true,
	item_type VARCHAR NOT NULL
);

CREATE TABLE q_asked_data (
	id SERIAL REFERENCES pending_items PRIMARY KEY,
	question_id SERIAL REFERENCES quiz_questions,
	correct_qa_id SERIAL REFERENCES question_answers
);

CREATE TABLE q_answered_data (
	id SERIAL REFERENCES q_asked_data PRIMARY KEY,
	answered_qa_id INTEGER REFERENCES question_answers,
	answered_date TIMESTAMPTZ NOT NULL,
	active_answer_time_ms INTEGER NOT NULL,
	full_answer_time_ms INTEGER NOT NULL
);

CREATE TABLE e_asked_data (
	id SERIAL REFERENCES pending_items PRIMARY KEY,
	word_id SERIAL REFERENCES words
);

CREATE TABLE e_answered_data (
	id SERIAL REFERENCES e_asked_data PRIMARY KEY,
	answered_date TIMESTAMPTZ NOT NULL DEFAULT current_timestamp,
	active_answer_time_ms INTEGER NOT NULL,
	full_answer_time_ms INTEGER NOT NULL,
	audio_times INTEGER NOT NULL,
	answer_level INTEGER
);

CREATE TABLE w_asked_data (
	id SERIAL REFERENCES pending_items PRIMARY KEY,
	word_id SERIAL REFERENCES words,
	show_accents BOOLEAN NOT NULL
);

CREATE TABLE w_answered_data (
	id SERIAL REFERENCES w_asked_data PRIMARY KEY,
	answer_time_ms INTEGER NOT NULL,
	audio_times INTEGER NOT NULL,
	checked_date TIMESTAMPTZ NOT NULL DEFAULT current_timestamp
);

CREATE TABLE question_data (
	question_id SERIAL REFERENCES quiz_questions,
	due SERIAL REFERENCES due_items,
	PRIMARY KEY(due, question_id)
);

CREATE TABLE exercise_data (
	word_id SERIAL REFERENCES words,
	due SERIAL REFERENCES due_items,
	PRIMARY KEY(due, word_id)
);

CREATE TABLE skill_data (
	user_id SERIAL REFERENCES users,
	skill_nugget SERIAL REFERENCES skill_nuggets,
	skill_level INTEGER NOT NULL DEFAULT 0,
	PRIMARY KEY(user_id, skill_nugget)
);

CREATE TABLE user_metrics (
	id SERIAL PRIMARY KEY REFERENCES users,
	new_words_since_break INTEGER NOT NULL DEFAULT 0,
	new_sentences_since_break INTEGER NOT NULL DEFAULT 0,
	new_words_today INTEGER NOT NULL DEFAULT 0,
	new_sentences_today INTEGER NOT NULL DEFAULT 0,
	break_until TIMESTAMPTZ NOT NULL DEFAULT current_timestamp,
	today TIMESTAMPTZ NOT NULL DEFAULT current_timestamp
);

INSERT INTO user_metrics (id) SELECT id FROM users;

CREATE TABLE event_experiences (
	user_id SERIAL REFERENCES users,
	event_id SERIAL REFERENCES events,
	event_init TIMESTAMPTZ NOT NULL DEFAULT current_timestamp,
	event_finish TIMESTAMPTZ,
	PRIMARY KEY(user_id, event_id)
);
