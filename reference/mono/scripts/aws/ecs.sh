#!/bin/sh

# Check if a service name is provided as an argument
if [ -z "$1" ]; then
    echo "Usage: $0 <service-name> [extra-args]"
    echo "Example: $0 my-service --profile prod"
    exit 1
fi

REGION=us-east-1  # Set your region here
CLUSTER=prod-rp-latest
SERVICE="${1}"
shift
EXTRA_ARGS="--profile prod"

# Check if the cluster exists
if ! aws ecs describe-clusters --clusters "${CLUSTER}" --query 'clusters[0].status' --output text --region "${REGION}" $EXTRA_ARGS | grep -q 'ACTIVE'; then
    echo "Error: Cluster '${CLUSTER}' not found or not active."
    exit 1
fi

# Get the list of running tasks
TASK_ARN=$(aws ecs list-tasks --cluster "${CLUSTER}" --service-name "${CLUSTER}-${SERVICE}" --desired-status RUNNING --query 'taskArns' --output text --region "${REGION}" $EXTRA_ARGS)

# Check if any tasks were found
if [ -z "$TASK_ARN" ]; then
    echo "Error: No running tasks found for service '${SERVICE}' in cluster '${CLUSTER}'."
    exit 1
fi

# Get details for all tasks
TASKS_JSON=$(aws ecs describe-tasks --cluster "${CLUSTER}" --tasks ${TASK_ARN} --region "${REGION}" $EXTRA_ARGS)

# Parse and display tasks
echo "\nRunning tasks for service '${SERVICE}':\n"
TASK_LIST=$(echo "$TASKS_JSON" | jq -r '.tasks | sort_by(.startedAt) | reverse | to_entries | .[] | "\(.key + 1). Task: \(.value.taskArn | split("/") | last) | Started: \(.value.startedAt) | Status: \(.value.lastStatus)"')

echo "$TASK_LIST"

# Count number of tasks
TASK_COUNT=$(echo "$TASKS_JSON" | jq '.tasks | length')

# If only one task, use it automatically
if [ "$TASK_COUNT" -eq 1 ]; then
    SELECTED_TASK_ARN=$(echo "$TASKS_JSON" | jq -r '.tasks[0].taskArn')
    echo "\nOnly one task found, connecting automatically..."
else
    # Prompt user to select a task
    echo "\nSelect a task (1-${TASK_COUNT}): "
    read SELECTION
    
    # Validate selection
    if ! [ "$SELECTION" -ge 1 ] 2>/dev/null || ! [ "$SELECTION" -le "$TASK_COUNT" ] 2>/dev/null; then
        echo "Error: Invalid selection. Please enter a number between 1 and ${TASK_COUNT}."
        exit 1
    fi
    
    # Get the selected task ARN (jq arrays are 0-indexed)
    SELECTED_TASK_ARN=$(echo "$TASKS_JSON" | jq -r ".tasks | sort_by(.startedAt) | reverse | .[$(($SELECTION - 1))].taskArn")
fi

# Check if the selected task ARN was retrieved
if [ -z "$SELECTED_TASK_ARN" ]; then
    echo "Error: Could not retrieve the selected task ARN."
    exit 1
fi

echo "Connecting to task: $(echo $SELECTED_TASK_ARN | sed 's/.*\///')\n"

# Execute the command on the selected running task
aws ecs execute-command --region "${REGION}" --cluster "${CLUSTER}" --task "$SELECTED_TASK_ARN" --container "${CLUSTER}-${SERVICE}" --command "/bin/sh" --interactive $EXTRA_ARGS