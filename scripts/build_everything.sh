#!/bin/sh
./build_static.sh && \
cargo build && \
echo "Backend built."
