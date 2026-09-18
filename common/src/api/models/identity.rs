use std::fmt::Display;

/// A unique string-based identity for an entity in the CoverDrop system (e.g. journalist, covernode, sentinel).
pub trait Identity: AsRef<String> + Display {}
