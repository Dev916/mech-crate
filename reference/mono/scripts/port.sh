#!/bin/bash

if [ -z "$1" ]; then
    echo "Usage: $0 <service>"
    exit 1
fi


cp -r docker/system/$1/* apps/$1/docker/system/$1/
