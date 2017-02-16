#!/bin/sh
command -v pg_restore >/dev/null 2>&1 || { echo >&2 "I require pg_restore but it's not installed. Aborting."; exit 1; }
if [ -z $1 ]
then
echo "Usage: restore_db_contents.sh <name of the database> <location of the backup dir/file>"
else
pg_restore -h localhost -d $1 -a -t audio_bundles_id_seq -t audio_bundles -t narrators_id_seq -t narrators -t skill_nuggets_id_seq -t skill_nuggets $2
pg_restore -h localhost -d $1 -a -t audio_files_id_seq -t audio_files $2
pg_restore -h localhost -d $1 -a -t words_id_seq -t words -t quiz_questions_id_seq -t quiz_questions $2
pg_restore -h localhost -d $1 -a -t question_answers_id_seq -t question_answers -t exercises_id_seq -t exercises $2
pg_restore -h localhost -d $1 -a -t exercise_variants_id_seq -t exercise_variants $2
fi
