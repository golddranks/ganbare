#!/bin/sh

docker run -it --rm --name ganbare_builder -v $PWD:/workdir golddranks/ganbare_build && \
docker build --no-cache -f Dockerfile.run -t golddranks/ganbare_run . && \
docker push golddranks/ganbare_run
