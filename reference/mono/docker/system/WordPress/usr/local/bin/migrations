#!/bin/bash

cd /srv/wp/web

/srv/wp/vendor/bin/wp --allow-root --url=/ migrations ${1:-list} ${@:2:99}
