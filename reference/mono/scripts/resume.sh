#!/bin/bash

source ./scripts/.bashrc

./scripts/down.sh

files=$(cat docker/context/up.txt)

run_service_in_context "$files"
