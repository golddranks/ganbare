#!/bin/sh

docker rm -v ganbare_builder
./build_static.sh && \
docker run -it --name ganbare_builder -v $PWD:/workdir golddranks/ganbare_build && \
docker cp ganbare_builder:/etc/ssl/certs certs_temp && \
docker rm -v ganbare_builder && \
docker build --no-cache -f Dockerfile.run -t golddranks/ganbare_run . && \
rm -r certs_temp && \
docker push golddranks/ganbare_run
