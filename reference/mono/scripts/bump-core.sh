#!/bin/bash

cd core

git checkout main

git pull

cd ..

git add core

git commit -m "chore(core): bump lp-core"

#git push
