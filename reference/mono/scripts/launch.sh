#!/bin/bash

set +e

service=$1

if [ -z "$service" ]; then
  echo "Usage: $0 <service>"
  exit 1
fi

if [ ! -f scripts/launch/$service.sh ]; then
  echo "Service $service not found"
  exit 1
fi

./scripts/launch/$service.sh
