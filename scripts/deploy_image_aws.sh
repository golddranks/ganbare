#!/bin/sh

scripts/build_release.sh

docker tag accent_ganbare:latest_prd 437938062036.dkr.ecr.eu-north-1.amazonaws.com/accent-ganbare-web:latest_prd
docker push 437938062036.dkr.ecr.eu-north-1.amazonaws.com/accent-ganbare-web:latest_prd
