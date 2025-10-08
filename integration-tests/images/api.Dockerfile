FROM lukemathwalker/cargo-chef:latest-rust-1.90.0 AS chef
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
RUN cargo chef cook --recipe-path recipe.json --bin api

# the rest is dependent on our code
COPY . .

# build the binary
ARG CARGO_BUILD_PROFILE="dev"
RUN cargo build --profile $CARGO_BUILD_PROFILE --bin api

# save the binary and then clean target folder
RUN mkdir /dist;
RUN cp target/debug/api target/release/api /dist 2>/dev/null | true;

#
# RUN
#
FROM ubuntu:24.04

RUN apt-get update

RUN apt-get update && \
    apt-get install -y ca-certificates && \
    update-ca-certificates

WORKDIR /usr/src/app/

COPY --from=builder /dist/api ./api

EXPOSE 3000
EXPOSE 4444

CMD ["./api"]
