use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    path::PathBuf,
    sync::Mutex,
};

use crate::api_wrappers::{
    get_journalist_dead_drops, get_latest_status, get_public_keys, get_user_dead_drops,
};
use crate::CoverDropStack;
use lazy_static::lazy_static;

type TestName = String;

lazy_static! {
    // test_name -> counter
    static ref TEST_NAME_TO_VECTORS_COUNTERS: Mutex<HashMap<TestName, u32>> = Mutex::new(HashMap::new());
}

#[macro_export]
macro_rules! save_test_vector {
    ($state_name:expr, $value:expr) => {
        let file = file!();
        let file = file
            .split(std::path::MAIN_SEPARATOR)
            .last()
            .unwrap()
            .replace(".rs", "");

        $crate::vectors::save_test_vectors_for_stack(&file, $state_name, $value).await;
    };
}

pub async fn save_test_vectors_for_stack(
    file_name: &str,
    state_name: &str,
    value: &CoverDropStack,
) {
    // Using `env!` means that the path is resolved at compile time
    // so the test binary isn't portable (just run it from `cargo test`)
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("vectors");

    save_test_vectors(path.clone(), file_name, state_name, value).await;
}

pub async fn save_test_vectors(
    mut path: PathBuf,
    file_name: &str,
    state_name: &str,
    stack: &CoverDropStack,
) {
    path.push(file_name);
    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }

    let counter = {
        let mut counter_lock = TEST_NAME_TO_VECTORS_COUNTERS.lock().unwrap();

        let counter = counter_lock.entry(file_name.to_string()).or_insert(0);
        *counter += 1;
        *counter
    };

    let public_keys = get_public_keys(stack.api_client_cached()).await;
    write_vector_for_serializable(
        path.clone(),
        state_name,
        counter,
        "published_keys",
        &public_keys,
    );

    let user_dead_drops = get_user_dead_drops(stack.api_client_cached(), 0).await;
    write_vector_for_serializable(
        path.clone(),
        state_name,
        counter,
        "user_dead_drops",
        &user_dead_drops,
    );

    let journalist_dead_drops = get_journalist_dead_drops(stack.api_client_cached(), 0).await;
    write_vector_for_serializable(
        path.clone(),
        state_name,
        counter,
        "journalist_dead_drops",
        &journalist_dead_drops,
    );

    let timestamp = stack.now();
    write_vector_for_serializable(path.clone(), state_name, counter, "timestamp", &timestamp);

    let system_status = get_latest_status(stack.api_client_cached()).await;
    write_vector_for_serializable(
        path.clone(),
        state_name,
        counter,
        "system_status",
        &system_status,
    );
}

fn write_vector_for_serializable<S>(
    mut path: PathBuf,
    state_name: &str,
    counter: u32,
    entity_name: &str,
    value: &S,
) where
    S: serde::Serialize,
{
    path.push(entity_name);

    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }

    let vector_file_name = format!("{counter:03}_{state_name}");

    path.push(vector_file_name);
    path.set_extension("json");
    let mut file = File::create(&path).expect("Create test vector file");
    serde_json::to_writer_pretty(&mut file, value).expect("Write test vector");
}
