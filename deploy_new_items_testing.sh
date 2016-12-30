#!/bin/sh

DEPLOY_SERVER=ganba.re
LOCAL_DB_NAME=ganbare_dev
LOCAL_AUDIO_DIR=audio
LOCAL_IMAGES_DIR=images
DEPLOY_DB_NAME=ganbare_testing
DEPLOY_AUDIO_DIR=/srv/ganbare_testing/audio
DEPLOY_IMAGES_DIR=/srv/ganbare_testing/images

pg_dump -h localhost --data-only --column-inserts -t audio_bundles -t narrators -t skill_nuggets -t audio_files -t words -t quiz_questions -t question_answers $LOCAL_DB_NAME -f new_items_temp.sql
rsync -r new_items_temp.sql $DEPLOY_SERVER:
rm new_items_temp.sql
ssh $DEPLOY_SERVER <<EOF
psql -h localhost -d $DEPLOY_DB_NAME -f new_items_temp.sql
rm new_items_temp.sql
EOF
rsync -r --exclude $LOCAL_AUDIO_DIR/trash $LOCAL_AUDIO_DIR/ $DEPLOY_SERVER:$DEPLOY_AUDIO_DIR
rsync -r --exclude $LOCAL_IMAGES_DIR/trash $LOCAL_IMAGES_DIR/ $DEPLOY_SERVER:$DEPLOY_IMAGES_DIR
