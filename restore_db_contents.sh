#!/bin/sh
command -v pg_restore >/dev/null 2>&1 || { echo >&2 "I require pg_restore but it's not installed. Aborting."; exit 1; }
if [ -z $1 ]
then
echo "Usage: restore_db_contents.sh <name of the database> <location of the backup dir/file>"
else
pg_restore -h localhost -d $1 -a -t audio_bundles -t narrators -t skill_nuggets $2
pg_restore -h localhost -d $1 -a -t audio_files $2
pg_restore -h localhost -d $1 -a -t words -t quiz_questions $2
pg_restore -h localhost -d $1 -a -t question_answers $2
fi
