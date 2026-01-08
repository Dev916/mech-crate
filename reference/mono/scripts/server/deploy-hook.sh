#!/bin/bash

# Configuration
DEPLOY_HOOK_PORT=${1:-9001}
NC_CMD=$(./scripts/utils/nc_resolver.sh $DEPLOY_HOOK_PORT)
LOG_FILE="deploy-hook.log"

cleanup() {
    echo "Stopping the server..."
    exit 0
}

# Trap SIGINT (Ctrl+C) to run the cleanup function
trap cleanup SIGINT
SECURITY_KEY=$(./scripts/utils/keygen.sh -l 256 -f hex)
echo "Starting deploy hook server on port $DEPLOY_HOOK_PORT..."
echo "Security key: $SECURITY_KEY"
echo "Log file: $LOG_FILE"

echo "Test with: curl -X POST \"http://localhost:$DEPLOY_HOOK_PORT/?key=$SECURITY_KEY\""

while true; do
    eval $NC_CMD | while read line; do
        # Skip empty lines
        if [[ -z "$line" ]]; then
            continue
        fi
        
        # Skip lines that don't look like HTTP requests
        if [[ ! "$line" =~ ^(GET|POST|PUT|DELETE|PATCH|HEAD|OPTIONS) ]]; then
            continue
        fi
        
        # Log the request
        echo "$(date): Received request: $line" >> $LOG_FILE
        
        # Check if it's a POST request and extract query parameters
        if [[ "$line" == *"POST"* ]]; then
            # Extract the security key from query parameters
            received_key=$(echo "$line" | grep -o "key=[^[:space:]]*" | cut -d= -f2)
            
            if [ "$received_key" = "$SECURITY_KEY" ]; then
                # Send success response immediately
                echo -e "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\nWebhook received, deploying in background.\r\n"
                
                # Execute make update in background
                (
                    echo "Starting deployment at $(date)" >> $LOG_FILE
                    make update >> $LOG_FILE 2>&1
                    echo "Deployment completed at $(date)" >> $LOG_FILE
                ) &
                
                break
            else
                echo -e "HTTP/1.1 401 Unauthorized\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\nInvalid security key"
                break
            fi
        else
            echo -e "HTTP/1.1 405 Method Not Allowed\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\nOnly POST requests are allowed"
            break
        fi
    done
done 