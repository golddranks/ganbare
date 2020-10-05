#!/bin/sh

. scripts/local.env

command -v pg_dump >/dev/null 2>&1 || { echo >&2 "I require pg_dump but it's not installed. Aborting."; exit 1; }
command -v xxd >/dev/null 2>&1 || { echo >&2 "I require xxd but it's not installed. Aborting."; exit 1; }

BACKUP_NAME=${1:?Usage: backup_db.sh <name of the backup>}

LOCAL_AUDIO_DIR="audio"
LOCAL_IMAGES_DIR="images"

if [ -f ./.env ]; then
	. .env
fi

if [ ! -d "$LOCAL_AUDIO_DIR" ]; then
	echo "The audio dir $LOCAL_AUDIO_DIR doesn't exist!" && exit 1
fi

if [ ! -d "$LOCAL_IMAGES_DIR" ]; then
	echo "The image dir $LOCAL_IMAGES_DIR doesn't exist!" && exit 1
fi

echo "Starting backup!"
BACKUP_DIR="backups/$(date -u +"%Y-%m-%dT%H-%M-%SZ")-$(xxd -l 4 -p /dev/urandom )_$BACKUP_NAME"
mkdir -p "$BACKUP_DIR" || { echo "Can't create the directory $BACKUP_DIR! Aborting."; exit 1; }
echo "Backing up to: $BACKUP_DIR Audio dir: $LOCAL_AUDIO_DIR Images dir: $LOCAL_IMAGES_DIR"
PGPASSWORD="$LOCAL_DB_PASSWORD" pg_dump -F d -h "$LOCAL_DB_HOSTNAME" -U "$LOCAL_DB_USER" "$DB_NAME" -f "$BACKUP_DIR/db_dump"
cp -r "$LOCAL_AUDIO_DIR" "${BACKUP_DIR}/"
cp -r "$LOCAL_IMAGES_DIR" "${BACKUP_DIR}/"