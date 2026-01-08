#!/bin/bash

echo "Setting up Vector DB..."
make run s=WordPress c="cli seed setupVectorDb" d=$1

echo "Syncing Vector DB..."
make run s=WordPress c="cli seed syncVectorDb" d=$1
