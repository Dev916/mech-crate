#!/bin/bash
set -e

source ./scripts/.bashrc

files=$(compose_context_files sqs true)

docker compose $files exec -T sqs /bin/bash -c "python /load/consume_sqs.py"
