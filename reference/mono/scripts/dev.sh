#!/bin/bash

source ./scripts/.bashrc

./scripts/down.sh

files=$(compose_context_files "${1:-wordpress}" "true")

run_service_in_context "$files" "$1"
