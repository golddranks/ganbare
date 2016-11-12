CREATE TABLE answer_data (
	id SERIAL PRIMARY KEY,
	user_id SERIAL REFERENCES users,
	q_audio_id SERIAL REFERENCES question_audio,
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

CREATE TABLE audio_bundles (
	id SERIAL PRIMARY KEY,
	listname VARCHAR NOT NULL
);

CREATE TABLE audio_bundle_memberships (
	bundle_id SERIAL REFERENCES audio_bundles,
	file_id SERIAL REFERENCES audio_files,
	PRIMARY KEY(bundle_id, file_id)
);

CREATE TABLE words (
	id SERIAL PRIMARY KEY,
	word VARCHAR NOT NULL,
	explanation VARCHAR NOT NULL,
	audio_bundle SERIAL REFERENCES audio_bundles,
	skill_nugget INTEGER REFERENCES skill_nuggets
);
