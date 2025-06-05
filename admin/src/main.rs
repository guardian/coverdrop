use admin::copy_anchor_org_pk;
use admin::copy_identity_api_key_pairs;
use admin::delete_journalist_form;
use admin::generate_admin_key_pair;
use admin::generate_constant_files;
use admin::generate_covernode_database;
use admin::generate_covernode_provisioning_key_pair;
use admin::generate_journalist;
use admin::generate_journalist_provisioning_key_pair;
use admin::generate_test_vectors;
use admin::post_log_config_form;
use admin::reseed_journalist_vault_id_key_pair;
use admin::run_post_ceremony_actions;
use admin::run_setup_ceremony;
use admin::submit_delete_journalist_form;
use admin::update_journalist;
use admin::update_system_status;
use admin::{
    copy_admin_key_pair, generate_covernode_identity_key_pair,
    generate_covernode_messaging_key_pair, generate_organization_key_pair,
};
use clap::Parser;
use cli::{Cli, Commands};
use common::api::api_client::ApiClient;
use common::clap::validate_password_from_args;
use common::crypto::human_readable_digest;
use common::crypto::keys::public_key::PublicKey;
use common::crypto::pbkdf::DEFAULT_PASSPHRASE_WORDS;
use common::generators::PasswordGenerator;
use common::time;
use common::tracing::init_tracing;
use journalist_vault::JournalistVault;
use journalist_vault::PASSWORD_EXTENSION;
use tokio::fs;

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
        Commands::RunPostCeremonyCopyTrustedOrganizationKey {
            bundle_directory_path,
            target_directory,
        } => copy_anchor_org_pk(bundle_directory_path, target_directory, time::now()).await,
        Commands::RunPostCeremonyCopyAdminKeyPair {
            bundle_directory_path,
            target_directory,
        } => copy_admin_key_pair(bundle_directory_path, target_directory).await,
        Commands::RunPostCeremonyCopyIdentityApiKeyPairs {
            bundle_directory_path,
            target_directory,
        } => copy_identity_api_key_pairs(bundle_directory_path, target_directory).await,
        Commands::RunPostCeremonyUpload {
            bundle_directory_path,
            api_url,
            aws_config,
            parameter_prefix,
        } => {
            let api_client = ApiClient::new(api_url);

            run_post_ceremony_actions(
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
        Commands::GenerateJournalistProvisioningKeyPair {
            keys_path,
            api_url,
            do_not_upload_to_api,
        } => {
            let api_client = ApiClient::new(api_url);
            generate_journalist_provisioning_key_pair(
                keys_path,
                api_client,
                do_not_upload_to_api,
                false,
                time::now(),
            )
            .await
        }
        Commands::GenerateCoverNodeProvisioningKeyPair {
            keys_path,
            api_url,
            do_not_upload_to_api,
        } => {
            let api_client = ApiClient::new(api_url);
            generate_covernode_provisioning_key_pair(
                keys_path,
                api_client,
                do_not_upload_to_api,
                false,
                time::now(),
            )
            .await
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
    };

    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }

    Ok(())
}
