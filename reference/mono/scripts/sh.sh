#!/bin/bash

source ./scripts/.bashrc

files=$(compose_context_files "$1" "true")

docker compose $files exec $1 sh
