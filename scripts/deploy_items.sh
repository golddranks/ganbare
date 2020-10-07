#!/bin/sh
set -eu

ENVFILE_LOCAL="${1:?Usage: deploy_items.sh <local env file> <remote env file> <dump_db|row_by_row|skip_db>}"
ENVFILE_REMOTE="${2:?Usage: deploy_items.sh <local env file> <remote env file> <dump_db|row_by_row|skip_db>}"
DB_DEPLOY="${3:?Usage: deploy_items.sh <local env file> <remote env file> <dump_db|row_by_row|skip_db>}"

. "$ENVFILE_LOCAL"
SOURCE_AUDIO_DIR=$AUDIO_DIR
SOURCE_IMAGES_DIR=$IMAGES_DIR
# Local DB parameters
export PGPASSWORD=$DB_PASSWORD
export PGHOST=$DB_HOSTNAME
export PGUSER=$DB_USER
export PGDATABASE=$DB_NAME
export PGPORT=$DB_PORT

echo "Local DB: $DB_HOSTNAME:$DB_PORT/$DB_NAME"

db_dump () {
	pg_dump --data-only $COLUMN_INSERTS -t audio_bundles -t narrators -t skill_nuggets -t audio_files -t words -t exercises -t exercise_variants -t quiz_questions -t question_answers "$DB_NAME" -f "$TEMPFILE"
	echo "Dumped DB to $TEMPFILE"
}

case "$DB_DEPLOY" in
	"dump_db" ) TEMPFILE="$(mktemp)" && COLUMN_INSERTS="" db_dump ;;
	"row_by_row" ) TEMPFILE="$(mktemp)" && COLUMN_INSERTS="--column-inserts" db_dump ;;
	"skip_db" ) TEMPFILE="" ;;
	* ) echo "Usage: deploy_items.sh <local env file> <remote env file> <dump_db|row_by_row|skip_db>" && exit 1 ;;
esac

# DB export done.

. "$ENVFILE_REMOTE"
# Re-define these to the remote DB parameters
export PGPASSWORD=$DB_PASSWORD
export PGHOST=$DB_HOSTNAME
export PGUSER=$DB_USER
export PGDATABASE=$DB_NAME
export PGPORT=$DB_PORT

echo "Remote DB: $DB_HOSTNAME:$DB_PORT/$DB_NAME"
if [ -f "$TEMPFILE" ]; then
	psql -f "$TEMPFILE"
else
	echo "Skipping DB deploying."
fi

echo "Upload audio files to the server"
rsync -r --exclude "$SOURCE_AUDIO_DIR/trash" "$SOURCE_AUDIO_DIR/" "$DEPLOY_SSH_HOSTNAME:$AUDIO_DIR"
echo "Upload image files to the server"
rsync -r --exclude "$SOURCE_IMAGES_DIR/trash" "$SOURCE_IMAGES_DIR/" "$DEPLOY_SSH_HOSTNAME:$IMAGES_DIR"
