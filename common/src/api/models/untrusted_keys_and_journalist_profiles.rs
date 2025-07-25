use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    client::{JournalistProfile, VerifiedKeysAndJournalistProfiles},
    epoch::Epoch,
    protocol::keys::{
        AnchorOrganizationPublicKey, UntrustedOrganizationPublicKey,
        UntrustedOrganizationPublicKeyFamilyList,
    },
};

use super::journalist_id::JournalistIdentity;

/// Not a huge fan of this structure - we should perhaps separate out the keys and the journalist info
/// there's potentially a lot of data in the journalist info that doesn't change very much which makes
/// pulling them both down together a bit wasteful.
#[derive(Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(deny_unknown_fields)]
pub struct UntrustedKeysAndJournalistProfiles {
    pub journalist_profiles: Vec<JournalistProfile>,
    pub default_journalist_id: Option<JournalistIdentity>,
    pub keys: UntrustedOrganizationPublicKeyFamilyList,
    pub max_epoch: Epoch,
}

impl UntrustedKeysAndJournalistProfiles {
    pub fn new(
        journalist_profiles: Vec<JournalistProfile>,
        default_journalist_id: Option<JournalistIdentity>,
        keys: UntrustedOrganizationPublicKeyFamilyList,
        max_epoch: Epoch,
    ) -> Self {
        Self {
            journalist_profiles,
            default_journalist_id,
            keys,
            max_epoch,
        }
    }

    pub fn into_trusted(
        self,
        anchor_org_pks: &[AnchorOrganizationPublicKey],
        now: DateTime<Utc>,
    ) -> VerifiedKeysAndJournalistProfiles {
        VerifiedKeysAndJournalistProfiles::from_untrusted(self, anchor_org_pks, now)
    }

    /// Get all the untrusted keys - useful for trust on first use.
    pub fn untrusted_org_pk_iter(&self) -> impl Iterator<Item = &UntrustedOrganizationPublicKey> {
        self.keys
            .0
            .iter()
            .map(|org_pk_family| &org_pk_family.org_pk)
    }

    pub fn profile(&self, id: &JournalistIdentity) -> Option<&JournalistProfile> {
        self.journalist_profiles
            .iter()
            .find(|profile| profile.id == *id)
    }
}
