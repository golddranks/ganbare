CREATE TABLE answer_data (
	id SERIAL PRIMARY KEY,
	user_id SERIAL REFERENCES users,
	q_audio_id SERIAL REFERENCES audio_files,
	correct_qa_id SERIAL REFERENCES question_answers,
	answered_qa_id INTEGER REFERENCES question_answers,
	answered_date TIMESTAMPTZ NOT NULL DEFAULT current_timestamp,
	answer_time_ms INTEGER NOT NULL,
	correct BOOLEAN
);

CREATE TABLE question_data (
	user_id SERIAL REFERENCES users,
	question_id SERIAL REFERENCES quiz_questions,
	due_date TIMESTAMPTZ NOT NULL,
	due_delay INTEGER NOT NULL,
	PRIMARY KEY(user_id, question_id)
);

CREATE TABLE word_data (
	user_id SERIAL REFERENCES users,
	word_id SERIAL REFERENCES words,
	answer_time_ms INTEGER NOT NULL,
	audio_times INTEGER NOT NULL,
	PRIMARY KEY(user_id, word_id)
);

CREATE TABLE skill_data (
	user_id SERIAL REFERENCES users,
	skill_nugget SERIAL REFERENCES skill_nuggets,
	skill_level INTEGER,
	PRIMARY KEY(user_id, skill_nugget)
);
