#!/bin/sh
set -eux

podman build . -t rtb

podman run --network host --rm -it -p 8080:8080 --volume $(pwd)/rtb.yml:/app/rtb.yml:ro rtb
