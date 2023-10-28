
# build some things
FROM docker.io/library/debian:latest AS build

RUN apt-get update && apt-get -y install gcc g++ make libexpat1-dev libyaml-dev \
    libssl-dev zlib1g-dev gnuplot-nox erlang-nox erlang-dev git && \
    mkdir -p /build/ && \
    git clone https://github.com/processone/rtb /build/ && rm -rf /build/rtb/.git && \
    chown -R nobody: /build/ && ls -lah /build/ && \
    cd /build/ && su -m -s /bin/bash nobody -c make && \
    find -type d -name .git -print0 | xargs -0 rm -rf && cp rtb.yml.xmpp.example rtb.yml

FROM docker.io/library/debian:latest

RUN apt-get update && apt-get -y install gnuplot-nox erlang-nox libyaml-0-2 && rm -rf /var/lib/apt/lists

COPY --from=build /build/cert.pem /build/rtb.sh /build/rtb.yml /app/
COPY --from=build /build/deps /app/deps
COPY --from=build /build/ebin /app/ebin
COPY --from=build /build/priv /app/priv
#COPY --from=build /build /app/all

WORKDIR /app
ENTRYPOINT /app/rtb.sh
