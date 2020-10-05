#!/bin/sh
command -v pg_restore >/dev/null 2>&1 || { echo >&2 "I require pg_restore but it's not installed. Aborting."; exit 1; }
DB_NAME=${1:?Usage: restore_db_contents.sh <name of the database> <location of the backup dir/file>}
BACKUP_NAME=${2:?Usage: restore_db_contents.sh <name of the database> <location of the backup dir/file>}

pg_restore -h localhost -d "$DB_NAME" -a -t audio_bundles_id_seq -t audio_bundles -t narrators_id_seq -t narrators -t skill_nuggets_id_seq -t skill_nuggets "$BACKUP_NAME"
pg_restore -h localhost -d "$DB_NAME" -a -t audio_files_id_seq -t audio_files "$BACKUP_NAME"
pg_restore -h localhost -d "$DB_NAME" -a -t words_id_seq -t words -t quiz_questions_id_seq -t quiz_questions "$BACKUP_NAME"
pg_restore -h localhost -d "$DB_NAME" -a -t question_answers_id_seq -t question_answers -t exercises_id_seq -t exercises "$BACKUP_NAME"
pg_restore -h localhost -d "$DB_NAME" -a -t exercise_variants_id_seq -t exercise_variants "$BACKUP_NAME"
