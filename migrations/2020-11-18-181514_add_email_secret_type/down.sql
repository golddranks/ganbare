-- This file should undo anything in `up.sql`
ALTER TABLE reset_email_secrets DROP CONSTRAINT reset_email_secrets_pkey;
ALTER TABLE reset_email_secrets ADD PRIMARY KEY (user_id);
ALTER TABLE reset_email_secrets DROP COLUMN type;
