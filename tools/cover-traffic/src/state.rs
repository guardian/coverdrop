use common::api::models::messages::journalist_to_covernode_message::EncryptedJournalistToCoverNodeMessage;
use common::api::models::messages::user_to_covernode_message::EncryptedUserToCoverNodeMessage;
use common::protocol::journalist::new_encrypted_cover_message_from_journalist_via_covernode;
use common::protocol::keys::{
    AnchorOrganizationPublicKey, CoverDropPublicKeyHierarchy, OrganizationPublicKeyFamilyList,
    UntrustedOrganizationPublicKeyFamilyList,
};
use common::protocol::user::new_encrypted_cover_message_from_user_via_covernode;
use common::time::now;
use std::sync::Arc;
use tokio::sync::RwLock;

/// The [CoverTrafficState] contains the key hierarchy that is used by the sending services.
/// It also contains the set of initially (TOFU) trusted organization keys that are used to verify
/// subsequent updates from the [KeyHierarchyUpdateService].
pub struct CoverTrafficState {
    key_hierarchy: Arc<RwLock<CoverDropPublicKeyHierarchy>>,
    tofu_anchor_org_keys: Vec<AnchorOrganizationPublicKey>,
}

impl CoverTrafficState {
    pub(crate) fn new(
        tofu_anchor_org_keys: Vec<AnchorOrganizationPublicKey>,
        untrusted_keys: UntrustedOrganizationPublicKeyFamilyList,
    ) -> Self {
        let verified_keys = OrganizationPublicKeyFamilyList::from_untrusted(
            untrusted_keys,
            &tofu_anchor_org_keys,
            now(),
        );
        CoverTrafficState {
            tofu_anchor_org_keys,
            key_hierarchy: Arc::new(RwLock::new(verified_keys)),
        }
    }

    pub(crate) async fn update_keys(
        &self,
        untrusted_keys: UntrustedOrganizationPublicKeyFamilyList,
    ) {
        let verified_keys = OrganizationPublicKeyFamilyList::from_untrusted(
            untrusted_keys,
            &self.tofu_anchor_org_keys,
            now(),
        );
        let mut write_guard = self.key_hierarchy.write().await;
        *write_guard = verified_keys
    }

    pub(crate) async fn create_user_to_journalist_cover_message(
        &self,
    ) -> anyhow::Result<EncryptedUserToCoverNodeMessage> {
        let read_guard = self.key_hierarchy.read().await;
        new_encrypted_cover_message_from_user_via_covernode(&read_guard)
    }

    pub(crate) async fn create_journalist_to_user_cover_message(
        &self,
    ) -> anyhow::Result<EncryptedJournalistToCoverNodeMessage> {
        let read_guard = self.key_hierarchy.read().await;
        new_encrypted_cover_message_from_journalist_via_covernode(&read_guard)
    }
}
