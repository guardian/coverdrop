use crate::protocol::keys::PublishedBackupIdPublicKeyFamilyList;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UntrustedBackupPublicKeyFamilyList(pub Vec<PublishedBackupIdPublicKeyFamilyList>);
