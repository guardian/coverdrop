use serde::{Deserialize, Serialize};

use crate::identity_api::forms::post_rotate_journalist_id::RotateJournalistIdPublicKeyForm;

use super::journalist_id::JournalistIdentity;

#[derive(Serialize, Deserialize)]
pub struct JournalistIdAndPublicKeyRotationForm {
    pub journalist_id: JournalistIdentity,
    pub form: RotateJournalistIdPublicKeyForm,
}

impl JournalistIdAndPublicKeyRotationForm {
    pub fn new(journalist_id: JournalistIdentity, form: RotateJournalistIdPublicKeyForm) -> Self {
        Self {
            journalist_id,
            form,
        }
    }
}
