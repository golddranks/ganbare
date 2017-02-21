#!/bin/sh

diesel print-schema > ganbare_backend/src/schema/specified_schema.rs
echo "$(( $(cat build_number.txt 2> /dev/null || echo "0") + 1 ))" > build_number.txt
rm target/x86_64-unknown-linux-musl/release/ganbare 2> /dev/null || true
docker rm -v ganbare_builder 2> /dev/null || true
scripts/build_static.sh && \
BUILD_BUILDTIME_PEPPER=$(cat .env.ganbare_buildtime_pepper)
docker run -it --name ganbare_builder -v $PWD:/workdir -e "BUILD_FEATURES=$BUILD_FEATURES" -e "GANBARE_BUILDTIME_PEPPER=$BUILD_BUILDTIME_PEPPER" golddranks/ganbare_build && \
docker cp ganbare_builder:/etc/ssl/certs certs_temp && \
docker rm -v ganbare_builder && \
docker build --no-cache -f scripts/Dockerfile.run -t golddranks/ganbare_run . && \
rm -r certs_temp && \
docker push golddranks/ganbare_run
