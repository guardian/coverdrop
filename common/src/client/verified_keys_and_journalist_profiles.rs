use chrono::{DateTime, Utc};

use crate::{
    api::models::{
        journalist_id::JournalistIdentity,
        untrusted_keys_and_journalist_profiles::UntrustedKeysAndJournalistProfiles,
    },
    epoch::Epoch,
    protocol::keys::{AnchorOrganizationPublicKey, OrganizationPublicKeyFamilyList},
};

use super::JournalistProfile;

pub struct VerifiedKeysAndJournalistProfiles {
    pub journalist_profiles: Vec<JournalistProfile>,
    pub default_journalist_id: Option<JournalistIdentity>,
    pub keys: OrganizationPublicKeyFamilyList,
    pub max_epoch: Epoch,
}

impl VerifiedKeysAndJournalistProfiles {
    pub fn from_untrusted(
        untrusted: UntrustedKeysAndJournalistProfiles,
        anchor_org_pks: &[AnchorOrganizationPublicKey],
        now: DateTime<Utc>,
    ) -> Self {
        let keys =
            OrganizationPublicKeyFamilyList::from_untrusted(untrusted.keys, anchor_org_pks, now);

        // If the default journalist is not in the list of journalists, hide it
        let default_journalist_id = untrusted.default_journalist_id.filter(|id| {
            keys.journalist_id_iter()
                .any(|journalist_with_key_id| journalist_with_key_id == id)
        });

        Self {
            journalist_profiles: untrusted.journalist_profiles,
            default_journalist_id,
            keys,
            max_epoch: untrusted.max_epoch,
        }
    }

    pub fn find_profile(&self, id: &JournalistIdentity) -> Option<&JournalistProfile> {
        self.journalist_profiles
            .iter()
            .find(|profile| profile.id == *id)
    }

    pub fn to_untrusted(&self) -> UntrustedKeysAndJournalistProfiles {
        let keys = self.keys.to_untrusted();
        UntrustedKeysAndJournalistProfiles::new(
            self.journalist_profiles.clone(),
            self.default_journalist_id.clone(),
            keys,
            self.max_epoch,
        )
    }
}
