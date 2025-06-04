use std::process::Command;

use crate::coverup_home::CoverUpHome;

#[derive(Hash)]
pub struct ExternalDependency {
    pub binary: &'static str,
    pub install_advice: &'static str,
}

impl ExternalDependency {
    pub const fn new(binary: &'static str, install_advice: &'static str) -> Self {
        Self {
            binary,
            install_advice,
        }
    }
}

pub const DOCKER: ExternalDependency = ExternalDependency::new(
    "docker",
    "Follow instructions on: https://docs.docker.com/engine/install/",
);

pub const CARGO: ExternalDependency =
    ExternalDependency::new("cargo", "Follow instructions on: https://rustup.rs/");

pub const SSH: ExternalDependency = ExternalDependency::new(
    "ssh",
    "You will need to search for platform specific installation instructions",
);

pub const MULTIPASS: ExternalDependency = ExternalDependency::new(
    "multipass",
    "Follow instructions on: https://multipass.run/install",
);

const DEPENDENCIES: [ExternalDependency; 4] = [CARGO, DOCKER, MULTIPASS, SSH];

pub fn external_dependency_preflight_check(coverup_home: &CoverUpHome) -> anyhow::Result<()> {
    if coverup_home.cached_dependencies_file_exists(&DEPENDENCIES) {
        return Ok(());
    }

    let mut missing_dependency = false;
    for dep in DEPENDENCIES {
        let command = Command::new("which").arg(dep.binary).output()?;

        if !command.status.success() {
            eprintln!(
                "Binary '{}' not found or not executable. {}",
                dep.binary, dep.install_advice
            );
            missing_dependency = true;
        }
    }

    if missing_dependency {
        anyhow::bail!("One or more missing dependencies found.")
    }

    coverup_home.create_cached_dependencies_file(&DEPENDENCIES)?;

    Ok(())
}
