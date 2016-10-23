CREATE TABLE skill_nuggets (
	id SERIAL PRIMARY KEY,
	skill_summary VARCHAR NOT NULL
);

CREATE TABLE quiz_questions (
	id SERIAL PRIMARY KEY,
	skill_id SERIAL REFERENCES skill_nuggets,
	q_name VARCHAR NOT NULL,
	q_explanation VARCHAR NOT NULL
);

CREATE TABLE question_answers (
	id SERIAL PRIMARY KEY,
	question_id SERIAL REFERENCES quiz_questions,
	answer_text VARCHAR NOT NULL
);

CREATE TABLE question_audio (
	id SERIAL PRIMARY KEY,
	answer_id SERIAL REFERENCES question_answers,
	audio_file VARCHAR NOT NULL
);
