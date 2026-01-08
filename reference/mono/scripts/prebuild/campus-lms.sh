#!/bin/bash

rm -rf apps/campus-lms/node_modules
echo "node_modules removed"

rm -rf apps/campus-lms/vendor
echo "vendor removed"

rm -rf apps/campus-lms/bootstrap/cache/*.php
echo "bootstrap cache removed"

if docker volume ls | grep compose_campus-lms; then
    docker volume ls -q | grep "campus-lms" | xargs -r docker volume rm
fi
