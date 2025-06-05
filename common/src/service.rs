/// In some instances it's useful to be able to differentiate a
/// structure based on what service it refers to. This empty trait
/// can be used as a marker to differentiate between
pub trait CoverDropService {}

pub struct CoverNode;
impl CoverDropService for CoverNode {}

pub struct Api;
impl CoverDropService for Api {}

pub struct IdentityApi;
impl CoverDropService for IdentityApi {}
