# Production Images

For our dev and production images we use frequently rebuilt and lightweight docker containers based off of the [wolfi undistro](https://www.chainguard.dev/unchained/introducing-wolfi-the-first-linux-un-distro).

## Building

For the images actually deployed to the production stack we use GitHub Actions which can be found in `.github/workflows/build-prod-images.yaml`.

If you wish to test changes to the local production `Dockerfile`s then check the scripts in the `infra/{cloud,on-premises}/scripts/build` directory or you can build them all using `infra/scripts/build-all.sh`.

## Vulnerability scanning

We scan the build time dependencies for our docker images.
As we do multi stage builds, we check any intermediary build dependencies that were used to produce the final image, even if they were not included in the final output.

We have SBOM generation and vulnerability scanning to our Docker images.

We have opted to produce Software Bill of Materials (SBOM) for each of the build steps in our Docker image.

This is done with the inbuilt support from [BuildKit](https://www.docker.com/blog/generate-sboms-with-buildkit/)

We generate in our final image by using:
`docker buildx build --sbom=true ...`
We generate SBOMs for intermediary steps by using the following build arg in our Docker files:

```
ARG BUILDKIT_SBOM_SCAN_CONTEXT=true
ARG BUILDKIT_SBOM_SCAN_STAGE=true
```

These SBOMs are then available for [querying](https://www.docker.com/blog/generate-sboms-with-buildkit/#analyzing-images) using the `docker buildx imagetools inspect` command.

We extract these SBOMs as part of the build process.

## Publish the SBOMs

We can attach the SBOMs as [workflow artifacts](https://docs.github.com/en/actions/using-workflows/storing-workflow-data-as-artifacts)
<img width="1020" alt="Screenshot 2024-01-30 at 10 41 42" src="https://github.com/guardian/coverdrop/assets/1289259/77b91a63-8cf3-40e2-af0a-756950d060d5">

Each Docker build can have up to 4 stages, so we can end up with 5 SBOMs per docker image. All these SBOMS are attached as build artefacts to the build action.

## Scanning the SBOMS

We [then Scan the SBOMs with `grype`](https://github.com/anchore/grype) to find any vulnerabilities.
We then uploading the SARIF scanning results from `grype` as artifacts to the build output.
<img width="1157" alt="Screenshot 2024-01-30 at 10 29 11" src="https://github.com/guardian/coverdrop/assets/1289259/e5344036-0b55-4439-a7f5-89c555e053f1">

We also push the results as a warning on the build output if there are failures
Publish the SBOM
<img width="870" alt="Screenshot 2024-01-30 at 10 29 51" src="https://github.com/guardian/coverdrop/assets/1289259/9e35497b-12e4-4697-84b2-2e0a81077fdc">

We also submit the SARIF files to `github/codeql-action/upload-sarif` [CodeQuality Submission api ](https://docs.github.com/en/code-security/code-scanning/integrating-with-code-scanning/uploading-a-sarif-file-to-github#uploading-a-code-scanning-analysis-with-github-actions) These results then show up in the [code scanning results](https://github.com/guardian/coverdrop/security/code-scanning)

<img width="1800" alt="Screenshot 2024-01-30 at 10 14 11" src="https://github.com/guardian/coverdrop/assets/1289259/aa4552c1-d4de-4b16-ad72-e845e3bcba5f">

## Future work

We can update this action to run on pull requests, and trigger dependency review on PRs (ie run the docker build steps, and publish any findings, but don’t publish the docker files) - https://github.com/actions/dependency-review-action

## Testing

We used the [`scanner-test` images provided by `chainguard`](https://github.com/chainguard-dev/vulnerability-scanner-support/blob/main/docs/verifying_scan_results.md) to verify the vulnerability scans worked correctly

`ghcr.io/chainguard-images/scanner-test:unfixed-vulnerabilities-many-chainguard`
