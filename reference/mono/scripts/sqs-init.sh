#!/bin/bash
set -e

source ./scripts/.bashrc

echo "Creating SQS queues in LocalStack..."

files=$(compose_context_files sqs true)

docker compose $files exec -T sqs awslocal sqs create-queue --queue-name $1
docker compose $files exec -T sqs awslocal sqs create-queue --queue-name $1-dlq --attributes '{
    "RedrivePolicy":"{\"deadLetterTargetArn\":\"arn:aws:sqs:us-east-1:000000000000:$1-dlq\",\"maxReceiveCount\":\"3\"}"
  }'

echo "Queues created correctly"

docker compose $files exec -T sqs awslocal sqs list-queues
