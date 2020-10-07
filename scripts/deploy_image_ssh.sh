#!/bin/sh
set -eu

ENVFILE="${1:?Usage: deploy_binary.sh <prd.env|stg.env>}"
. "$ENVFILE"

# shellcheck disable=SC2087
ssh "${DEPLOY_SSH_HOSTNAME:?}" /bin/sh <<EOF
docker pull golddranks/ganbare_run
docker stop $CONTAINER_NAME && docker rm $CONTAINER_NAME
docker run -d --restart=unless-stopped \
--name $CONTAINER_NAME \
--link ganbare-postgres \
-p $PORT:8080 \
-e "GANBARE_DATABASE_URL=postgres://\$(whoami)@ganbare-postgres/$DB_NAME" \
-e "GANBARE_RUNTIME_PEPPER=$LOCAL_PEPPER" \
-e "GANBARE_SITE_DOMAIN=$SITE_DOMAIN" \
-e "GANBARE_EMAIL_DOMAIN=$EMAIL_DOMAIN" \
-e "GANBARE_EMAIL_SERVER=$EMAIL_SERVER" \
-e "GANBARE_EMAIL_SMTP_USERNAME=$EMAIL_SMTP_USERNAME" \
-e "GANBARE_EMAIL_SMTP_PASSWORD=$EMAIL_SMTP_PASSWORD" \
-e "GANBARE_SITE_LINK=$SITE_LINK" \
-e "GANBARE_BUILD_NUMBER=$BUILD_NUMBER" \
-e "GANBARE_COMMIT_NAME=$COMMIT_NAME" \
-e "GANBARE_PARANOID=$PARANOID" \
-e "GANBARE_CACHE_MAX_AGE=$CACHE_MAX_AGE" \
-e "GANBARE_SERVER_THREADS=$SERVER_THREADS" \
-e "GANBARE_PERF_TRACE=$PERF_TRACE" \
-e "GANBARE_COOKIE_HMAC_KEY=$COOKIE_HMAC_KEY" \
-e "GANBARE_ENABLE_SOURCE_MAPS=$ENABLE_SOURCE_MAPS" \
-e "RUST_LOG=$LOGLEVEL" \
-v $ROOT_DIR/audio:/ganbare/audio \
-v $ROOT_DIR/images:/ganbare/images \
-v $ROOT_DIR/user_audio:/ganbare/user_audio \
golddranks/ganbare_run
EOF
