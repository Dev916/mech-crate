#!/bin/sh

mkdir -p ./tmp/up

./scripts/submodules.sh

./scripts/run.sh WordPress import-data
