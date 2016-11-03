CREATE TABLE skill_nuggets (
	id SERIAL PRIMARY KEY,
	skill_summary VARCHAR NOT NULL
);

CREATE TABLE quiz_questions (
	id SERIAL PRIMARY KEY,
	skill_id INTEGER REFERENCES skill_nuggets,
	q_name VARCHAR NOT NULL,
	q_explanation VARCHAR NOT NULL,
	question_text VARCHAR NOT NULL
);

CREATE TABLE narrators (
	id SERIAL PRIMARY KEY,
	name VARCHAR NOT NULL
);

CREATE TABLE audio_files (
	id SERIAL PRIMARY KEY,
	narrators_id SERIAL REFERENCES narrators,
	file_path VARCHAR NOT NULL,
	mime VARCHAR NOT NULL
);

CREATE TABLE question_answers (
	id SERIAL PRIMARY KEY,
	question_id SERIAL REFERENCES quiz_questions,
	audio_files_id INTEGER REFERENCES audio_files,
	answer_text VARCHAR NOT NULL
);

CREATE TABLE question_audio (
	id SERIAL REFERENCES audio_files ON DELETE CASCADE PRIMARY KEY,
	question_answers_id SERIAL REFERENCES question_answers
);
