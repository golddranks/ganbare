#!/bin/sh

DEPLOY_SERVER=ganba.re
LOCAL_DB_NAME=ganbare_dev
DEPLOY_DB_NAME=ganbare_testing

pg_dump -h localhost --data-only --column-inserts -t audio_bundles -t narrators -t skill_nuggets -t audio_files -t words -t quiz_questions -t question_answers $LOCAL_DB_NAME -f new_items_temp.sql
rsync -r new_items_temp.sql ganba.re:
rm new_items_temp.sql
ssh $DEPLOY_SERVER <<EOF
psql -h localhost -d $DEPLOY_DB_NAME -f new_items_temp.sql
rm new_items_temp.sql
EOF
