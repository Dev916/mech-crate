#!/bin/bash

if [ ! -f "./tmp/start.txt" ]; then
    echo "Start file not found!"
    exit 1
fi

services=$(cat ./tmp/start.txt)

if [ -z "$services" ]; then
    echo "No services to update!"
    exit 1
fi

echo "Pruning docker images..."

docker image prune -a -f

# Update the latest code for each service
for service in $services; do
    echo "Pulling latest code for $service..."
    ./scripts/pull.sh $service &
done

wait

echo "Restarting the stack..."
./scripts/down.sh
./scripts/start.sh
