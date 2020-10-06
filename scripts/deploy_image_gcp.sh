#!/bin/sh

scripts/build_release.sh

docker tag accent_ganbare:latest_prd gcr.io/striped-rhino-291704/accent_ganbare:latest_prd
docker push gcr.io/striped-rhino-291704/accent_ganbare:latest_prd
