FROM lukemathwalker/cargo-chef:latest AS chef
WORKDIR /usr/src/app

#
# PLAN (learn dependencies)
#
FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

#
# BUILD
#
FROM chef as builder

# pre-build dependencies (and cache!)
COPY --from=planner /usr/src/app/recipe.json recipe.json
RUN cargo chef cook --recipe-path recipe.json --bin identity-api

# the rest is dependent on our code
COPY . .

# build the binary
ARG CARGO_BUILD_PROFILE="dev"
RUN cargo build --profile $CARGO_BUILD_PROFILE --bin identity-api

# save the binary and then clean target folder
RUN mkdir exec;
RUN cp target/debug/identity-api target/release/identity-api exec 2>/dev/null | true;
RUN rm -rf target


#
# RUN
#
FROM ubuntu:24.04

WORKDIR /usr/src/app/

RUN apt-get update && \
    apt-get install -y ca-certificates && \
    update-ca-certificates

COPY --from=builder /usr/src/app/exec/ .

EXPOSE 3010
EXPOSE 4444

CMD ["./identity-api"]
