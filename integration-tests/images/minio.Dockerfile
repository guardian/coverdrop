FROM minio/minio:RELEASE.2025-09-07T16-13-09Z

HEALTHCHECK --start-period=30s --interval=1s --timeout=5s --retries=3 CMD curl -f http://127.0.0.1:9000/minio/health/live || exit 1


