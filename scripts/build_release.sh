#!/bin/sh
set -eu

docker build -f scripts/Dockerfile -t accent_ganbare:latest_prd --build-arg BUILDTIME_PEPPER="$(cat .env.buildtime_pepper)" .