use admin::generate_admin_key_pair;
use admin::generate_constant_files;
use admin::generate_covernode_database;
use admin::generate_covernode_provisioning_key_pair;
use admin::generate_journalist;
use admin::generate_journalist_provisioning_key_pair;
use admin::generate_test_vectors;
use admin::post_covernode_provisioning_key_pair;
use admin::post_journalist_provisioning_key_pair;
use admin::post_log_config_form;
use admin::reseed_journalist_vault_id_key_pair;
use admin::run_setup_ceremony;
use admin::submit_delete_journalist_form;
use admin::update_journalist;
use admin::update_system_status;
use admin::upload_keys_to_api;
use admin::{
    backup_complete_restore, backup_initiate_restore, backup_initiate_restore_finalize,
    delete_journalist_form,
};
use admin::{
    generate_covernode_identity_key_pair, generate_covernode_messaging_key_pair,
    generate_organization_key_pair,
};
use clap::Parser;
use cli::{Cli, Commands};
use common::api::api_client::ApiClient;
use common::api::forms::PostBackupIdKeyForm;
use common::api::forms::PostBackupMsgKeyForm;
use common::api::forms::BACKUP_MESSAGING_KEY_FORM_FILENAME;
use common::api::forms::BACKUP_SIGNING_KEY_FORM_FILENAME;
use common::aws::s3::client::S3Client;
use common::backup::keys::generate_backup_id_key_pair;
use common::backup::keys::generate_backup_msg_key_pair;
use common::clap::validate_password_from_args;
use common::crypto::human_readable_digest;
use common::crypto::keys::public_key::PublicKey;
use common::crypto::keys::serde::StorableKeyMaterial;
use common::crypto::pbkdf::DEFAULT_PASSPHRASE_WORDS;
use common::generators::PasswordGenerator;
use common::protocol::backup::WrappedSecretShare;
use common::protocol::keys::load_anchor_org_pks;
use common::protocol::keys::load_backup_id_key_pairs;
use common::protocol::keys::load_latest_org_key_pair;
use common::protocol::keys::LatestKey;
use common::time;
use common::time::now;
use common::tracing::init_tracing;
use journalist_vault::JournalistVault;
use journalist_vault::PASSWORD_EXTENSION;
use tokio::fs;

#[cfg(feature = "integration-tests")]
mod integration_tests;

mod cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    init_tracing("debug");

    let result = match cli.command {
        Commands::RunSetupCeremony {
            output_directory,
            covernode_count,
            assume_yes,
            covernode_db_password,
            org_key_pair_path,
        } => {
            run_setup_ceremony(
                output_directory,
                covernode_count,
                assume_yes,
                covernode_db_password,
                org_key_pair_path,
                time::now(),
            )
            .await
        }
        Commands::UploadKeysToApi {
            bundle_directory_path,
            api_url,
            aws_config,
            parameter_prefix,
        } => {
            let api_client = ApiClient::new(api_url);

            upload_keys_to_api(
                bundle_directory_path,
                &api_client,
                &aws_config,
                &parameter_prefix,
            )
            .await
        }
        Commands::GenerateOrganizationKeyPair { keys_path } => {
            generate_organization_key_pair(keys_path, false, time::now())
        }
        Commands::GenerateJournalistProvisioningKeyPair { keys_path } => {
            generate_journalist_provisioning_key_pair(keys_path, false, time::now()).await
        }
        Commands::PostJournalistProvisioningKeyForm { api_url, form_path } => {
            let api_client = ApiClient::new(api_url);
            post_journalist_provisioning_key_pair(form_path, api_client).await
        }
        Commands::GenerateCoverNodeProvisioningKeyPair { keys_path } => {
            generate_covernode_provisioning_key_pair(keys_path, false, time::now()).await
        }
        Commands::PostCoverNodeProvisioningKeyForm { api_url, form_path } => {
            let api_client = ApiClient::new(api_url);
            post_covernode_provisioning_key_pair(form_path, api_client).await
        }
        Commands::GenerateCoverNodeIdentityKeyPair {
            covernode_id,
            keys_path,
            api_url,
            do_not_upload_to_api,
        } => {
            let api_client = ApiClient::new(api_url);
            generate_covernode_identity_key_pair(
                covernode_id,
                keys_path,
                api_client,
                do_not_upload_to_api,
                false,
                time::now(),
            )
            .await
        }
        Commands::GenerateCoverNodeMessagingKeyPair {
            keys_path,
            api_url,
            do_not_upload_to_api,
        } => {
            let api_client = ApiClient::new(api_url);
            generate_covernode_messaging_key_pair(
                keys_path,
                api_client,
                do_not_upload_to_api,
                false,
                time::now(),
            )
            .await
        }
        Commands::GenerateCoverNodeDatabase {
            covernode_id,
            keys_path,
            db_password,
            output_path,
        } => generate_covernode_database(keys_path, covernode_id, &db_password, output_path).await,
        Commands::GenerateJournalist {
            display_name,
            id,
            description,
            password,
            status,
            sort_name,
            vault_path,
            is_desk,
            keys_path,
        } => {
            let password = password.map(anyhow::Ok).unwrap_or_else(|| {
                let password_generator = PasswordGenerator::from_eff_large_wordlist()?;
                anyhow::Ok(password_generator.generate(DEFAULT_PASSPHRASE_WORDS))
            })?;

            generate_journalist(
                keys_path,
                display_name,
                id,
                sort_name,
                description,
                is_desk,
                &password,
                status,
                vault_path,
                time::now(),
            )
            .await?;

            Ok(())
        }
        #[cfg(feature = "integration-tests")]
        Commands::GenerateJournalistMessagingKeysForIntegrationTest { keys_path } => {
            integration_tests::generate_journalist_messaging_keys_for_integration_test(
                keys_path,
                time::now(),
            )
            .await?;
            Ok(())
        }
        Commands::ChangeVaultPassword {
            vault_path,
            current_password,
        } => {
            let password_generator = PasswordGenerator::from_eff_large_wordlist()?;
            let new_password = password_generator.generate(DEFAULT_PASSPHRASE_WORDS);

            let journalist_vault = JournalistVault::open(&vault_path, &current_password).await?;

            journalist_vault.change_password(&new_password).await?;

            let password_path = vault_path.with_extension(PASSWORD_EXTENSION);
            fs::write(&password_path, new_password).await?;

            Ok(())
        }
        Commands::UpdateJournalist {
            api_url,
            journalist_id,
            display_name,
            sort_name,
            is_desk,
            description,
            keys_path,
        } => {
            update_journalist(
                api_url,
                journalist_id,
                display_name,
                sort_name,
                is_desk,
                description,
                keys_path,
                time::now(),
            )
            .await?;

            Ok(())
        }
        Commands::ReseedJournalistVaultIdKeyPair {
            journalist_id,
            keys_path,
            vault_path,
            password,
            password_path,
        } => {
            let password = validate_password_from_args(password, password_path)?;
            let vault = JournalistVault::open(&vault_path, &password).await?;

            let now = time::now();

            reseed_journalist_vault_id_key_pair(keys_path, journalist_id, &vault, now).await?;

            Ok(())
        }
        Commands::DeleteJournalistForm {
            journalist_id,
            keys_path,
            output_path,
        } => {
            delete_journalist_form(keys_path, &journalist_id, output_path, time::now()).await?;

            Ok(())
        }
        Commands::DeleteJournalist { api_url, form_path } => {
            let api_client = ApiClient::new(api_url);

            submit_delete_journalist_form(&api_client, form_path).await?;

            Ok(())
        }
        Commands::GenerateTestVectors { path } => generate_test_vectors(&path),
        Commands::GenerateMobileConstantsFiles {
            android_path,
            ios_path,
        } => generate_constant_files(&android_path, &ios_path),
        Commands::GenerateAdminKeyPair {
            keys_path,
            api_url,
            do_not_upload_to_api,
        } => {
            let api_client = ApiClient::new(api_url);

            generate_admin_key_pair(
                keys_path,
                api_client,
                do_not_upload_to_api,
                false,
                time::now(),
            )
            .await
        }
        Commands::GenerateBackupIdentityKeyPair { keys_path } => {
            let org_key_pair = load_latest_org_key_pair(&keys_path, time::now())?;
            let backup_id_key_pair = generate_backup_id_key_pair(&org_key_pair, time::now());
            let backup_id_key_pair_path =
                backup_id_key_pair.to_untrusted().save_to_disk(&keys_path)?;

            println!(
                "Backup identity key pair saved to {:?}. Move this to a tamper-proof bag and check it in.",
                backup_id_key_pair_path
            );

            // TODO the form type should be in the type system, then we won't need to pass the file name to save_to_disk
            let form_path = keys_path.join(BACKUP_SIGNING_KEY_FORM_FILENAME);
            PostBackupIdKeyForm::new(
                backup_id_key_pair.public_key().to_untrusted(),
                &org_key_pair,
                time::now(),
            )?
            .save_to_disk(&form_path)?;

            println!("Backup identity key form saved to {:?}.", form_path);
            println!("Move this to an online machine and post it to the api with post-backup-identity-key-form WITHIN ONE HOUR!");

            Ok(())
        }
        Commands::PostBackupIdentityKeyForm { api_url, form_path } => {
            let api_client = ApiClient::new(api_url);

            // TODO the form type should be in the type system, then we won't need to pass the file name to save_to_disk
            let json =
                fs::read_to_string(&form_path.join(BACKUP_SIGNING_KEY_FORM_FILENAME)).await?;
            let form: PostBackupIdKeyForm = serde_json::from_str(&json)?;

            api_client.post_backup_signing_pk(form).await?;

            Ok(())
        }
        Commands::GenerateBackupMessagingKeyPair { keys_path } => {
            let anchor_org_pks = load_anchor_org_pks(&keys_path, time::now())?;
            // return early if no anchor org pks found
            if anchor_org_pks.is_empty() {
                anyhow::bail!("No anchor organization public keys found in the keys path. Cannot generate backup messaging key pair.");
            }
            let backup_id_key_pairs =
                load_backup_id_key_pairs(&keys_path, &anchor_org_pks, time::now())?;
            let latest_backup_id_key_pair = backup_id_key_pairs.into_latest_key_required()?;

            let backup_msg_key_pair =
                generate_backup_msg_key_pair(&latest_backup_id_key_pair, time::now());
            let backup_msg_key_pair_path = backup_msg_key_pair
                .to_untrusted()
                .save_to_disk(&keys_path)?;

            println!(
                "Backup messaging key pair saved to {:?}. Move this to a tamper-proof bag and check it in.",
                backup_msg_key_pair_path
            );

            // TODO the form type should be in the type system, then we won't need to pass the file name to save_to_disk
            let form_path = keys_path.join(BACKUP_MESSAGING_KEY_FORM_FILENAME);
            PostBackupMsgKeyForm::new(
                backup_msg_key_pair.public_key().to_untrusted(),
                &latest_backup_id_key_pair,
                time::now(),
            )?
            .save_to_disk(&form_path)?;

            println!("Backup messaging key form saved to {:?}.", form_path);
            println!("Move this to an online machine and post it to the api with post-backup-messaging-key-form WITHIN ONE HOUR!");

            Ok(())
        }
        Commands::PostBackupMessagingKeyForm { api_url, form_path } => {
            let api_client = ApiClient::new(api_url);

            let json =
                fs::read_to_string(&form_path.join(BACKUP_MESSAGING_KEY_FORM_FILENAME)).await?;
            let form: common::api::forms::PostBackupMsgKeyForm = serde_json::from_str(&json)?;

            api_client.post_backup_encryption_pk(form).await?;

            Ok(())
        }
        Commands::UpdateSystemStatus {
            keys_path,
            api_url,
            status,
            description,
        } => {
            let api_client = ApiClient::new(api_url);

            update_system_status(keys_path, &api_client, status, description, time::now()).await?;

            Ok(())
        }
        Commands::PostReloadLoggingForm {
            service_url,
            keys_path,
            rust_log_directive,
        } => post_log_config_form(service_url, keys_path, rust_log_directive).await,
        Commands::PrintOrganisationKeyDigests { api_url, keys_path } => {
            if api_url.is_none() && keys_path.is_none() {
                anyhow::bail!("Provide either --api-url or --keys-path or both");
            }

            if let Some(api_url) = api_url {
                let api_client = ApiClient::new(api_url);
                let keys = api_client.get_public_keys().await?;

                let org_keys = keys.untrusted_org_pk_iter().collect::<Vec<_>>();

                for key in org_keys {
                    println!(
                        "[API] {} -> {}",
                        &key.public_key_hex(),
                        human_readable_digest(&key.key)
                    );
                }
            }

            if let Some(keys_path) = keys_path {
                let keys = common::protocol::keys::load_anchor_org_pks(keys_path, time::now())?;
                for key in keys {
                    println!(
                        "[LOCAL] {} -> {}",
                        &key.public_key_hex(),
                        human_readable_digest(&key.key)
                    );
                }
            }

            Ok(())
        }
        Commands::BackupInitiateRestore {
            api_url,
            output_path,
            aws_config,
            s3_url,
            stage,
            journalist_id,
        } => {
            let s3_client = S3Client::new(aws_config, s3_url).await;
            let response_bundle_file =
                backup_initiate_restore(api_url, &s3_client, &stage, &output_path, &journalist_id)
                    .await?;

            print_step(
                "Step 1/3",
                "Response bundle retrieved from API (ONLINE MACHINE)",
            );
            println!("Response file: {}", response_bundle_file.display());

            print_next_steps(&[
                "Transfer the response bundle file to the AIR-GAPPED machine",
                "On the AIR-GAPPED machine, run:",
                &format!(
                    "\n     admin backup-initiate-restore-finalize \\\n\
                           --bundle-response-path {} \\\n\
                           --keys-path <KEYS_DIR> \\\n\
                           --output-path <OUTPUT_DIR>",
                    response_bundle_file.display()
                ),
            ]);

            Ok(())
        }
        Commands::BackupInitiateRestoreFinalize {
            bundle_response_path,
            keys_path,
            output_path,
        } => {
            let (in_progress_bundle_file, encrypted_share_files) =
                backup_initiate_restore_finalize(
                    &bundle_response_path,
                    keys_path,
                    &output_path,
                    now(),
                )
                .await?;

            print_step(
                "Step 2/3",
                "Backup decrypted and shares created (AIR-GAPPED MACHINE)",
            );
            println!("In-progress bundle: {}", in_progress_bundle_file.display());
            println!("Encrypted shares created: {}", encrypted_share_files.len());
            for (i, share_file) in encrypted_share_files.iter().enumerate() {
                println!("   Share {}: {}", i + 1, share_file.display());
            }

            print_next_steps(&[
                "Distribute the encrypted shares to the trusted recovery contacts. \
                 It is safe to send them via Signal.",
                "Each recovery contact should decrypt their share in Sentinel and \
                 return the recovered share (if any)",
                &format!(
                    "Once you have collected enough shares (k shares), complete the restore:\n\n\
                           admin backup-complete-restore \\\n\
                           --in-progress-bundle-path {} \\\n\
                           --restore-to-vault-path <VAULT_PATH> \\\n\
                           --keys-path <KEYS_DIR> \\\n\
                           --shares <SHARE_1> <SHARE_2> ...",
                    in_progress_bundle_file.display()
                ),
            ]);

            Ok(())
        }
        Commands::BackupCompleteRestore {
            in_progress_bundle_path,
            restore_to_vault_path,
            keys_path,
            shares,
        } => {
            let wrapped_shares = shares
                .into_iter()
                .map(|s| WrappedSecretShare::from_base64_string(&s))
                .collect::<Result<Vec<WrappedSecretShare>, _>>()?;

            let restored_vault_path = backup_complete_restore(
                &in_progress_bundle_path,
                &restore_to_vault_path,
                &keys_path,
                wrapped_shares,
                now(),
            )
            .await?;

            print_step("Step 3/3", "Backup restore complete!");
            println!("Restored vault file: {}", restored_vault_path.display());

            print_next_steps(&[
                "Verify the restored vault can be opened with the correct password",
                "Check that all expected data is present in the vault",
                &format!(
                    "Securely delete the intermediate bundle and share files:\n       - {}",
                    in_progress_bundle_path.display()
                ),
            ]);

            Ok(())
        }
    };

    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }

    Ok(())
}

/// Print a step completion message
fn print_step(step: &str, description: &str) {
    println!("\n{step} complete: {description}");
}

/// Print a list of next steps
fn print_next_steps(steps: &[&str]) {
    println!("\nNEXT STEPS:");
    for (i, step) in steps.iter().enumerate() {
        println!("  {}. {}", i + 1, step);
    }
    println!();
}
