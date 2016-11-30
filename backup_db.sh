#!/bin/sh
command -v pg_dump >/dev/null 2>&1 || { echo >&2 "I require pg_dump but it's not installed. Aborting."; exit 1; }
if [ -z $1 ]
then
echo "Usage: backup_db.sh <name of the database>"
else
BACKUP_DIR="backups/$(date "+%Y-%m-%d")-$(xxd -l 4 -p /dev/urandom )"
mkdir -p $BACKUP_DIR
pg_dump -F d -h localhost $1 -f "$BACKUP_DIR/db_dump"
cp -r "audio" "${BACKUP_DIR}/"
cp -r "images" "${BACKUP_DIR}/"
fi
