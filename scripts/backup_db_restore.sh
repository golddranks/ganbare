#!/bin/sh
set -eu

BACKUP_NAME=${1:?Usage: restore_db_contents.sh <location of the backup dir/file> <env file>}
ENV_FILE=${2:?Usage: restore_db_contents.sh <location of the backup dir/file> <env file>}

. "$ENV_FILE"

BACKUP_DIR="$BACKUP_NAME/db_dump"

export PGPASSWORD=$DB_PASSWORD
export PGHOST=$DB_HOSTNAME
export PGUSER=$DB_USER
export PGDATABASE=$DB_NAME
export PGPORT=$DB_PORT
pg_restore -d "$DB_NAME" --data-only -t audio_bundles_id_seq -t audio_bundles -t narrators_id_seq -t narrators -t skill_nuggets_id_seq -t skill_nuggets "$BACKUP_DIR"
pg_restore -d "$DB_NAME" --data-only -t audio_files_id_seq -t audio_files "$BACKUP_DIR"
pg_restore -d "$DB_NAME" --data-only -t words_id_seq -t words -t quiz_questions_id_seq -t quiz_questions "$BACKUP_DIR"
pg_restore -d "$DB_NAME" --data-only -t question_answers_id_seq -t question_answers -t exercises_id_seq -t exercises "$BACKUP_DIR"
pg_restore -d "$DB_NAME" --data-only -t exercise_variants_id_seq -t exercise_variants "$BACKUP_DIR"
