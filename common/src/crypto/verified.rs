use std::ops::Deref;

/// Simple wrapper newtype for verified versions of structs.
/// Useful when a verified version of something is purely semantic and doesn't require
/// any extra fields or functions.
pub struct Verified<T>(pub T);

impl<T> Deref for Verified<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
