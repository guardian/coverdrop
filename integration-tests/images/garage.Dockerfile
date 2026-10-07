# Garage's published image is `FROM scratch` and has no shell or curl in it,
# so we layer its (statically linked) binary onto Alpine to allow a HEALTHCHECK.
FROM alpine:3.20

RUN apk add --no-cache curl

COPY --from=dxflrs/garage:v2.3.0 /garage /garage
COPY integration-tests/images/garage.toml /etc/garage.toml

ENV RUST_LOG=garage=info
ENTRYPOINT ["/garage"]
CMD ["server"]

EXPOSE 3900 3901 3902 3903

HEALTHCHECK --start-period=10s --interval=1s --timeout=5s --retries=30 CMD curl -sf http://127.0.0.1:3903/health || exit 1

