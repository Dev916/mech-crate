#!/bin/bash

files=$(cat tmp/up/*.txt)

docker compose $files logs -f --tail=1000 $1
