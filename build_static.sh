#!/bin/sh
tsc
echo "TypeScript built."
sass static/assets/sass/main.scss static/assets/css/main.css
echo "SCSS built."
