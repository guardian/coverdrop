use rand::Rng;

/// Generate a random token for use in the K3S cluster
pub fn generate_random_token() -> String {
    let mut rng = rand::thread_rng();

    let token: String = (0..32)
        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
        .collect();

    token
}
