use std::fmt::{self, Display, Formatter};

use crate::api::models::identity::Identity;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BackupIdentity(String);

/// This is a "fake" identity used for signing and verification of backup id keys.
/// Backup Id keys aren't really associated with an identity in the same way that journalist, covernode,
/// and sentinel id keys are, but the type system expects identity keys to be associated with an identity,
/// so we hard-code this one wherever an [`Identity`] is required for backup id keys. This is a hack to make the type system happy.
///
/// TODO It might be better to model "backup signing" and "backup encryption" keys independently of "identity"
/// and "messaging" roles as they don't follow the same patterns as the other id and messaging keys in terms of
/// their validity duration, rotation period etc. Both keys could be signed directly by the organization key.
impl BackupIdentity {
    pub fn new(s: &str) -> Self {
        BackupIdentity(s.to_string())
    }
}

impl Identity for BackupIdentity {}

impl AsRef<String> for BackupIdentity {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

impl Display for BackupIdentity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
