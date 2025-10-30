//! General email utils

use std::collections::HashMap;

use common::{
    api::models::{covernode_id::CoverNodeIdentity, journalist_id::JournalistIdentity},
    aws::ssm::{client::SsmClient, parameters::NOTIFICATION_EMAIL_SENDER, prefix::ParameterPrefix},
    crypto::keys::{role::Role, signed::SignedKey},
    protocol::keys::{
        CoverNodeIdPublicKey, CoverNodeMessagingPublicKey, CoverNodeProvisioningPublicKey,
        JournalistIdPublicKey, JournalistMessagingPublicKey, JournalistProvisioningPublicKey,
        OrganizationPublicKey,
    },
};

use crate::expiry_state::ExpiryState;

pub async fn source_email(
    ssm_client: &SsmClient,
    parameter_prefix: &ParameterPrefix,
) -> anyhow::Result<String> {
    let parameter = parameter_prefix.get_parameter(NOTIFICATION_EMAIL_SENDER);
    let email = ssm_client.get_parameter(&parameter).await?;

    Ok(email)
}

fn add_text_for_should_have_rotated_key<R, PK>(text: &mut String, title: &str, pk: &PK)
where
    R: Role,
    PK: SignedKey<R>,
{
    let rotation_time = pk.rotation_notification_time();
    let expiry_time = pk.not_valid_after();
    text.push_str(title);
    text.push_str(" | Should have rotated at ");
    text.push_str(&rotation_time.format("%Y-%m-%d %H:%M").to_string());
    text.push_str(", expiring at ");
    text.push_str(&expiry_time.format("%Y-%m-%d %H:%M").to_string());
    text.push('\n');
}

fn add_text_for_expired_key(text: &mut String, title: &str) {
    text.push_str(title);
    text.push_str(" | ");
    text.push_str("Expired");
    text.push('\n');
}

fn add_text_for_expiring_pk<R, PK>(text: &mut String, title: &str, expiry_state: ExpiryState<&PK>)
where
    R: Role,
    PK: SignedKey<R>,
{
    match expiry_state {
        ExpiryState::Nominal => {
            // No concern about expiry - leave it out the email
        }
        ExpiryState::ShouldHaveRotated(pk) => add_text_for_should_have_rotated_key(text, title, pk),
        ExpiryState::Expired => add_text_for_expired_key(text, title),
    }
}

fn add_text_for_expiring_pks_with_identities<Identity, R, PK>(
    text: &mut String,
    title: &str,
    expiries: HashMap<&Identity, ExpiryState<&PK>>,
) where
    Identity: AsRef<String>,
    R: Role,
    PK: SignedKey<R>,
{
    if expiries.iter().all(|e| matches!(e.1, ExpiryState::Nominal)) {
        return;
    }

    text.push('\n');
    text.push_str(title);
    text.push('\n');
    for _ in 0..title.len() {
        text.push('-')
    }
    text.push('\n');

    for (id, expiry_state_pk) in expiries {
        match expiry_state_pk {
            ExpiryState::Nominal => {
                // No concern about expiry - leave it out the email
            }
            ExpiryState::ShouldHaveRotated(pk) => {
                add_text_for_should_have_rotated_key(text, id.as_ref(), pk)
            }
            ExpiryState::Expired => {
                add_text_for_expired_key(text, id.as_ref());
            }
        }
    }
    text.push_str("\n\n");
}

pub fn create_email_body(
    expiring_org_pk: ExpiryState<&OrganizationPublicKey>,
    expiring_covernode_provisioning_pk: ExpiryState<&CoverNodeProvisioningPublicKey>,
    expiring_journalist_provisioning_pk: ExpiryState<&JournalistProvisioningPublicKey>,
    expiring_covernode_id_pks: HashMap<&CoverNodeIdentity, ExpiryState<&CoverNodeIdPublicKey>>,
    expiring_covernode_msg_pks: HashMap<
        &CoverNodeIdentity,
        ExpiryState<&CoverNodeMessagingPublicKey>,
    >,
    expiring_journalist_id_pks: HashMap<&JournalistIdentity, ExpiryState<&JournalistIdPublicKey>>,
    expiring_journalist_msg_pks: HashMap<
        &JournalistIdentity,
        ExpiryState<&JournalistMessagingPublicKey>,
    >,
) -> Option<String> {
    let mut text = String::new();

    add_text_for_expiring_pk(&mut text, "Organization", expiring_org_pk);
    add_text_for_expiring_pk(
        &mut text,
        "CoverNode Provisioning",
        expiring_covernode_provisioning_pk,
    );
    add_text_for_expiring_pk(
        &mut text,
        "Journalist Provisioning",
        expiring_journalist_provisioning_pk,
    );
    add_text_for_expiring_pks_with_identities(
        &mut text,
        "CoverNode identity",
        expiring_covernode_id_pks,
    );
    add_text_for_expiring_pks_with_identities(
        &mut text,
        "CoverNode messaging",
        expiring_covernode_msg_pks,
    );

    add_text_for_expiring_pks_with_identities(
        &mut text,
        "Journalist identity",
        expiring_journalist_id_pks,
    );
    add_text_for_expiring_pks_with_identities(
        &mut text,
        "Journalist messaging",
        expiring_journalist_msg_pks,
    );

    if !text.is_empty() {
        Some(text)
    } else {
        None
    }
}
