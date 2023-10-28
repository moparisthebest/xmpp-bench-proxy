
# build some things
FROM docker.io/library/debian:latest

RUN apt-get update && apt-get -y install tsung && rm -rf /var/lib/apt/lists

RUN ln -sf /usr/lib/x86_64-linux-gnu/tsung/bin/* /usr/bin/

WORKDIR /root
ENTRYPOINT /usr/bin/tsung
CMD start
