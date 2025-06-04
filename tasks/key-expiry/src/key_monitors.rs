use std::{collections::HashMap, hash::Hash};

use chrono::Days;
use common::crypto::keys::{role::Role, signed::SignedKey};

use crate::expiry_state::ExpiryState;

pub fn check_pks_with_identifiers<'a, Identifier, R, SignedPublicKey>(
    all_ids: &'a [&Identifier],
    keys: impl Iterator<Item = (&'a Identifier, &'a SignedPublicKey)>,
    notification_cadence: &[Days],
) -> HashMap<&'a Identifier, ExpiryState<&'a SignedPublicKey>>
where
    Identifier: AsRef<String> + Eq + Hash,
    R: Role + 'static,
    SignedPublicKey: SignedKey<R>,
{
    let mut expiry_states = HashMap::<&Identifier, _, _>::new();

    for (id, pk) in keys {
        let hex_pk = hex::encode(&pk.as_bytes()[0..4]);

        if let Some(expiry_day) = notification_cadence
            .iter()
            .find(|&&day| pk.is_not_valid_after(common::time::now() + day))
        {
            tracing::warn!(
                "⏰ The {} key {} for {} will expire within {:?}",
                R::display(),
                hex_pk,
                id.as_ref(),
                expiry_day
            );

            expiry_states.insert(id, ExpiryState::ShouldHaveRotated(pk));
        } else {
            tracing::info!(
                "✅ The {} key {} for {} is not due to expire soon",
                R::display(),
                hex_pk,
                id.as_ref(),
            );

            expiry_states.insert(id, ExpiryState::Nominal);
        }
    }

    for id in all_ids.iter() {
        if !expiry_states.contains_key(id) {
            expiry_states.insert(id, ExpiryState::Expired);
        }
    }

    expiry_states
}

pub fn check_pk<'a, R, SignedPublicKey>(
    pk: Option<&'a SignedPublicKey>,
    notification_cadence: &[Days],
) -> ExpiryState<&'a SignedPublicKey>
where
    R: Role + 'static,
    SignedPublicKey: SignedKey<R>,
{
    if let Some(pk) = pk {
        let hex_pk = hex::encode(&pk.as_bytes()[0..4]);

        if let Some(expiry_day) = notification_cadence
            .iter()
            .find(|&&day| pk.is_not_valid_after(common::time::now() + day))
        {
            tracing::warn!(
                "⏰ The {} key {} will expire within {:?}",
                R::display(),
                hex_pk,
                expiry_day
            );

            ExpiryState::ShouldHaveRotated(pk)
        } else {
            tracing::info!(
                "✅ The {} key {} is not due to expire soon",
                R::display(),
                hex_pk
            );
            ExpiryState::Nominal
        }
    } else {
        tracing::warn!("☠️ The {} key has already expired!", R::display());
        ExpiryState::Expired
    }
}
