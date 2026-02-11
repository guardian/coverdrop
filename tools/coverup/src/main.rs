use crate::aws::ec2::get_file_contents_from_instance;
use crate::k8s::cluster::tunnel_and_run;
use crate::k8s::port_forward::{
    port_forward_argo, port_forward_kubernetes_dashboard, port_forward_longhorn,
};
use crate::ssh_scp::scp::{scp, ScpDirection};
use crate::subprocess::wait_for_subprocess;
use admin::generate_identity_api_db;
use clap::Parser;
use cli::{
    AdminCommand, BackupCommand, Cli, Command, CoverNodeCommand, DevelopmentCommand,
    IdentityApiCommand, JournalistVaultCommand, ProductionCommand, StagingCommand, VerifyCommand,
};
use commands::development_commands::{copy_all_images_to_multipass, copy_image_to_multipass};
use commands::staging_commands::minio_tunnel;
use commands::{
    back_up, copy_file, covernode_commands, data_copier_shell, development_commands,
    identity_api_commands, list_files, production_commands, staging_commands,
};
use common::argon2_sqlcipher::Argon2SqlCipher;
use common::clap::{validate_password_from_args, Stage};
use common::crypto::keys::serde::StorableKeyMaterial;
use common::protocol::keys::{
    anchor_org_pk, verify_journalist_provisioning_pk, UntrustedAnchorOrganizationPublicKey,
    UntrustedJournalistProvisioningKeyPair, UntrustedJournalistProvisioningPublicKey,
};
use common::time;
use coverup_home::CoverUpHome;
use external_dependencies::external_dependency_preflight_check;
use journalist_vault::JournalistVault;
use multipass::list_coverdrop_nodes;
use rpassword::prompt_password;
use ssh_scp::{command_over_ssh, tunnel_and_port_forward};
use trust_anchors::get_trust_anchors;

use std::fs::File;
use std::io::Write;
use tokio::{process, signal};
use tracing_subscriber::EnvFilter;

mod admin;
mod aws;
mod bring_up;
mod cli;
mod commands;
mod coverdrop_service;
mod coverup_home;
mod data_copier_pod;
mod dev;
mod development_image_source;
mod docker;
mod external_dependencies;
mod k8s;
mod kube_client;
mod listed_file;
mod local_or_pvc_path;
mod log_handler;
mod multipass;
mod ssh_scp;
mod subprocess;
mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(not(unix))]
    panic!("Cannot run coverup on non-unix platform");

    // Create a home directory for coverup, used for storing files for use with other programs
    // such as the SSH keys for use with ansible.
    let coverup_home = CoverUpHome::new()?;

    let cli = Cli::parse();
    tracing::debug!("Cli args: {:?}", cli);

    let subscriber = tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env());

    if !cli.skip_preflight_checks {
        external_dependency_preflight_check(&coverup_home)?;
    }

    if matches!(
        cli.command,
        Command::Development {
            command: DevelopmentCommand::Watch
        }
    ) {
        let file_writer = File::create("./coverup.log")?;
        subscriber.with_writer(file_writer).init();
    } else {
        subscriber.init();
    }

    // Plumbing from the CLI to the command modules

    match cli.command {
        Command::CopyFile {
            source,
            destination,
            force,
            stage,
        } => {
            let kubeconfig_path = coverup_home.kubeconfig_path_for_optional_stage(stage)?;

            copy_file(&source, &destination, force, &kubeconfig_path).await?
        }
        Command::Backup { command } => match command {
            BackupCommand::Create {
                output_directory,
                stage,
            } => {
                let kubeconfig_path = coverup_home.kubeconfig_path_for_optional_stage(stage)?;

                back_up(&output_directory, kubeconfig_path).await?
            }
        },
        Command::ListFiles { path, long, stage } => {
            let kubeconfig_path = coverup_home.kubeconfig_path_for_optional_stage(stage)?;

            list_files(&path, long, kubeconfig_path).await?
        }
        Command::DataCopierShell { service, stage } => {
            let kubeconfig_path = coverup_home.kubeconfig_path_for_optional_stage(stage)?;
            data_copier_shell(service, &kubeconfig_path).await?;
        }

        //
        // Production Cluster
        //
        Command::Production { command } => match command {
            ProductionCommand::BringUp { nodes } => production_commands::bring_up(&nodes).await?,
            ProductionCommand::K8s {
                ssh_user,
                admin_machine_ip,
                port,
            } => {
                println!(
                    "Fetching bearer token, which you'll need to login to the dashboard. Check the stdout of the ssh command for the token"
                );
                command_over_ssh(
                    &ssh_user,
                    admin_machine_ip,
                    "kubectl -n kubernetes-dashboard create token admin-user",
                )
                .await?;
                tunnel_and_port_forward(
                    &ssh_user,
                    admin_machine_ip,
                    "kubernetes-dashboard-kong-proxy",
                    "kubernetes-dashboard",
                    443,
                    port,
                )
                .await?;
            }
            ProductionCommand::Argo {
                ssh_user,
                admin_machine_ip,
                port,
            } => {
                tunnel_and_port_forward(
                    &ssh_user,
                    admin_machine_ip,
                    "argocd-server",
                    "argocd",
                    443,
                    port,
                )
                .await?;
            }
            ProductionCommand::Longhorn {
                admin_machine_ip,
                ssh_user,
                local_port,
            } => {
                tunnel_and_port_forward(
                    &ssh_user,
                    admin_machine_ip,
                    "longhorn-frontend",
                    "longhorn-system",
                    80,
                    local_port,
                )
                .await?;
            }
            ProductionCommand::Minio => {
                unimplemented!("See docs/on_premises_backups.md");
            }
        },

        //
        // Staging Cluster
        //
        Command::Staging { command } => match command {
            StagingCommand::TearDown { aws_config } => {
                staging_commands::tear_down(aws_config).await?
            }
            StagingCommand::KubectlTunnel { aws_config, port } => {
                let child = staging_commands::kubectl_tunnel(aws_config, port).await?;
                wait_for_subprocess(child, "Kubectl tunnel").await?;
            }
            StagingCommand::Argo { aws_config, port } => {
                let kubeconfig_path = coverup_home.kubeconfig_path_for_stage(Stage::Staging)?;

                tunnel_and_run(aws_config, &kubeconfig_path, || async {
                    if let Err(e) = port_forward_argo(port, &kubeconfig_path).await {
                        eprint!("Error setting up port forwarding for argo {e:?}")
                    };
                })
                .await?;
            }
            StagingCommand::Longhorn { aws_config, port } => {
                let kubeconfig_path = coverup_home.kubeconfig_path_for_stage(Stage::Staging)?;

                tunnel_and_run(aws_config, &kubeconfig_path, || async {
                    if let Err(e) = port_forward_longhorn(port, &kubeconfig_path).await {
                        eprint!("Error setting up port forwarding for longhorn {e:?}")
                    };
                })
                .await?;
            }
            StagingCommand::Minio { aws_config, port } => {
                let mut child = minio_tunnel(aws_config, port)
                    .await
                    .map_err(|e| anyhow::anyhow!("Error setting up minio tunnel: {:?}", e))?;

                // Wait for Ctrl+C
                signal::ctrl_c().await?;
                println!("\nReceived Ctrl+C, shutting down...");

                // Kill the tunnel process
                child.kill().await?;
            }
            StagingCommand::K8s { aws_config, port } => {
                let kubeconfig_path = coverup_home.kubeconfig_path_for_stage(Stage::Staging)?;
                tunnel_and_run(aws_config, &kubeconfig_path, || async {
                    if let Err(e) = port_forward_kubernetes_dashboard(port, &kubeconfig_path).await
                    {
                        eprint!("Error setting up port forwarding for k8s {e:?}")
                    };
                })
                .await?;
            }
            StagingCommand::Kubeconfig {
                aws_config,
                ssm_output_bucket,
            } => {
                let kubeconfig_path = coverup_home.kubeconfig_for_stage(Stage::Staging);
                let kubeconfig_str = kubeconfig_path.to_str().expect("Generate kubeconfig path");
                let kubeconfig_string = get_file_contents_from_instance(
                    aws_config,
                    ssm_output_bucket,
                    "/etc/rancher/k3s/k3s.yaml",
                )
                .await?;

                println!("Fetched context file, writing to {kubeconfig_str}");
                // Trying to apply a vague standard that port number for a remote machine will be 1xxxx
                // whereas currently the dev multipass cluster is accessible on 6443
                let context_string_new_port =
                    kubeconfig_string.replace("https://127.0.0.1:6443", "https://127.0.0.1:16443");
                let mut file = File::create(kubeconfig_path)?;
                file.write_all(context_string_new_port.as_bytes())?;
            }
        },

        //
        // Development
        //
        Command::Development { command } => match command {
            DevelopmentCommand::BringUp {
                image_source,
                node_count,
                cpus_per_node,
                ram_gb_per_node,
                storage_gb_per_node,
            } => {
                development_commands::bring_up(
                    &image_source,
                    node_count,
                    cpus_per_node,
                    ram_gb_per_node,
                    storage_gb_per_node,
                )
                .await?
            }
            DevelopmentCommand::Build { service } => development_commands::build(service).await?,
            DevelopmentCommand::Watch => {
                let kubeconfig_dev_path =
                    coverup_home.kubeconfig_path_for_stage(Stage::Development)?;
                development_commands::watch(kubeconfig_dev_path).await?
            }
            DevelopmentCommand::Kubeconfig {
                ssh_key_path,
                ssh_user,
            } => {
                let nodes = list_coverdrop_nodes()?;
                let node = nodes
                    .first()
                    .ok_or_else(|| anyhow::anyhow!("No nodes found"))?;
                let node_ip = node
                    .ipv4
                    .first()
                    .ok_or_else(|| anyhow::anyhow!("No IP found"))?;
                scp(
                    node_ip,
                    ssh_user,
                    "/etc/rancher/k3s/k3s.yaml",
                    &coverup_home.kubeconfig_for_stage(Stage::Development),
                    ScpDirection::RemoteToLocal,
                    &ssh_key_path,
                )
                .await?;
            }
            DevelopmentCommand::CopyImageToMultipass { image, all } => {
                if all {
                    copy_all_images_to_multipass()?;
                } else {
                    copy_image_to_multipass(&image)?;
                }
            }
        },

        //
        // CoverNode
        //
        Command::CoverNode { command } => match command {
            CoverNodeCommand::Healthcheck { stage } => {
                let kubeconfig_path = coverup_home.kubeconfig_path_for_optional_stage(stage)?;
                covernode_commands::healthcheck(kubeconfig_path).await?
            }
            CoverNodeCommand::PublicKeys { stage } => {
                let kubeconfig_path = coverup_home.kubeconfig_path_for_optional_stage(stage)?;
                covernode_commands::public_keys(kubeconfig_path).await?;
            }
        },

        //
        // Identity API
        //
        Command::IdentityApi { command } => match command {
            IdentityApiCommand::Healthcheck { stage } => {
                let kubeconfig_path = coverup_home.kubeconfig_path_for_optional_stage(stage)?;
                identity_api_commands::healthcheck(kubeconfig_path).await?
            }
            IdentityApiCommand::PublicKeys { stage } => {
                let kubeconfig_path = coverup_home.kubeconfig_path_for_optional_stage(stage)?;
                identity_api_commands::public_keys(kubeconfig_path).await?
            }
        },

        //
        // Journalist Vault
        //
        Command::JournalistVault { command } => match command {
            JournalistVaultCommand::DeriveKey {
                vault_path,
                password,
                password_path,
            } => {
                println!("Deriving key for vault at {vault_path:?}");

                let password = validate_password_from_args(password, password_path)?;

                let key_string_result =
                    Argon2SqlCipher::derive_database_key(&vault_path, &password).await;

                match key_string_result {
                    Err(e) => eprintln!("Error: {e:?}"),
                    Ok(key_string) => println!("Key: {key_string}"),
                }
            }

            JournalistVaultCommand::OpenVault {
                vault_path,
                password,
                password_path,
            } => {
                println!("Opening vault at {vault_path:?}");

                let password = validate_password_from_args(password, password_path)?;

                let key_string_result =
                    Argon2SqlCipher::derive_database_key(&vault_path, &password).await;

                match key_string_result {
                    Err(e) => eprintln!("Error calculating key: {e:?}"),
                    Ok(key_string) => {
                        let pragma_arg = format!("PRAGMA key=\"x'{key_string}'\";");
                        let exec_args =
                            vec![vault_path.to_str().unwrap(), "-cmd", pragma_arg.as_str()];

                        let status = process::Command::new("sqlcipher")
                            .args(&exec_args)
                            .status()
                            .await?;

                        if !status.success() {
                            anyhow::bail!("Failed to open vault with error code: {}", status);
                        }
                    }
                }
            }
            JournalistVaultCommand::ExecuteVaultQuery {
                vault_path,
                password,
                password_path,
                sql_query,
            } => {
                let password = validate_password_from_args(password, password_path)?;

                let key_string_result =
                    Argon2SqlCipher::derive_database_key(&vault_path, &password).await;

                match key_string_result {
                    Err(e) => eprintln!("Error calculating key: {e:?}"),
                    Ok(key_string) => {
                        let pragma_arg = format!("PRAGMA key=\"x'{key_string}'\"");
                        let exec_args = vec![
                            vault_path.to_str().unwrap(),
                            "-cmd",
                            pragma_arg.as_str(),
                            &sql_query,
                        ];

                        let output = process::Command::new("sqlcipher")
                            .args(exec_args.clone())
                            .output()
                            .await?;

                        let stdout = String::from_utf8_lossy(&output.stdout);

                        let after_newline = stdout.split_once('\n').map(|x| x.1).unwrap_or("");

                        if !output.status.success() {
                            anyhow::bail!(
                                "Failed to open vault with error code: {}",
                                output.status
                            );
                        }
                        println!("{}", after_newline);
                    }
                }
            }
            JournalistVaultCommand::AddProvisioningPublicKey {
                vault_path,
                password,
                password_path,
                journalist_provisioning_pk_path,
                stage,
            } => {
                let password = validate_password_from_args(password, password_path)?;
                let trust_anchors = get_trust_anchors(&stage, time::now())?;
                let vault = JournalistVault::open(&vault_path, &password, trust_anchors).await?;

                let now = time::now();

                let org_pks = vault.org_pks()?;

                let journalist_provisioning_pk =
                    UntrustedJournalistProvisioningPublicKey::load_from_file(
                        &journalist_provisioning_pk_path,
                    )?;

                let maybe_verified_journalist_provisioning_pk = org_pks.iter().find_map(|org_pk| {
                    let org_pk = org_pk.to_non_anchor();
                    verify_journalist_provisioning_pk(&journalist_provisioning_pk, &org_pk, now)
                        .ok()
                });

                let Some(journalist_provisioning_pk) = maybe_verified_journalist_provisioning_pk
                else {
                    anyhow::bail!(
                        "Could not find trust anchor for journalist provisioning public key"
                    );
                };

                vault
                    .add_provisioning_pk(&journalist_provisioning_pk, now)
                    .await?;

                println!("OK");
            }
            // TODO: delete https://github.com/guardian/coverdrop-internal/issues/3100
            JournalistVaultCommand::MigrateHexArgon2Database {
                vault_path,
                password,
                password_path,
            } => {
                let password = validate_password_from_args(password, password_path)?;

                Argon2SqlCipher::migrate_hex_argon2(vault_path, &password)
                    .await
                    .unwrap();

                println!("OK");
            }
        },
        Command::Verify { command } => match command {
            VerifyCommand::JournalistProvisioningKeyPair {
                journalist_provisioning_key_pair_path,
                organization_public_key_path,
            } => {
                let now = time::now();

                let journalist_provisioning_key_pair =
                    UntrustedJournalistProvisioningKeyPair::load_from_file(
                        journalist_provisioning_key_pair_path,
                    )
                    .map_err(|_| {
                        anyhow::anyhow!("Failed to load journalist provisioning key pair")
                    })?;

                let org_pk = UntrustedAnchorOrganizationPublicKey::load_from_file(
                    organization_public_key_path,
                )
                .map_err(|_| anyhow::anyhow!("Failed to load organization public key"))?;

                let org_pk = anchor_org_pk(&org_pk, now).map_err(|_| {
                    anyhow::anyhow!(
                        "Failed to verify self-signed organization public key, is it expired?"
                    )
                })?;

                let result = journalist_provisioning_key_pair.to_trusted(&org_pk, now);

                if result.is_ok() {
                    println!("OK");
                } else {
                    println!("Failed to verify");
                }
            }
        },
        Command::Admin { command } => match command {
            AdminCommand::GenerateIdentityApiDatabase { keys_path, db_path } => {
                let password = prompt_password("Enter new identity-api database password: ")?;
                let confirm_password = prompt_password("Confirm identity-api database password: ")?;

                if password != confirm_password {
                    anyhow::bail!("Provided passwords did not match");
                }

                generate_identity_api_db(db_path, &password, keys_path, true).await?;
            }
        },
    }

    Ok(())
}
