#!/bin/sh
DEPLOY_ROOT_DIR=/srv/ganbare_production
DEPLOY_SERVER=akusento.ganba.re
DEPLOY_DB_NAME=ganbare_production
DEPLOY_LOCAL_PEPPER=$(cat .env.ganbare_production_runtime_pepper)
DEPLOY_PORT=8000
DEPLOY_SITE_DOMAIN=akusento.ganba.re
DEPLOY_SITE_LINK=https://akusento.ganba.re/
DEPLOY_EMAIL_DOMAIN=ganba.re
DEPLOY_EMAIL_SERVER=smtp.mailgun.org:587
DEPLOY_EMAIL_SMTP_USERNAME=postmaster@ganba.re
DEPLOY_EMAIL_SMTP_PASSWORD=$(cat .env.ganbare_email_password)
DEPLOY_BUILD_NUMBER="$(cat build_number.txt)"
DEPLOY_COMMIT_NAME="$(git log HEAD --oneline --no-walk)"
DEPLOY_PARANOID=true
DEPLOY_CONTAINER_NAME=ganbare_runner_production
DEPLOY_LOGLEVEL=ganbare=debug,ganbare_backend=debug
DEPLOY_CACHE_MAX_AGE=1200
DEPLOY_SERVER_THREADS=25

ssh $DEPLOY_SERVER /bin/sh <<EOF
docker pull golddranks/ganbare_run
docker stop $DEPLOY_CONTAINER_NAME && docker rm $DEPLOY_CONTAINER_NAME
docker run -d --restart=unless-stopped \
--name $DEPLOY_CONTAINER_NAME \
--link ganbare-postgres \
-p $DEPLOY_PORT:8080 \
-e "GANBARE_DATABASE_URL=postgres://\$(whoami)@ganbare-postgres/$DEPLOY_DB_NAME" \
-e "GANBARE_RUNTIME_PEPPER=$DEPLOY_LOCAL_PEPPER" \
-e "GANBARE_SITE_DOMAIN=$DEPLOY_SITE_DOMAIN" \
-e "GANBARE_EMAIL_DOMAIN=$DEPLOY_EMAIL_DOMAIN" \
-e "GANBARE_EMAIL_SERVER=$DEPLOY_EMAIL_SERVER" \
-e "GANBARE_EMAIL_SMTP_USERNAME=$DEPLOY_EMAIL_SMTP_USERNAME" \
-e "GANBARE_EMAIL_SMTP_PASSWORD=$DEPLOY_EMAIL_SMTP_PASSWORD" \
-e "GANBARE_SITE_LINK=$DEPLOY_SITE_LINK" \
-e "GANBARE_BUILD_NUMBER=$DEPLOY_BUILD_NUMBER" \
-e "GANBARE_COMMIT_NAME=$DEPLOY_COMMIT_NAME" \
-e "GANBARE_PARANOID=$DEPLOY_PARANOID" \
-e "GANBARE_CACHE_MAX_AGE=$DEPLOY_CACHE_MAX_AGE" \
-e "GANBARE_SERVER_THREADS=$DEPLOY_SERVER_THREADS" \
-e "RUST_LOG=$DEPLOY_LOGLEVEL" \
-v $DEPLOY_ROOT_DIR/audio:/ganbare/audio \
-v $DEPLOY_ROOT_DIR/images:/ganbare/images \
-v $DEPLOY_ROOT_DIR/user_audio:/ganbare/user_audio \
golddranks/ganbare_run
EOF
