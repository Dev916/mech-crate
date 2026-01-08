#!/bin/bash

docker run --rm -it --platform linux/x86_64 --workdir /workdir -v $(
    cd ~/
    pwd
)/.terraform.d:/root/.terraform.d -v $(pwd):/workdir -v $(pwd)/.ssh:/root/.ssh hashicorp/terraform:0.14.11 $@
