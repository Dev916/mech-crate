#!/bin/bash

# Function to get the appropriate netcat command for the current platform

PORT=${1:-8080}
    
if [[ -z "$PORT" ]]; then
    echo "Error: Port number is required" >&2
    return 1
fi

if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    echo "nc -l $PORT"
else
    # Linux
    echo "nc -l -p $PORT"
fi