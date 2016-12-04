#!/bin/sh
DEPLOY_STATIC_DIR=/srv/ganbare_testing
DEPLOY_SERVER=ganba.re
DEPLOY_DB_NAME=ganbare_testing
DEPLOY_LOCAL_PEPPERFILE=.env.ganbare_testing_runtime_pepper

rsync -r images $DEPLOY_SERVER:$DEPLOY_STATIC_DIR/
rsync -r audio $DEPLOY_SERVER:$DEPLOY_STATIC_DIR/
rsync -r backups $DEPLOY_SERVER:$DEPLOY_STATIC_DIR/
ssh $DEPLOY_SERVER <<EOF
NEWEST_BACKUP=\$( find $DEPLOY_STATIC_DIR/backups/*/db_dump -type d | tail -n 1 )
docker pull golddranks/ganbare_run
docker stop ganbare_runner_testing && docker rm ganbare_runner_testing
docker run -d --restart=unless-stopped \
--name ganbare_runner_testing \
--link ganbare-postgres \
-p 8087:8080 \
-e "GANBARE_DATABASE_URL=postgres://\$(whoami)@ganbare-postgres/$DEPLOY_DB_NAME" \
-e "GANBARE_RUNTIME_PEPPER=$(cat $DEPLOY_LOCAL_PEPPERFILE)" \
-e "GANBARE_SITE_DOMAIN=testing.ganba.re" \
-v $DEPLOY_STATIC_DIR/audio:/ganbare/audio \
-v $DEPLOY_STATIC_DIR/images:/ganbare/images \
golddranks/ganbare_run
EOF
