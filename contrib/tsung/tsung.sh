#!/bin/sh
set -eux

podman build . -t tsung

# podman run --rm --entrypoint tar tsung -C /usr/share/doc/tsung/examples -cf - . > tsungexamples.tar

rm -rf tsung-logs; mkdir -p tsung-logs; podman run --network host --rm -it -p 8091:8091 --volume $(pwd)/tsung-logs:/root/.tsung/log --volume $(pwd)/tsung.xml:/root/.tsung/tsung.xml:ro --entrypoint tsung tsung start

