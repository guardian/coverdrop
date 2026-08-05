use serde::{Deserialize, Serialize};

use crate::identity_api::forms::post_rotate_sentinel_id::RotateSentinelIdPublicKeyForm;

use super::sentinel_id::SentinelIdentity;

#[derive(Serialize, Deserialize)]
pub struct SentinelIdAndPublicKeyRotationForm {
    pub sentinel_id: SentinelIdentity,
    pub form: RotateSentinelIdPublicKeyForm,
}

impl SentinelIdAndPublicKeyRotationForm {
    pub fn new(sentinel_id: SentinelIdentity, form: RotateSentinelIdPublicKeyForm) -> Self {
        Self { sentinel_id, form }
    }
}
