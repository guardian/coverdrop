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
RUN cargo chef cook --recipe-path recipe.json --bin u2j-appender

# the rest is dependent on our code
COPY . .

# build the binary
ARG CARGO_BUILD_PROFILE="dev"
RUN cargo build --profile $CARGO_BUILD_PROFILE --bin u2j-appender

# save the binary and then clean target folder
RUN mkdir /dist;
RUN cp target/debug/u2j-appender target/release/u2j-appender /dist 2>/dev/null | true;

#
# RUN
#
FROM ubuntu:24.04

RUN apt-get update && \
    apt-get install -y ca-certificates && \
    update-ca-certificates

WORKDIR /usr/src/app/

COPY --from=builder /dist/u2j-appender ./u2j-appender

EXPOSE 3040

CMD ["./u2j-appender"]
