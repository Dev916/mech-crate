#!/bin/bash

# List all sessions
sessions=$(tmux list-sessions -F "#{session_name}")

for session in $sessions; do
    echo "Session: $session"
    # List all windows in the session
    windows=$(tmux list-windows -t $session -F "#{window_index}: #{window_name}")

    for window in $windows; do
        echo "  Window: $window"
        # List all panes in the window
        panes=$(tmux list-panes -t $session:$(echo $window | cut -d':' -f1) -F "#{pane_index}: #{pane_title}")
        for pane in $panes; do
            echo "    Pane: $pane"
        done
    done
done
