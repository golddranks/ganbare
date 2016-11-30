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
	proposed_token BYTEA NOT NULL UNIQUE,
	current_token BYTEA NOT NULL UNIQUE,
	retired_token BYTEA NOT NULL UNIQUE,
	access_version INTEGER NOT NULL DEFAULT 0,
	user_id SERIAL REFERENCES users ON DELETE CASCADE,
	started TIMESTAMPTZ NOT NULL DEFAULT current_timestamp,
	last_seen TIMESTAMPTZ NOT NULL DEFAULT current_timestamp,
	last_ip BYTEA NOT NULL
);

CREATE TABLE user_groups (
	id SERIAL PRIMARY KEY,
	group_name VARCHAR NOT NULL UNIQUE,
	anonymous BOOLEAN NOT NULL default false
);

CREATE TABLE group_memberships (
	user_id SERIAL REFERENCES users,
	group_id SERIAL REFERENCES user_groups,
	anonymous BOOLEAN NOT NULL default false,
	PRIMARY KEY(user_id, group_id)
);

CREATE TABLE anon_aliases (
	id SERIAL PRIMARY KEY,
	name VARCHAR NOT NULL,
	user_id INTEGER REFERENCES users,
	group_id INTEGER REFERENCES user_groups
);

INSERT INTO user_groups VALUES (1, 'admins'), (2, 'editors'), (3, 'betatesters'), (4, 'subjects'), (5, 'input_group'), (6, 'output_group'), (7, 'survey');
