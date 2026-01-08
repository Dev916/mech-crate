#!/bin/bash

set +e

pwd

./scripts/dev.sh
./scripts/append.sh theblock.pro
./scripts/append.sh campus-lms
./scripts/append.sh campus-queue
./scripts/append.sh campus-sqs-queue
./scripts/append.sh sqs-monitoring
./scripts/sqs-init.sh high-priority
