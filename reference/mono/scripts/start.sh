#!/bin/bash

if [ ! -f "./tmp/start.txt" ]; then
    echo "Start file not found!"
    exit 1
fi

services=$(cat ./tmp/start.txt)

if [ -z "$services" ]; then
    echo "No services to start!"
    exit 1
fi

make up

for service in $services; do
    make up s=$service
done
