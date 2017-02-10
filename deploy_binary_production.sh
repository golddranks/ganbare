#!/bin/sh
DEPLOY_STATIC_DIR=/srv/ganbare_production
DEPLOY_SERVER=ganba.re
DEPLOY_DB_NAME=ganbare_production
DEPLOY_LOCAL_PEPPER=$(cat .env.ganbare_testing_production_pepper)
DEPLOY_PORT=8000
DEPLOY_SITE_DOMAIN=testing.ganba.re
DEPLOY_SITE_LINK=https://akusento.ganba.re/
DEPLOY_EMAIL_DOMAIN=ganba.re
DEPLOY_EMAIL_SERVER=smtp.mailgun.org:587
DEPLOY_EMAIL_SMTP_USERNAME=postmaster@ganba.re
DEPLOY_EMAIL_SMTP_PASSWORD=$(cat .env.ganbare_email_password)
DEPLOY_BUILD_NUMBER="Build number: $(cat build_number.txt) Commit: $(git log HEAD --oneline --no-walk)"
DEPLOY_PARANOID=true

rsync -r images $DEPLOY_SERVER:$DEPLOY_STATIC_DIR/
rsync -r audio $DEPLOY_SERVER:$DEPLOY_STATIC_DIR/
ssh $DEPLOY_SERVER /bin/sh <<EOF
docker pull golddranks/ganbare_run
docker stop ganbare_runner_testing && docker rm ganbare_runner_testing
docker run -d --restart=unless-stopped \
--name ganbare_runner_testing \
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
-e "GANBARE_PARANOID=$DEPLOY_PARANOID" \
-e "RUST_LOG=ganbare=debug,ganbare_backend=debug" \
-v $DEPLOY_STATIC_DIR/audio:/ganbare/audio \
-v $DEPLOY_STATIC_DIR/images:/ganbare/images \
-v $DEPLOY_STATIC_DIR/user_audio:/ganbare/user_audio \
golddranks/ganbare_run
EOF
