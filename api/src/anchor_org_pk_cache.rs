use std::sync::Arc;

use common::protocol::keys::AnchorOrganizationPublicKey;
use tokio::sync::{RwLock, RwLockReadGuard};

#[derive(Clone)]
pub struct AnchorOrganizationPublicKeyCache(Arc<RwLock<Vec<AnchorOrganizationPublicKey>>>);

impl Default for AnchorOrganizationPublicKeyCache {
    fn default() -> Self {
        Self(Arc::new(RwLock::new(vec![])))
    }
}

impl AnchorOrganizationPublicKeyCache {
    pub async fn set(&self, keys: Vec<AnchorOrganizationPublicKey>) {
        let mut guard = self.0.write().await;
        *guard = keys;
    }

    pub async fn get(&self) -> RwLockReadGuard<'_, Vec<AnchorOrganizationPublicKey>> {
        self.0.read().await
    }
}
