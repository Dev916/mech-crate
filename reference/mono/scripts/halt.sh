#!/bin/bash

# Stop all running Docker containers

# Get a list of all running container IDs
container_ids=$(docker ps -q)

# Check if there are any running containers
if [ -n "$container_ids" ]; then
  echo "Stopping all running Docker containers..."
  
  # Stop all running containers
  docker stop $container_ids
  docker rm $container_ids
  
  echo "All running Docker containers have been stopped and removed."
else
  echo "No running Docker containers found."
fi