# Integration test images

These should be kept largely up to date with their matching Dockerfile under `infra` except with an ubuntu based runtime.

This is to help when debugging, since it allows the developer to open a shell in a running container. Chainguard images used in production lack a shell, and can only be debugged with great difficulty using Kubernetes ephemeral containers.
