#!/bin/bash

# Detect if tmux is installed
if ! [ -x "$(command -v tmux)" ]; then
    echo "Error: tmux is not installed. (ueg: brew install tmux)" >&2
    exit 1
fi

# Services
services=("redis" "redis-ro" "postgres" "mysql" "haproxy" "WordPress")
session_name=log-panel-session

# Start a new tmux session
tmux new-session -d -s $session_name -n logs

# Create the first row of three panes
tmux send-keys "make logs service=${services[0]}" C-m
tmux split-window -h -t $session_name:0
tmux send-keys "make logs service=${services[1]}" C-m
tmux split-window -h -t $session_name:0
tmux send-keys "make logs service=${services[2]}" C-m

# Select the layout to be tiled evenly
tmux select-layout even-horizontal

# Split each of the top three panes vertically to create the bottom row
tmux select-pane -t 0
tmux split-window -v -t $session_name:0.0
tmux send-keys "make logs service=${services[3]}" C-m

tmux select-pane -t 2
tmux split-window -v -t $session_name:0.2
tmux send-keys "make logs service=${services[4]}" C-m

tmux select-pane -t 4
tmux split-window -v -t $session_name:0.4
tmux send-keys "make logs service=${services[5]}" C-m

# Attach to the session
tmux attach-session -t $session_name
