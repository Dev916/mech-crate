#!/bin/bash

git stash

git fetch --all

git rebase origin/main

git push --force

git stash pop