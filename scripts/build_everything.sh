#!/bin/sh
scripts/build_static.sh && \
cargo build && \
echo "Backend built."
