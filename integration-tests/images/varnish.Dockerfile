FROM varnish:6.0

RUN apt-get update && apt-get install -y --no-install-recommends curl && rm -rf /var/lib/apt/lists/*

HEALTHCHECK --start-period=30s --interval=1s --timeout=5s --retries=3 CMD curl -s -o /dev/null http://127.0.0.1:80/ || exit 1
