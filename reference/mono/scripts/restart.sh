#!/bin/bash

if [ -z "$1" ]; then
  echo "No argument supplied"
  exit 1
fi

./scripts/down.sh $1
./scripts/append.sh $1
