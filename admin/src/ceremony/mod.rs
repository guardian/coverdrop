pub mod anchor_public_key_bundle;
pub mod public_key_forms_bundle;
mod script;
mod set_system_status_available_bundle;
mod state_machine;
mod tests;

use anchor_public_key_bundle::*;
use chrono::{DateTime, Utc};
use common::{
    api::api_client::ApiClient,
    aws::ssm::{
        client::SsmClient, parameters::ANCHOR_ORG_PK_SSM_PARAMETER, prefix::ParameterPrefix,
    },
    crypto::keys::untrusted::signing::UntrustedSignedPublicSigningKey,
    protocol::{keys::UntrustedAnchorOrganizationPublicKey, roles::AnchorOrganization},
    throttle::Throttle,
    time,
};
use public_key_forms_bundle::*;
use serde::de::DeserializeOwned;
use state_machine::*;

use self::set_system_status_available_bundle::{
    save_set_system_status_available_bundle, SetSystemStatusAvailableBundle,
};
use common::clap::AwsConfig;
use std::{
    collections::HashMap,
    fs::File,
    num::NonZeroU8,
    path::{Path, PathBuf},
    time::Duration,
};

const PUBLIC_KEY_FORMS_BUNDLE_FILENAME: &str = "public_key_forms.bundle.json";
const SET_SYSTEM_STATUS_AVAILABLE_BUNDLE_FILENAME: &str = "set_system_status_available.bundle.json";
pub const ANCHOR_ORGANIZATION_PUBLIC_KEY_BUNDLE_FILENAME: &str =
    "anchor_organization_public_key.bundle.json";

/// Prompts the user to type in a confirmation word in order to proceed
/// with the next step of the ceremony
fn ask_user_to_confirm(message: &str, assume_yes: bool) -> anyhow::Result<()> {
    if assume_yes {
        return Ok(());
    }

    const CONFIRMATION_WORD: &str = "yes";

    loop {
        println!("{message}\nType '{CONFIRMATION_WORD}' to confirm.");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() == CONFIRMATION_WORD {
            break;
        }
    }

    Ok(())
}

pub async fn run_setup_ceremony(
    output_directory: impl AsRef<Path>,
    covernode_count: NonZeroU8,
    assume_yes: bool,
    covernode_db_password: String,
    org_key_pair_path: Option<PathBuf>,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    if !output_directory.as_ref().is_dir() {
        anyhow::bail!("Provided path to output directory is not a directory");
    }

    let mut state = CeremonyState::new(
        &output_directory,
        covernode_count,
        assume_yes,
        covernode_db_password,
        org_key_pair_path,
        now,
    );

    let mut ceremony = CeremonyStep::new();

    // Start the ceremony - Step Zero
    ceremony.execute(&mut state).await?;

    // Walk through every step of the ceremony until it's complete
    while let Some(step) = ceremony.next() {
        step.execute(&mut state).await?;
        ceremony = step;
    }

    // Now we've got our keys we need to plan what we're going to do with them once we're outside the ceremony.

    // Key Role                | Secret post-ceremony action              | Public post-ceremony action
    // ------------------------+------------------------------------------+-----------------------------------
    // Organization            | Keep key pair in safe                    | Distribute trusted public key
    // CoverNode Provisioning  | Store key pair for identity service      | Upload public key to API via form
    // Journalist Provisioning | Store key pair for identity service      | Upload public key to API via form
    // CoverNode Identity      | Store key pair for CoverNode initial key | Upload public key to API via form
    // System Status           | Store key pair for developers            | Set initial status in API via form

    // The post-ceremony actions are performed over several "bundles" of data. Some of which can be exposed to the internet:
    //   - Trusted public key bundle | a single key bundle for distributing the trusted organization key to our various build systems
    //                               | this is separate from the upload bundle because it must be handled differently
    //                               | e.g. the API must be seeded with a trusted key before it can accept upload forms.
    //   - Upload bundle             | for uploading public keys and system status to the API in a production system

    // The follow bundles can be on a machine which has internet access, but must NOT be published
    //   - System status key pairs         | Distributed to developers to allow them to rotate keys
    //   - Identity provisioning key pairs | for distributing to the identity services
    //   - CoverNode initial key pair      | to provide the CoverNode services with their initial identities.

    // The following bundles must never be exposed to the internet, and should be treated with care:
    //   - Organization key pair           | for high security storage of the top level key pair

    Ok(())
}

/// Check that the bundles necessary for the post-ceremony actions exist on disk.
/// If they don't, an error message listing the missing files is displayed.
fn check_post_ceremony_bundles_exist(
    bundle_directory_path: impl AsRef<Path>,
) -> Result<(), anyhow::Error> {
    let base_path = bundle_directory_path.as_ref();

    let all_bundles = [
        base_path.join(ANCHOR_ORGANIZATION_PUBLIC_KEY_BUNDLE_FILENAME),
        base_path.join(PUBLIC_KEY_FORMS_BUNDLE_FILENAME),
        base_path.join(SET_SYSTEM_STATUS_AVAILABLE_BUNDLE_FILENAME),
    ];
    let all_bundles = all_bundles
        .iter()
        .map(|p| (p, p.exists()))
        .collect::<HashMap<_, _>>();

    let all_bundles_exist = all_bundles.iter().all(|(_, exists)| *exists);

    if !all_bundles_exist {
        let mut error_message = String::new();
        error_message.push_str(
            "The following files are required to run the post-ceremony actions, but were not found:\n",
        );
        for (path, _) in all_bundles.iter().filter(|&(_, exists)| !exists) {
            error_message.push_str(&path.to_string_lossy());
            error_message.push('\n');
        }
        anyhow::bail!("{}", error_message);
    }

    Ok(())
}

pub fn read_bundle_from_disk<T>(bundle_path: impl AsRef<Path>) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    let reader = File::open(bundle_path)?;
    let bundle = serde_json::from_reader(reader)?;
    Ok(bundle)
}

pub async fn put_anchor_org_pk_parameter(
    ssm_client: &SsmClient,
    parameter_prefix: &ParameterPrefix,
    anchor_org_pk: &UntrustedSignedPublicSigningKey<AnchorOrganization>,
) -> anyhow::Result<()> {
    let anchor_org_pk = serde_json::to_string(&anchor_org_pk)?;

    ssm_client
        .put_string_parameter(
            parameter_prefix.get_parameter(ANCHOR_ORG_PK_SSM_PARAMETER),
            anchor_org_pk,
            "The organization's trusted public key",
        )
        .await?;

    Ok(())
}

pub async fn api_has_anchor_org_pk(
    api_client: &ApiClient,
    anchor_org_pk: &UntrustedAnchorOrganizationPublicKey,
) -> anyhow::Result<bool> {
    let has_key = api_client
        .get_public_keys()
        .await?
        .keys
        .org_pk_iter()
        // Converting to the key to a TOFU key allows us to not have to revalidated the entire hierarchy every time
        // this functionally does the same thing as validating but requires less cloning
        .any(|api_org_pk| api_org_pk.to_tofu_anchor() == *anchor_org_pk);

    Ok(has_key)
}

pub async fn upload_keys_to_api(
    bundle_path: impl AsRef<Path>,
    api_client: &ApiClient,
    aws_config: &AwsConfig,
    parameter_prefix: &Option<ParameterPrefix>,
) -> anyhow::Result<()> {
    // Make sure all files needed for post-ceremony actions exist
    check_post_ceremony_bundles_exist(&bundle_path)?;

    let base_path = bundle_path.as_ref();

    // Trusted org pk bundle
    let bundle = base_path.join(ANCHOR_ORGANIZATION_PUBLIC_KEY_BUNDLE_FILENAME);
    let anchor_org_pk_bundle = read_bundle_from_disk::<AnchorOrganizationPublicKeyBundle>(bundle)?;

    match &parameter_prefix {
        Some(parameter_prefix) => {
            let ssm_client = SsmClient::new(aws_config.region.to_owned(), aws_config.profile.to_owned()).await;
            put_anchor_org_pk_parameter(
                &ssm_client,
                parameter_prefix,
                &anchor_org_pk_bundle.anchor_org_pk,
            )
            .await?;
        },
        _ => println!("Running locally, not inserting trusted organization public key in the AWS parameter store")
    }

    let started_polling = time::now();
    let max_duration = chrono::Duration::minutes(10);
    let mut throttle = Throttle::new(Duration::from_secs(10));

    while !api_has_anchor_org_pk(api_client, &anchor_org_pk_bundle.anchor_org_pk).await? {
        let elapsed = time::now() - started_polling;

        println!(
            "Waiting for new organization key to appear in API (waited {}s/{}s)",
            elapsed.num_seconds(),
            max_duration.num_seconds()
        );

        if elapsed > max_duration {
            anyhow::bail!(
                "Trusted organization key does not appear in API after {} seconds of checking",
                elapsed.num_seconds()
            );
        }

        throttle.wait().await;
    }

    // Public keys bundle
    let bundle = base_path.join(PUBLIC_KEY_FORMS_BUNDLE_FILENAME);
    let bundle = read_bundle_from_disk::<PublicKeyFormsBundle>(bundle)?;

    api_client
        .post_journalist_provisioning_pk(bundle.journalist_provisioning_pk_form)
        .await?;

    api_client
        .post_covernode_provisioning_pk(bundle.covernode_provisioning_pk_form)
        .await?;

    api_client.post_admin_pk(bundle.admin_pk_form).await?;

    // Set system status available
    let bundle = base_path.join(SET_SYSTEM_STATUS_AVAILABLE_BUNDLE_FILENAME);
    let bundle = read_bundle_from_disk::<SetSystemStatusAvailableBundle>(bundle)?;

    api_client
        .post_status_event_form(bundle.set_system_status_form)
        .await?;

    Ok(())
}
