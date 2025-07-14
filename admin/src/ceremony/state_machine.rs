use chrono::{DateTime, Utc};

use common::{
    api::{forms::PostCoverNodeIdPublicKeyForm, models::covernode_id::CoverNodeIdentity},
    crypto::keys::signing::{SignedPublicSigningKey, SigningKeyPair},
    protocol::{
        keys::{
            generate_covernode_id_key_pair, generate_covernode_provisioning_key_pair,
            generate_journalist_provisioning_key_pair, generate_organization_key_pair,
            load_org_key_pairs, CoverNodeProvisioningKeyPair, JournalistProvisioningKeyPair,
        },
        roles::Organization,
    },
    system::keys::{generate_admin_key_pair, AdminKeyPair},
};
use covernode_database::Database;
use std::{num::NonZeroU8, path::PathBuf};

use super::{script::*, *};

#[derive(Clone)]
pub struct CeremonyState {
    pub output_directory: PathBuf,
    pub covernode_count: NonZeroU8,
    pub assume_yes: bool,
    pub covernode_db_password: String,
    pub now: DateTime<Utc>,

    // Key pairs
    pub org_key_pair: Option<SigningKeyPair<Organization, SignedPublicSigningKey<Organization>>>,
    pub journalist_provisioning_key_pair: Option<JournalistProvisioningKeyPair>,
    pub covernode_provisioning_key_pair: Option<CoverNodeProvisioningKeyPair>,
    pub admin_key_pair: Option<AdminKeyPair>,

    // Bundles
    pub org_key_pair_bundle: Option<PathBuf>,
    pub journalist_provisioning_key_pair_bundle: Option<PathBuf>,
    pub covernode_provisioning_key_pair_bundle: Option<PathBuf>,
    pub admin_key_pair_bundle: Option<PathBuf>,
    pub set_system_status_available_bundle: Option<PathBuf>,
    pub anchor_org_pk_bundle: Option<PathBuf>,
    pub public_key_forms_bundle: Option<PathBuf>,
}

impl CeremonyState {
    pub fn new(
        output_directory: impl AsRef<Path>,
        covernode_count: NonZeroU8,
        assume_yes: bool,
        covernode_db_password: String,
        org_key_pair_path: Option<PathBuf>,
        now: DateTime<Utc>,
    ) -> Self {
        let mut org_key_pair = None;

        if let Some(org_key_pair_path) = org_key_pair_path {
            let org_key_pairs =
                load_org_key_pairs(org_key_pair_path, now).expect("read org key pairs");

            if org_key_pairs.len() != 1 {
                panic!("Must have exactly one org key pair in directory")
            }

            org_key_pair = org_key_pairs.into_iter().next();
        }

        Self {
            output_directory: output_directory.as_ref().to_owned(),
            covernode_count,
            assume_yes,
            covernode_db_password,
            now,
            // Key pairs
            org_key_pair,
            journalist_provisioning_key_pair: None,
            covernode_provisioning_key_pair: None,
            admin_key_pair: None,
            // Bundles
            org_key_pair_bundle: None,
            journalist_provisioning_key_pair_bundle: None,
            covernode_provisioning_key_pair_bundle: None,
            admin_key_pair_bundle: None,
            set_system_status_available_bundle: None,
            anchor_org_pk_bundle: None,
            public_key_forms_bundle: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CeremonyStep {
    /// The initial step prompting the user to start the ceremony
    Zero,
    /// Generates the trusted organization key pair bundle
    One,
    /// Generates the journalist provisioning key pair bundle
    Two,
    /// Generates the CoverNode provisioning key pair bundle
    Three,
    /// Generates a CoverNode key database for each CoverNode
    Four,
    /// Generates the system status key pair bundle
    Five,
    /// Generates the "set system status available" bundle
    Six,
    /// Generates the trusted organization public key bundle
    Seven,
    /// Generates the public keys form bundle
    Eight,
    /// Final step prompting the user to delete files
    Nine,
}

impl CeremonyStep {
    pub fn new() -> Self {
        CeremonyStep::Zero
    }
}

impl CeremonyStep {
    /// Move on to the next step of the ceremony
    pub fn next(&self) -> Option<CeremonyStep> {
        match self {
            CeremonyStep::Zero => Some(CeremonyStep::One),
            CeremonyStep::One => Some(CeremonyStep::Two),
            CeremonyStep::Two => Some(CeremonyStep::Three),
            CeremonyStep::Three => Some(CeremonyStep::Four),
            CeremonyStep::Four => Some(CeremonyStep::Five),
            CeremonyStep::Five => Some(CeremonyStep::Six),
            CeremonyStep::Six => Some(CeremonyStep::Seven),
            CeremonyStep::Seven => Some(CeremonyStep::Eight),
            CeremonyStep::Eight => Some(CeremonyStep::Nine),
            CeremonyStep::Nine => None,
        }
    }

    /// Execute the current step
    pub async fn execute(&self, state: &mut CeremonyState) -> anyhow::Result<()> {
        let CeremonyState {
            output_directory,
            covernode_count,
            assume_yes,
            covernode_db_password,
            now,
            ..
        } = state;

        if !*assume_yes {
            println!("\n=== Step {self:?} ===");
        }

        match self {
            CeremonyStep::Zero => {
                ask_user_to_confirm(START_CEREMONY, *assume_yes)?;
                Ok(())
            }
            CeremonyStep::One => {
                let (org_key_pair, org_key_pair_preprovided) = match state.org_key_pair {
                    Some(ref org_key_pair) => (org_key_pair.clone(), true),
                    None => {
                        ask_user_to_confirm(PRE_ANCHOR_ORG_KEY_PAIR, *assume_yes)?;
                        let org_key_pair = generate_organization_key_pair(*now);
                        state.org_key_pair = Some(org_key_pair.clone());
                        (org_key_pair, false)
                    }
                };

                let bundle = save_organization_key_pair_bundle(output_directory, &org_key_pair)?;
                state.org_key_pair_bundle = Some(bundle);

                if org_key_pair_preprovided {
                    ask_user_to_confirm(SKIP_CREATE_ANCHOR_ORG_KEY_PAIR, *assume_yes)?;
                } else {
                    ask_user_to_confirm(POST_ANCHOR_ORG_KEY_PAIR, *assume_yes)?;
                }

                Ok(())
            }
            CeremonyStep::Two => {
                let journalist_provisioning_key_pair = generate_journalist_provisioning_key_pair(
                    state.org_key_pair.as_ref().unwrap(),
                    *now,
                );
                state.journalist_provisioning_key_pair =
                    Some(journalist_provisioning_key_pair.clone());

                let bundle = save_journalist_provisioning_bundle(
                    output_directory,
                    &journalist_provisioning_key_pair,
                )?;
                state.journalist_provisioning_key_pair_bundle = Some(bundle);

                ask_user_to_confirm(JOURNALIST_PROVISIONING_KEY_PAIR, *assume_yes)?;
                Ok(())
            }
            CeremonyStep::Three => {
                let covernode_provisioning_key_pair = generate_covernode_provisioning_key_pair(
                    state.org_key_pair.as_ref().unwrap(),
                    *now,
                );
                state.covernode_provisioning_key_pair =
                    Some(covernode_provisioning_key_pair.clone());

                let bundle = save_covernode_provisioning_bundle(
                    output_directory,
                    &covernode_provisioning_key_pair,
                )?;
                state.covernode_provisioning_key_pair_bundle = Some(bundle);

                ask_user_to_confirm(COVERNODE_PROVISIONING_KEY_PAIR, *assume_yes)?;
                Ok(())
            }
            CeremonyStep::Four => {
                for covernode_id in 1..=(*covernode_count).into() {
                    let covernode_identity = CoverNodeIdentity::from_node_id(covernode_id);

                    let db_path = output_directory.join(format!("{covernode_identity}.db"));
                    let db = Database::open(&db_path, covernode_db_password).await?;

                    let covernode_id_key_pair = generate_covernode_id_key_pair(
                        state.covernode_provisioning_key_pair.as_ref().unwrap(),
                        *now,
                    );

                    let now = time::now();

                    let form = PostCoverNodeIdPublicKeyForm::new(
                        covernode_identity.clone(),
                        covernode_id_key_pair.public_key().to_untrusted(),
                        state.covernode_provisioning_key_pair.as_ref().unwrap(),
                        now,
                    )?;

                    db.insert_setup_bundle(&form, &covernode_id_key_pair, now)
                        .await?;
                }

                ask_user_to_confirm(COVERNODE_DB, *assume_yes)?;

                Ok(())
            }

            CeremonyStep::Five => {
                let admin_key_pair =
                    generate_admin_key_pair(state.org_key_pair.as_ref().unwrap(), *now);
                state.admin_key_pair = Some(admin_key_pair.clone());

                let bundle = save_admin_key_pair_bundle(output_directory, &admin_key_pair)?;
                state.admin_key_pair_bundle = Some(bundle);

                ask_user_to_confirm(ADMIN_KEY_PAIR, *assume_yes)?;
                Ok(())
            }
            CeremonyStep::Six => {
                let bundle = save_set_system_status_available_bundle(
                    output_directory,
                    state.admin_key_pair.as_ref().unwrap(),
                )?;
                state.set_system_status_available_bundle = Some(bundle);

                ask_user_to_confirm(SET_SYSTEM_STATUS_BUNDLE, *assume_yes)?;
                Ok(())
            }
            CeremonyStep::Seven => {
                let anchor_org_pk = state
                    .org_key_pair
                    .as_ref()
                    .unwrap()
                    .public_key()
                    .clone()
                    .into_anchor();

                let bundle = save_anchor_public_key_bundle(output_directory, &anchor_org_pk)?;
                state.anchor_org_pk_bundle = Some(bundle);

                ask_user_to_confirm(ANCHOR_ORG_PK_BUNDLE, *assume_yes)?;
                Ok(())
            }
            CeremonyStep::Eight => {
                let bundle = save_public_key_forms_bundle(
                    output_directory,
                    state.org_key_pair.as_ref().unwrap(),
                    state.journalist_provisioning_key_pair.as_ref().unwrap(),
                    state.covernode_provisioning_key_pair.as_ref().unwrap(),
                    state.admin_key_pair.as_ref().unwrap(),
                )?;
                state.public_key_forms_bundle = Some(bundle);

                ask_user_to_confirm(PUBLIC_KEY_FORMS_BUNDLE, *assume_yes)?;
                Ok(())
            }
            CeremonyStep::Nine => {
                ask_user_to_confirm(DELETE_KEY_MATERIAL, *assume_yes)?;

                Ok(())
            }
        }
    }
}
