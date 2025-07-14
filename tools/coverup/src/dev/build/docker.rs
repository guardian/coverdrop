use std::{net::Ipv4Addr, path::Path, process::Stdio};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt as _, BufReader},
    process,
};

use crate::{
    coverdrop_service::CoverDropService, dev::watch::builder::BuilderSignal,
    log_handler::LogHandler,
};

const DOCKERFILE_TEMPLATE: &str = r#"
# Special docker file that doesn't build the Rust binary within docker
# but instead copies it in from the host.
#
# This build cannot run without substitutions

# Use an official Rust image as the base
FROM rust:latest AS builder

# Set the working directory in the container
WORKDIR /usr/src/coverdrop

# Copy the actual source code
COPY . .

# Build the application
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/coverdrop/target \
    cargo build -p {{service_name}}

RUN ls .

# move the binary out of the target directory so it's out of Buildkit's cache
# and available to the COPY command in the runtime container

RUN mkdir /usr/src/coverdrop/build/

RUN --mount=type=cache,target=/usr/src/coverdrop/target \
    cp /usr/src/coverdrop/target/debug/{{service_name}} /usr/src/coverdrop/build/{{service_name}}

FROM cgr.dev/chainguard/glibc-dynamic

COPY --chown=nonroot:nonroot \
     --from=builder          \
     /usr/src/coverdrop/build/{{service_name}} /usr/local/bin/{{service_name}}


CMD ["/usr/local/bin/{{service_name}}"]
"#;

pub async fn docker_build_rust(
    workspace_path: impl AsRef<Path>,
    service: &CoverDropService,
    log_handler: &LogHandler<'_>,
) -> anyhow::Result<String> {
    let workspace_path = workspace_path.as_ref();

    let tag = format!("coverdrop_{}:coverup", service.as_str());

    let tag_arg = format!("--tag={tag}");

    let mut docker_build = process::Command::new("docker")
        .arg("build")
        .arg(tag_arg)
        .arg("--progress=plain")
        .arg("--file=-")
        .arg(workspace_path)
        .env("DOCKER_BUILDKIT", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let dockerfile = DOCKERFILE_TEMPLATE.replace("{{service_name}}", service.as_str());

    if let Some(mut stdin) = docker_build.stdin.take() {
        tracing::info!("Writing docker file on stdin");
        stdin.write_all(dockerfile.as_bytes()).await?;
    }

    let Some(stdout) = docker_build.stdout.take() else {
        anyhow::bail!("Could not get stdout from docker build process");
    };

    let mut stdout_lines = BufReader::new(stdout).lines();

    let Some(stderr) = docker_build.stderr.take() else {
        anyhow::bail!("Could not get stderr from docker build process");
    };

    let mut stderr_lines = BufReader::new(stderr).lines();

    loop {
        tokio::select! {
            Ok(Some(stdout_line)) = stdout_lines.next_line() => {
                match &log_handler {
                    LogHandler::ForwardLogForBuilder(service, log_tx) => {
                        _ = log_tx.send(BuilderSignal::LogLine(*service, stdout_line))
                    }
                    LogHandler::None => tracing::info!("    > {}", stdout_line),
                }
            }
            Ok(Some(stderr_line)) = stderr_lines.next_line() => {
                match &log_handler {
                    LogHandler::ForwardLogForBuilder(service, log_tx) => {
                        _ = log_tx.send(BuilderSignal::LogLine(*service, stderr_line))
                    }
                    LogHandler::None => tracing::info!("    > {}", stderr_line),
                }
            }
            else => break,
        }
    }

    tracing::info!("Waiting for docker build...");

    let status = docker_build.wait().await?;
    tracing::info!("Finished waiting");

    if !status.success() {
        anyhow::bail!("Failed to create docker image");
    }

    Ok(tag)
}

pub async fn copy_image_to_node(
    image_and_tag: &str,
    node: &Ipv4Addr,
    log_handler: &LogHandler<'_>,
) -> anyhow::Result<()> {
    let mut docker_build = process::Command::new("docker")
        .arg("save")
        .arg(image_and_tag)
        .stdout(Stdio::piped())
        .spawn()?;

    let mut ssh = process::Command::new("ssh")
        .arg(format!("ubuntu@{node}"))
        .arg("sudo k3s ctr images import -")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let (Some(mut docker_stdout), Some(mut ssh_stdin)) =
        (docker_build.stdout.take(), ssh.stdin.take())
    {
        tokio::io::copy(&mut docker_stdout, &mut ssh_stdin).await?;
    }

    let Some(stdout) = ssh.stdout.take() else {
        anyhow::bail!("Could not get stdout from docker build process");
    };

    let mut stdout_lines = BufReader::new(stdout).lines();

    let Some(stderr) = ssh.stderr.take() else {
        anyhow::bail!("Could not get stderr from docker build process");
    };

    let mut stderr_lines = BufReader::new(stderr).lines();

    loop {
        tokio::select! {
            Ok(Some(stdout_line)) = stdout_lines.next_line() => {
                match &log_handler {
                    LogHandler::ForwardLogForBuilder(service, log_tx) => {
                        _ = log_tx.send(BuilderSignal::LogLine(*service, stdout_line))
                    }
                    LogHandler::None => tracing::info!("    > {}", stdout_line),
                }
            }
            Ok(Some(stderr_line)) = stderr_lines.next_line() => {
                match &log_handler {
                    LogHandler::ForwardLogForBuilder(service, log_tx) => {
                        _ = log_tx.send(BuilderSignal::LogLine(*service, stderr_line))
                    }
                    LogHandler::None => tracing::info!("    > {}", stderr_line),
                }
            }
            else => break,
        }
    }

    let docker_status = docker_build.wait().await?;
    let ssh_status = ssh.wait().await?;

    if !docker_status.success() || !ssh_status.success() {
        anyhow::bail!("Failed to copy image to node");
    }

    Ok(())
}
