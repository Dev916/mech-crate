#!/bin/bash
# Set the default region and profile
REGION=us-east-1
PROFILE=prod
# Default cluster name
CLUSTER_NAME=${1:-prod-rp-latest}

# Check if the cluster is active
cluster_status=$(aws ecs describe-clusters --clusters $CLUSTER_NAME --region $REGION --profile $PROFILE | jq -r '.clusters[0].status')
if [ "$cluster_status" != "ACTIVE" ]; then
echo "Cluster $CLUSTER_NAME is not active."
exit 1
fi
# role_credentials=$(aws sts assume-role --role-arn "arn:aws:iam::<account-id>:role/stage-admin" --role-session-name "ECSListSession" --profile $PROFILE)
# export AWS_ACCESS_KEY_ID=$(echo $role_credentials | jq -r '.Credentials.AccessKeyId')
# export AWS_SECRET_ACCESS_KEY=$(echo $role_credentials | jq -r '.Credentials.SecretAccessKey')
# export AWS_SESSION_TOKEN=$(echo $role_credentials | jq -r '.Credentials.SessionToken')

# List running ECS tasks
task_arns=$(aws ecs list-tasks --cluster $CLUSTER_NAME --desired-status RUNNING --region $REGION --profile $PROFILE --query 'taskArns' --output text)

if [ -z "$task_arns" ]; then
echo "No running tasks found."
exit 0
fi

task_data=()
for task_arn in $task_arns; do
task_details=$(aws ecs describe-tasks --cluster $CLUSTER_NAME --tasks $task_arn --region $REGION --profile $PROFILE)
task_name=$(echo $task_details | jq -r '.tasks[0].taskDefinitionArn' | awk -F'/' '{print $2}')
cpu=$(echo $task_details | jq -r '.tasks[0].overrides.containerOverrides[0].cpu // "N/A"')
memory=$(echo $task_details | jq -r '.tasks[0].overrides.containerOverrides[0].memory // "N/A"')
container_instance_arn=$(echo $task_details | jq -r '.tasks[0].containerInstanceArn')

instance_details=$(aws ecs describe-container-instances --cluster $CLUSTER_NAME --container-instances $container_instance_arn --region $REGION --profile $PROFILE)
instance_count=$(echo $instance_details | jq -r '.containerInstances[0].runningTasksCount')

task_data+=("$task_name $cpu $memory $instance_count")
done

printf "%-20s %-10s %-10s %-15s\n" "Task Name" "CPU" "Memory" "Instance Count"
printf "%-20s %-10s %-10s %-15s\n" "--------- " "---" "------" "--------------"
for data in "${task_data[@]}"; do
printf "%-20s %-10s %-10s %-15s\n" $data
done

