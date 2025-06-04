use crate::{
    crypto::keys::{role::Role, signed::SignedKey},
    Error,
};

/// Utility trait for pulling out the latest key from a list of signed key material
pub trait LatestKey<R: Role, PK: SignedKey<R>>: Sized {
    /// Get the key with the maximum expiry time.
    ///
    /// Returns `None` if the iterator is empty
    fn latest_key(&self) -> Option<&PK>;

    /// Consume `self` and get the key with the maximum expiry time.
    ///
    /// Returns `None` if the iterator is empty
    fn into_latest_key(self) -> Option<PK>;

    /// Get the key with the maximum expiry time.
    ///
    /// Returns `Err` if the iterator is empty, useful when having a latest key
    /// is required by the protocol as it allows the caller to simply use the `?`
    /// operator rather than calling `ok_or_else` or pattern matching on the `None`.
    fn latest_key_required(&self) -> anyhow::Result<&PK> {
        Ok(self
            .latest_key()
            .ok_or_else(|| Error::LatestKeyPairNotFound(R::display()))?)
    }

    /// Consume `self` and get the key with the maximum expiry time.
    ///
    /// Returns `Err` if the iterator is empty, useful when having a latest key
    /// is required by the protocol as it allows the caller to simply use the `?`
    /// operator rather than calling `ok_or_else` or pattern matching on the `None`.
    fn into_latest_key_required(self) -> anyhow::Result<PK> {
        Ok(self
            .into_latest_key()
            .ok_or_else(|| Error::LatestKeyPairNotFound(R::display()))?)
    }
}

impl<R, PK> LatestKey<R, PK> for Vec<PK>
where
    R: Role,
    PK: SignedKey<R>,
{
    fn latest_key(&self) -> Option<&PK> {
        self.iter().max_by_key(|k| k.not_valid_after())
    }

    fn into_latest_key(self) -> Option<PK> {
        self.into_iter().max_by_key(|k| k.not_valid_after())
    }
}

impl<R, PK> LatestKey<R, PK> for &[PK]
where
    R: Role,
    PK: SignedKey<R> + Clone,
{
    fn latest_key(&self) -> Option<&PK> {
        self.iter().max_by_key(|k| k.not_valid_after())
    }

    fn into_latest_key(self) -> Option<PK> {
        self.iter().max_by_key(|k| k.not_valid_after()).cloned()
    }
}
