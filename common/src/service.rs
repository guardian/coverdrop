/// In some instances it's useful to be able to differentiate a
/// structure based on what service it refers to. This empty trait
/// can be used as a marker to differentiate between
pub trait TaskClientService {}

pub struct CoverNode;
impl TaskClientService for CoverNode {}

pub struct Api;
impl TaskClientService for Api {}

pub struct IdentityApi;
impl TaskClientService for IdentityApi {}
