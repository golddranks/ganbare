#!/bin/sh
set -eu

BACKUP_NAME=${1:?Usage: backup_db.sh <name of the backup> <env file>}
ENV_FILE=${2:?Usage: backup_db.sh <name of the backup> <env file>}

. "$ENV_FILE"

if [ ! -d "$AUDIO_DIR" ]; then
	echo "The audio dir $AUDIO_DIR doesn't exist!" && exit 1
fi

if [ ! -d "$IMAGES_DIR" ]; then
	echo "The image dir $IMAGES_DIR doesn't exist!" && exit 1
fi

echo "Starting backup!"
BACKUP_DIR="backups/$(date -u +"%Y-%m-%dT%H-%M-%SZ")-$(xxd -l 4 -p /dev/urandom )_$BACKUP_NAME"
mkdir -p "$BACKUP_DIR" || { echo "Can't create the directory $BACKUP_DIR! Aborting."; exit 1; }
echo "Backing up to: $BACKUP_DIR Audio dir: $AUDIO_DIR Images dir: $IMAGES_DIR"

export PGPASSWORD=$DB_PASSWORD
export PGHOST=$DB_HOSTNAME
export PGUSER=$DB_USER
export PGDATABASE=$DB_NAME
export PGPORT=$DB_PORT
pg_dump -F d "$DB_NAME" -f "$BACKUP_DIR/db_dump"

cp -r "$AUDIO_DIR" "${BACKUP_DIR}/"
cp -r "$IMAGES_DIR" "${BACKUP_DIR}/"
