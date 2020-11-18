-- Your SQL goes here
ALTER TABLE reset_email_secrets ADD COLUMN type VARCHAR NOT NULL DEFAULT 'pw_reset';
ALTER TABLE reset_email_secrets DROP CONSTRAINT reset_email_secrets_pkey;
ALTER TABLE reset_email_secrets ADD PRIMARY KEY (user_id, type);
