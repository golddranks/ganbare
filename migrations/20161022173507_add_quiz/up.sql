CREATE TABLE skill_nuggets (
	id SERIAL PRIMARY KEY,
	skill_summary VARCHAR NOT NULL
);

CREATE TABLE audio_bundles (
	id SERIAL PRIMARY KEY,
	listname VARCHAR NOT NULL
);

CREATE TABLE narrators (
	id SERIAL PRIMARY KEY,
	name VARCHAR NOT NULL
);

CREATE TABLE audio_files (
	id SERIAL PRIMARY KEY,
	narrators_id SERIAL REFERENCES narrators,
	bundle_id SERIAL REFERENCES audio_bundles,
	file_path VARCHAR NOT NULL,
	mime VARCHAR NOT NULL
);

CREATE TABLE quiz_questions (
	id SERIAL PRIMARY KEY,
	skill_id INTEGER REFERENCES skill_nuggets,
	q_name VARCHAR NOT NULL,
	q_explanation VARCHAR NOT NULL,
	question_text VARCHAR NOT NULL
);

CREATE TABLE question_answers (
	id SERIAL PRIMARY KEY,
	question_id SERIAL REFERENCES quiz_questions,
	a_audio_bundle INTEGER REFERENCES audio_bundles,
	q_audio_bundle SERIAL REFERENCES audio_bundles,
	answer_text VARCHAR NOT NULL
);

CREATE TABLE words (
	id SERIAL PRIMARY KEY,
	word VARCHAR NOT NULL,
	explanation VARCHAR NOT NULL,
	audio_bundle SERIAL REFERENCES audio_bundles,
	skill_nugget INTEGER REFERENCES skill_nuggets
);
