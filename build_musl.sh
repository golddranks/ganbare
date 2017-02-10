#!/bin/sh

echo "$(( $(cat build_number.txt 2> /dev/null || echo "0") + 1 ))" > build_number.txt
rm target/x86_64-unknown-linux-musl/release/ganbare 2> /dev/null || true
docker rm -v ganbare_builder 2> /dev/null || true
./build_static.sh && \
docker run -it --name ganbare_builder -v $PWD:/workdir -e "BUILD_FEATURES=$BUILD_FEATURES" golddranks/ganbare_build && \
docker cp ganbare_builder:/etc/ssl/certs certs_temp && \
docker rm -v ganbare_builder && \
docker build --no-cache -f Dockerfile.run -t golddranks/ganbare_run . && \
rm -r certs_temp && \
docker push golddranks/ganbare_run
