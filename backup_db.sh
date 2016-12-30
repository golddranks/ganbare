#!/bin/sh
command -v pg_dump >/dev/null 2>&1 || { echo >&2 "I require pg_dump but it's not installed. Aborting."; exit 1; }
command -v xxd >/dev/null 2>&1 || { echo >&2 "I require xxd but it's not installed. Aborting."; exit 1; }

if [ -f "./.env" ]
then
. "./.env"
else
GANBARE_AUDIO_DIR="audio"
GANBARE_IMAGES_DIR="images"
fi

if [ -z $1 ]
then
echo "Usage: backup_db.sh <name of the database>" && exit 1
fi

echo "Starting backup!"
BACKUP_DIR="backups/$(date -u +"%Y-%m-%dT%H-%M-%SZ")-$(xxd -l 4 -p /dev/urandom )"
mkdir -p $BACKUP_DIR || { echo "Can't create the directory $BACKUP_DIR! Aborting."; exit 1; }
echo "Backing up to: $BACKUP_DIR Audio dir: $GANBARE_AUDIO_DIR Images dir: $GANBARE_IMAGES_DIR"
pg_dump -F d -h localhost $1 -f "$BACKUP_DIR/db_dump"
cp -r "$GANBARE_AUDIO_DIR" "${BACKUP_DIR}/"
cp -r "$GANBARE_IMAGES_DIR" "${BACKUP_DIR}/"
