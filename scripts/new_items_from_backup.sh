#!/bin/sh

echo "Local DB: $LOCAL_DB_NAME

pg_dump -h localhost --data-only --column-inserts -t audio_bundles -t narrators -t skill_nuggets -t audio_files -t words -t exercises -t exercise_variants -t quiz_questions -t question_answers $LOCAL_DB_NAME -f new_items_temp.sql
echo "Moving the SQL data to the server"
rsync -r new_items_temp.sql $DEPLOY_SERVER:
rm new_items_temp.sql
echo "SSH to server and insert the data"
ssh $DEPLOY_SERVER /bin/sh <<EOF
psql -h localhost -d $DEPLOY_DB_NAME -f new_items_temp.sql
rm new_items_temp.sql
EOF
echo "Upload audio files to the server"
rsync -r --exclude $LOCAL_AUDIO_DIR/trash $LOCAL_AUDIO_DIR/ $DEPLOY_SERVER:$DEPLOY_AUDIO_DIR
echo "Upload image files to the server"
rsync -r --exclude $LOCAL_IMAGES_DIR/trash $LOCAL_IMAGES_DIR/ $DEPLOY_SERVER:$DEPLOY_IMAGES_DIR
