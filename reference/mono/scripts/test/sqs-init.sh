#!/usr/bin/env bash
set -euo pipefail

AWS_ENDPOINT="http://localhost:4566"

QUEUE="campus-lms-local-critical"

DLQ="campus-lms-local-critical-dlq"
aws --endpoint-url $AWS_ENDPOINT sqs create-queue --queue-name "$DLQ"

DLQ_ARN=$(aws --endpoint-url $AWS_ENDPOINT sqs get-queue-attributes \
  --queue-url $(aws --endpoint-url $AWS_ENDPOINT sqs get-queue-url --queue-name "$DLQ" --query 'QueueUrl' --output text) \
  --attribute-names QueueArn --query 'Attributes.QueueArn' --output text)
aws --endpoint-url $AWS_ENDPOINT sqs create-queue --queue-name "$QUEUE" \
  --attributes RedrivePolicy="{\"deadLetterTargetArn\":\"$DLQ_ARN\",\"maxReceiveCount\":\"5\"}",ReceiveMessageWaitTimeSeconds="20",VisibilityTimeout="60"

echo "Created $QUEUE and $DLQ in LocalStack"
