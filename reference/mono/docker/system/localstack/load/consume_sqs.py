#!/usr/bin/env python3
import os
import time

import boto3

queue_name = os.environ.get("SQS_CRITICAL_QUEUE", "critical")
endpoint_url = os.environ.get("SQS_ENDPOINT", "http://localhost:4566")
region_name = os.environ.get("AWS_REGION", "us-east-1")

sqs = boto3.client(
    "sqs",
    endpoint_url=endpoint_url,
    region_name=region_name,
    aws_access_key_id="test",
    aws_secret_access_key="test",
)

queue_url = sqs.get_queue_url(QueueName=queue_name)["QueueUrl"]

print(f"[*] Listening to {queue_name} at {queue_url}")

while True:
    messages = sqs.receive_message(
        QueueUrl=queue_url,
        MaxNumberOfMessages=10,
        WaitTimeSeconds=5,
    ).get("Messages", [])

    if not messages:
        time.sleep(1)
        continue

    for msg in messages:
        body = msg["Body"]
        time.sleep(0.1)
        _ = sqs.delete_message(
            QueueUrl=queue_url,
            ReceiptHandle=msg["ReceiptHandle"]
        )
        print(f"[√] Message processed: {body}")
