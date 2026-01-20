FROM lukemathwalker/cargo-chef:latest-rust-1.92.0 AS chef
WORKDIR /usr/src/app

#
# PLAN (learn dependencies)
#
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

#
# BUILD
#
FROM chef AS builder

# pre-build dependencies (and cache!)
COPY --from=planner /usr/src/app/recipe.json recipe.json
RUN cargo chef cook --recipe-path recipe.json --bin covernode

# everything below now only rebuilds our actual code
COPY . .

# build the binary
ARG CARGO_BUILD_PROFILE="dev"
RUN cargo build --profile ${CARGO_BUILD_PROFILE} --bin covernode

# save the binary and then clean target folder
RUN mkdir exec;
RUN cp target/debug/covernode target/release/covernode exec 2>/dev/null | true;
RUN rm -rf target

#
# RUN
#
FROM ubuntu:24.04

RUN apt-get update && \
    apt-get install -y ca-certificates && \
    update-ca-certificates

WORKDIR /usr/src/app/

COPY --from=builder /usr/src/app/exec/ .

EXPOSE 4444
