#!/bin/bash

files=$(cat tmp/up/*.txt)

docker compose $files exec $1 $2
