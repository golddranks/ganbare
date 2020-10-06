#!/bin/sh
set -eu

tsc
echo "TypeScript built."
sass src/sass/main.scss static/css/main.css
echo "SCSS built."
