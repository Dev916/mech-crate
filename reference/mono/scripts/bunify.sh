#!/bin/sh

NUXT_EXEC=./node_modules/nuxt/bin/nuxt.js

sed -i '' 's|#!/usr/bin/env node|#!/usr/bin/env bun|' $NUXT_EXEC
sed -i '' 's|../package.json|/app/package.json|g' $NUXT_EXEC