CREATE TABLE due_items (
	id SERIAL PRIMARY KEY,
	user_id SERIAL REFERENCES users,
	due_date TIMESTAMPTZ NOT NULL,
	due_delay INTEGER NOT NULL,
	correct_streak INTEGER NOT NULL DEFAULT 0,
	item_type CHAR(8) NOT NULL
);

CREATE TABLE q_answer_data (
	id SERIAL PRIMARY KEY,
	user_id SERIAL REFERENCES users,
	question_id SERIAL REFERENCES quiz_questions,
	q_audio_id SERIAL REFERENCES audio_files,
	correct_qa_id SERIAL REFERENCES question_answers,
	answered_qa_id INTEGER REFERENCES question_answers,
	answered_date TIMESTAMPTZ NOT NULL DEFAULT current_timestamp,
	active_answer_time_ms INTEGER NOT NULL,
	full_answer_time_ms INTEGER NOT NULL,
	correct BOOLEAN
);

CREATE TABLE e_answer_data (
	id SERIAL PRIMARY KEY,
	user_id SERIAL REFERENCES users,
	word_id SERIAL REFERENCES words,
	answered_date TIMESTAMPTZ NOT NULL DEFAULT current_timestamp,
	active_answer_time_ms INTEGER NOT NULL,
	full_answer_time_ms INTEGER NOT NULL,
	audio_times INTEGER NOT NULL,
	correct BOOLEAN
);

CREATE TABLE word_data (
	user_id SERIAL REFERENCES users,
	word_id SERIAL REFERENCES words,
	answer_time_ms INTEGER NOT NULL,
	audio_times INTEGER NOT NULL,
	checked_date TIMESTAMPTZ NOT NULL DEFAULT current_timestamp,
	PRIMARY KEY(user_id, word_id)
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
	event_time TIMESTAMPTZ,
	PRIMARY KEY(user_id, event_id)
);
