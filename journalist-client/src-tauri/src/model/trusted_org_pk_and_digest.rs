use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TrustedOrganizationPublicKeyAndDigest {
    pub pk_hex: String,
    pub digest: String,
}

impl TrustedOrganizationPublicKeyAndDigest {
    pub fn new(pk_hex: String, digest: String) -> Self {
        Self { pk_hex, digest }
    }
}
