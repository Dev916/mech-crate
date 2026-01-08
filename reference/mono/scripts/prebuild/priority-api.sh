#!/bin/bash

if [ -d 'apps/priority-api/target' ]; then
    echo "Priority API target directory exists"
    rm -rf apps/priority-api/target
    echo "Priority API target directory removed 🧹🧹🧹"
fi

if [ -n 'apps/priority-api/priority-api-docs/target' ]; then
    echo "Priority API docs target directory exists"
    rm -rf apps/priority-api/priority-api-docs/target
    echo "Priority API docs target directory removed 🧹🧹🧹"
fi


