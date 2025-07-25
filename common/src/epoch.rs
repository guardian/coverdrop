use std::{
    fmt::{self, Display, Formatter},
    mem::size_of,
    ops::Deref,
};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(
    Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, sqlx::Type, PartialOrd, Ord, TS,
)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct Epoch(pub i32);

// Convenience function for reaching into the inner i32 value
// which means we don't need to do `foo.0` everywhere
impl Deref for Epoch {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Epoch {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

// We use epochs in signatures so here's a dumb compile time check to confirm
// that a newtype is always the same size as it's inner type.
//
// This truly is an experiment in paranoia.
const _: () = assert!(size_of::<i32>() == size_of::<Epoch>());
