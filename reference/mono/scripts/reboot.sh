#!/bin/bash

if [ -z "$1" ]
  then
    echo "No argument supplied"
    exit 1
fi


docker compose stop -t0 $1
docker compose rm -f $1
docker compose create $1
docker compose start $1