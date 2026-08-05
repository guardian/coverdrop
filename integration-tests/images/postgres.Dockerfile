FROM postgres:14.5

HEALTHCHECK --start-period=30s --interval=1s --timeout=5s --retries=3 CMD pg_isready || exit 1


