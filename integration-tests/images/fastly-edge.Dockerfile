FROM denoland/deno:ubuntu-1.40.5 AS deno

RUN apt-get update
RUN apt-get install unzip
RUN mkdir fastly-edge
COPY fastly-edge fastly-edge

RUN cd fastly-edge

RUN deno compile --allow-net fastly-edge/forwarder.ts

FROM rust:latest AS rust
WORKDIR /usr/src/app

COPY . .

WORKDIR /usr/src/app/fastly-edge

# Download the Fastly CLI and Viceroy which is the small runtime that serves the wasm bundle in the final docker image
# Supports both ARM (AArch64) and x86_64
RUN /bin/bash -c 'set -ex &&                                                                                   \
    ARCH=$(uname -m) &&                                                                                        \
    if [[ "$ARCH" == "aarch64" ]]; then                                                                        \
    echo "Detected ARM architecture" &&                                                                    \
    wget "https://github.com/fastly/cli/releases/download/v10.2.0/fastly_v10.2.0_linux-arm64.tar.gz" &&    \
    wget "https://github.com/fastly/Viceroy/releases/download/v0.5.1/viceroy_v0.5.1_linux-arm64.tar.gz" && \
    tar -xzf fastly_v10.2.0_linux-arm64.tar.gz &&                                                          \
    tar -xzf viceroy_v0.5.1_linux-arm64.tar.gz;                                                            \
    else                                                                                                       \
    echo "Assuming x86_64 architecture" &&                                                                 \
    wget "https://github.com/fastly/cli/releases/download/v10.2.0/fastly_v10.2.0_linux-amd64.tar.gz" &&    \
    wget "https://github.com/fastly/Viceroy/releases/download/v0.5.1/viceroy_v0.5.1_linux-amd64.tar.gz" && \
    tar -xzf fastly_v10.2.0_linux-amd64.tar.gz &&                                                          \
    tar -xzf viceroy_v0.5.1_linux-amd64.tar.gz;                                                            \
    fi'

# Build the WebAssembly binary
RUN ./fastly compute build

FROM ubuntu:22.04

WORKDIR /opt/fastly-edge

# Viceroy issues a warning if there aren't any CA certificates.
RUN apt-get update
RUN apt-get install curl ca-certificates -y
RUN update-ca-certificates

COPY --from=deno forwarder ./forwarder

COPY --from=rust /usr/src/app/fastly-edge/viceroy     ./viceroy
COPY --from=rust /usr/src/app/fastly-edge/fastly      ./fastly
COPY --from=rust /usr/src/app/fastly-edge/fastly.toml ./fastly.toml
COPY --from=rust /usr/src/app/fastly-edge/bin         ./bin

EXPOSE 7676
