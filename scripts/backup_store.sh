#!/bin/sh
command -v pg_dump >/dev/null 2>&1 || { echo >&2 "I require pg_dump but it's not installed. Aborting."; exit 1; }
command -v xxd >/dev/null 2>&1 || { echo >&2 "I require xxd but it's not installed. Aborting."; exit 1; }

DB_NAME=${1:?Usage: backup_db.sh <name of the database> <name of the backup>}
BACKUP_NAME=${2:?Usage: backup_db.sh <name of the database> <name of the backup>}

GANBARE_AUDIO_DIR="audio"
GANBARE_IMAGES_DIR="images"

if [ -f ./.env ]; then
	. .env
fi

if [ ! -d "$GANBARE_AUDIO_DIR" ]; then
	echo "The audio dir $GANBARE_AUDIO_DIR doesn't exist!" && exit 1
fi

if [ ! -d "$GANBARE_IMAGES_DIR" ]; then
	echo "The image dir $GANBARE_IMAGES_DIR doesn't exist!" && exit 1
fi

echo "Starting backup!"
BACKUP_DIR="backups/$(date -u +"%Y-%m-%dT%H-%M-%SZ")-$(xxd -l 4 -p /dev/urandom )_$BACKUP_NAME"
mkdir -p "$BACKUP_DIR" || { echo "Can't create the directory $BACKUP_DIR! Aborting."; exit 1; }
echo "Backing up to: $BACKUP_DIR Audio dir: $GANBARE_AUDIO_DIR Images dir: $GANBARE_IMAGES_DIR"
pg_dump -F d -h localhost "$DB_NAME" -f "$BACKUP_DIR/db_dump"
cp -r "$GANBARE_AUDIO_DIR" "${BACKUP_DIR}/"
cp -r "$GANBARE_IMAGES_DIR" "${BACKUP_DIR}/"
