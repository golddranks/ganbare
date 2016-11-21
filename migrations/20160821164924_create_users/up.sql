CREATE TABLE users (
	id SERIAL PRIMARY KEY,
	email VARCHAR NOT NULL UNIQUE,
	joined TIMESTAMPTZ NOT NULL DEFAULT current_timestamp
);

CREATE TABLE pending_email_confirms (
	secret VARCHAR PRIMARY KEY,
	email VARCHAR NOT NULL,
	groups INTEGER[] NOT NULL,
	added TIMESTAMPTZ NOT NULL DEFAULT current_timestamp
);

CREATE TABLE passwords (
	id SERIAL REFERENCES users ON DELETE CASCADE PRIMARY KEY,
	password_hash BYTEA NOT NULL,
	salt BYTEA NOT NULL,
	initial_rounds SMALLINT NOT NULL DEFAULT 0,
	extra_rounds SMALLINT NOT NULL DEFAULT 0
);

CREATE TABLE sessions (
	id SERIAL PRIMARY KEY,
	sess_id BYTEA NOT NULL UNIQUE,
	user_id SERIAL REFERENCES users ON DELETE CASCADE,
	started TIMESTAMPTZ NOT NULL DEFAULT current_timestamp,
	last_seen TIMESTAMPTZ NOT NULL DEFAULT current_timestamp,
	last_ip BYTEA NOT NULL
);

CREATE TABLE user_groups (
	id SERIAL PRIMARY KEY,
	group_name VARCHAR NOT NULL
);

CREATE TABLE group_memberships (
	user_id SERIAL REFERENCES users,
	group_id SERIAL REFERENCES user_groups,
	PRIMARY KEY(user_id, group_id)
	);

INSERT INTO user_groups VALUES (1, 'admins'), (2, 'editors'), (3, 'testers'), (4, 'input_group'), (5, 'output_group');
