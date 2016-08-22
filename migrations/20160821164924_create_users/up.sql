CREATE TABLE users (
	id SERIAL PRIMARY KEY,
	email VARCHAR NOT NULL UNIQUE,
	joined timestamptz NOT NULL DEFAULT current_timestamp,
	password_hash BYTEA NOT NULL,
	salt BYTEA NOT NULL,
	extra_rounds SMALLINT NOT NULL DEFAULT 0
)
