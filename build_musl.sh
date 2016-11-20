#!/bin/sh

docker run -it --rm --name ganbare_builder -v $PWD:/ganbare golddranks/ganbare_build
