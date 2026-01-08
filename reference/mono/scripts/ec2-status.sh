#!/bin/bash

TAG_NAME=${1:-}
if [ -z "$TAG_NAME" ]; then
  # Query all instances if no tag name is provided
  aws ec2 describe-instances \
      --query 'Reservations[*].Instances[*].[InstanceId, State.Name, PublicIpAddress, PrivateIpAddress, Tags[?Key==`Name`].Value|[0]]' \
      --output table
else
  # Query instances with the specified tag name

  aws ec2 describe-instances \
      --query 'Reservations[*].Instances[*].[InstanceId, State.Name, PublicIpAddress, PrivateIpAddress, Tags[?Key==`Name`].Value|[0]]' \
      --output json | jq -r --arg substring "$TAG_NAME" '.[] | .[] | select(.[4] != null and (.[4] | test($substring))) | @tsv'
fi
