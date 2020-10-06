#!/bin/sh

scripts/build_release.sh

docker tag accent_ganbare:latest_prd gcr.io/striped-rhino-291704/accent_ganbare:latest_prd
docker push gcr.io/striped-rhino-291704/accent_ganbare:latest_prd
gcloud run deploy accent-ganbare-web --image gcr.io/striped-rhino-291704/accent_ganbare:latest_prd --platform managed --region europe-north1
