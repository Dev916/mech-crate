#!/bin/bash

# Read input from argument or stdin
if [ -t 0 ] && [ $# -gt 0 ]; then
  # If input is from terminal and an argument is passed
  query="$1"
else
  # Otherwise read from stdin
  query=$(cat)
fi

echo "Running query: $query"

file_name="reader-output-$(date +%Y-%m-%d-%H-%M-%S).txt"

PGPASSWORD="Z1Rb4h1wF8P8-ykD8MpW2RE-3jm910sh" /usr/bin/psql -h 172.31.26.228 -U theblock campus -c  "$query" > $file_name

echo "Output saved to $file_name"
