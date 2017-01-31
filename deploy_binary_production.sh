#!/bin/sh
DEPLOY_STATIC_DIR=/srv/ganbare_production
DEPLOY_SERVER=ganba.re
DEPLOY_DB_NAME=ganbare_production
DEPLOY_LOCAL_PEPPERFILE=.env.ganbare_production_runtime_pepper
DEPLOY_PORT=8088
DEPLOY_DOMAIN=akusento.ganba.re

rsync -r images $DEPLOY_SERVER:$DEPLOY_STATIC_DIR/
rsync -r audio $DEPLOY_SERVER:$DEPLOY_STATIC_DIR/
ssh $DEPLOY_SERVER /bin/sh <<EOF
docker pull golddranks/ganbare_run
docker stop ganbare_runner_production && docker rm ganbare_runner_production
docker run -d --restart=unless-stopped \
--name ganbare_runner_production \
--link ganbare-postgres \
-p $DEPLOY_PORT:8080 \
-e "GANBARE_DATABASE_URL=postgres://\$(whoami)@ganbare-postgres/$DEPLOY_DB_NAME" \
-e "GANBARE_RUNTIME_PEPPER=$(cat $DEPLOY_LOCAL_PEPPERFILE)" \
-e "GANBARE_SITE_DOMAIN=$DEPLOY_DOMAIN" \
-v $DEPLOY_STATIC_DIR/audio:/ganbare/audio \
-v $DEPLOY_STATIC_DIR/images:/ganbare/images \
golddranks/ganbare_run
EOF
