#!/bin/bash

source scripts/.bashrc

files=$(compose_context_files "base" "true")

docker compose $files ps
