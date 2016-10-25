CREATE TABLE skill_nuggets (
	id SERIAL PRIMARY KEY,
	skill_summary VARCHAR NOT NULL
);

CREATE TABLE quiz_questions (
	id SERIAL PRIMARY KEY,
	skill_id INTEGER REFERENCES skill_nuggets,
	q_name VARCHAR NOT NULL,
	q_explanation VARCHAR NOT NULL
);

CREATE TABLE question_answers (
	id SERIAL PRIMARY KEY,
	question_id SERIAL REFERENCES quiz_questions,
	answer_text VARCHAR NOT NULL,
	answer_audio VARCHAR NOT NULL
);

CREATE TABLE narrators (
	id SERIAL PRIMARY KEY,
	name VARCHAR NOT NULL
);

CREATE TABLE question_audio (
	id SERIAL PRIMARY KEY,
	answer_id SERIAL REFERENCES question_answers,
	narrator_id SERIAL REFERENCES narrators,
	audio_file VARCHAR NOT NULL
);
