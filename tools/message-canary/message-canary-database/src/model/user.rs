use common::protocol::keys::UserKeyPair;

#[derive(Clone)]
pub struct User {
    pub user_id: i32,
    pub key_pair: UserKeyPair,
}
