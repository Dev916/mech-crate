#!/bin/bash

# Default port
PORT=${1:-9090}

NC_CMD=$(./scripts/utils/nc_resolver.sh $PORT)

# Function to handle cleanup on exit
cleanup() {
    echo "Stopping the server..."
    exit 0
}

# Trap SIGINT (Ctrl+C) to run the cleanup function
trap cleanup SIGINT

# Function to get Docker container stats
get_docker_stats() {
    echo "<h2>Docker Container Stats</h2>"
    echo "<pre>"
    docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}\t{{.BlockIO}}"
    echo "</pre>"
}

# Function to get system disk usage
get_disk_usage() {
    echo "<h2>Disk Usage</h2>"
    echo "<pre>"
    df -h
    echo "</pre>"
}

# Function to get system memory usage
get_memory_usage() {
    echo "<h2>Memory Usage</h2>"
    echo "<pre>"
    if command -v free &> /dev/null; then
        free -h
    else
        # macOS alternative using vm_stat
        vm_stat | awk '
        BEGIN { FS=":"; total=0; free=0; }
        /Pages free/ { free=$2 }
        /Pages active/ { total+=$2 }
        /Pages inactive/ { total+=$2 }
        /Pages speculative/ { total+=$2 }
        /Pages wired down/ { total+=$2 }
        /Pages occupied by compressor/ { total+=$2 }
        END {
            total=total*4096/1024/1024;
            free=free*4096/1024/1024;
            printf "Total: %.2f MB\nFree: %.2f MB\n", total, free
        }'
    fi
    echo "</pre>"
}

# Function to get system CPU usage
get_cpu_usage() {
    echo "<h2>CPU Usage</h2>"
    echo "<pre>"
    if [[ "$OSTYPE" == "darwin"* ]]; then
        top -l 1 | awk '/CPU usage/ {print $0}'
    else
        top -bn1 | grep "Cpu(s)"
    fi
    echo "</pre>"
}

echo "Starting the server on port $PORT..."
echo "Visit http://localhost:$PORT to see the system stats."
echo "Press Ctrl+C to stop the server."

# Start a simple HTTP server
while true; do
    {
        echo -e "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n"
        echo "<html>"
        echo "<head>"
        echo "<meta http-equiv=\"refresh\" content=\"5\">"
        echo "<title>System Stats</title>"
        echo "<style>"
        echo "body { font-family: Arial, sans-serif; margin: 20px; }"
        echo "pre { background: #f5f5f5; padding: 10px; border-radius: 5px; }"
        echo "h1, h2 { color: #333; }"
        echo "</style>"
        echo "</head>"
        echo "<body>"
        echo "<h1>System Stats</h1>"
        echo "<p>Last updated: $(date)</p>"
        get_docker_stats
        get_disk_usage
        get_memory_usage
        get_cpu_usage
        echo "</body></html>"
    } | eval $NC_CMD
done
