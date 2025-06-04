use chrono::{DateTime, Utc};
use common::api::api_client::ApiClient;
use common::protocol::keys::{AnchorOrganizationPublicKey, JournalistMessagingPublicKey};
use common::protocol::recipient_tag::RecipientTag;
use std::collections::HashMap;

pub struct RecipientTagKeyLookupTable {
    recipient_tag_to_public_key: HashMap<RecipientTag, JournalistMessagingPublicKey>,
}

impl RecipientTagKeyLookupTable {
    /// Creates a new look-up table that maps recipient tags to the respective public encryption
    /// keys. For this the key is duplicated to allow the look-up table to have a separate lifetime.
    pub async fn from_api(
        api_client: &ApiClient,
        anchor_org_pks: &[AnchorOrganizationPublicKey],
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let keys_and_profiles = api_client
            .get_public_keys()
            .await?
            .into_trusted(anchor_org_pks, now);

        let keys = keys_and_profiles.keys;
        let mut recipient_tag_to_public_key = HashMap::new();

        for (journalist_id, journalist_msg_pk) in keys.journalist_msg_pk_iter() {
            let recipient_tag = RecipientTag::from_journalist_id(journalist_id);

            let current_msg_pk = recipient_tag_to_public_key.get(&recipient_tag);

            // If either there's no existing messaging key in the map, or if the messaging key
            // in the map expires before the current candidate key
            if current_msg_pk.is_none()
                || current_msg_pk.is_some_and(|pk: &JournalistMessagingPublicKey| {
                    pk.not_valid_after < journalist_msg_pk.not_valid_after
                })
            {
                recipient_tag_to_public_key.insert(recipient_tag, journalist_msg_pk.clone());
            }
        }

        Ok(RecipientTagKeyLookupTable {
            recipient_tag_to_public_key,
        })
    }

    pub fn get(&self, recipient_tag: &RecipientTag) -> Option<&JournalistMessagingPublicKey> {
        self.recipient_tag_to_public_key.get(recipient_tag)
    }
}
