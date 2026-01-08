#!/bin/bash

files=$(cat tmp/up/*.txt)

docker compose $files pull $1
docker compose $files stop $1
docker compose $files rm -f $1
docker compose $files create $1
docker compose $files start $1