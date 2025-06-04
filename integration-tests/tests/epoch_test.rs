use api::services::queries::organization_key_queries::OrganizationKeyQueries;
use chrono::{DateTime, Utc};
use common::{
    api::api_client::ApiClient,
    epoch::Epoch,
    protocol::keys::{anchor_org_pk, generate_organization_key_pair},
};

pub use futures_util::stream::FuturesUnordered;
use futures_util::StreamExt;
use integration_tests::{
    constants::{POSTGRES_DB, POSTGRES_PASSWORD, POSTGRES_PORT, POSTGRES_USER},
    CoverDropStack,
};
use sqlx::postgres::PgPoolOptions;
use tokio::task::JoinHandle;

// This test checks that our epoch generation for newly added public keys into the api
// adds the correct epoch id based on the time that the key was inserted into the database.
// We have achieved this by using an advisory lock within a trigger for any adds, updates or deletes into
// any of the key tables in the api database.
// To test this works correctly, we fire lots of requests at the database and make sure the commit_inserted_timestamp and
// epoch id ordering are always the same.
// To also prove this does not work in without the advisory lock, we also update the trigger without the lock, and show that the
// ordering is not correct.

#[tokio::test]
async fn epoch_test() {
    pretty_env_logger::try_init().unwrap();

    let stack = CoverDropStack::builder()
        .with_additional_journalists(1)
        .build()
        .await;

    let postgres_ip = "localhost";

    let postgres_port = stack
        .api_postgres()
        .get_host_port_ipv4(POSTGRES_PORT)
        .await
        .expect("Get postgres port");

    let db_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        POSTGRES_USER, POSTGRES_PASSWORD, postgres_ip, postgres_port, POSTGRES_DB
    );

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
        .expect("Connect to database");

    // Check that our new epoch field is added to the api response

    {
        let public_keys = stack
            .api_client_cached()
            .get_public_keys()
            .await
            .expect("Get public keys");

        tracing::debug!("Current epoch is: {:?}", public_keys.max_epoch);

        assert_eq!(public_keys.max_epoch, Epoch(10));
    }

    let queries = OrganizationKeyQueries::new(pool.clone());

    let mut tasks: FuturesUnordered<_> = (1..=5)
        .map(|_| execute_query_as_task(queries.clone(), stack.now()))
        .collect();

    let mut results: Vec<Result<(), anyhow::Error>> = Vec::new();

    while let Some(res) = tasks.next().await {
        match res {
            Ok(result) => {
                // We will fold all results into a single error if there are any errors
                if result.iter().any(|r| r.is_err()) {
                    results.push(Err(anyhow::anyhow!("Error in query processing")));
                } else {
                    results.push(Ok(()));
                }
            }
            Err(e) => results.push(Err(e.into())),
        }
    }

    // Process results
    for result in results {
        //We want to fail the test if any of the queries have failed
        assert!(result.is_ok())
    }

    let result =
        integration_test_queries::ordering_by_transaction_timestamp_and_epoch_should_be_the_same(
            &pool,
        )
        .await;

    let results_count = result.len();
    // This shows that the ordering of all keys by epoch id or transaction_commit_timestamp are always the same.
    assert_eq!(results_count, 0);

    //
    // Confirm that reinserting the same organization public key should not re-trigger a epoch increment
    //     NB: this should probably be a separate test since it's logically distinct to the rest of this test
    //     but the overhead of spinning up another stack is a bit excessive.
    //

    let max_epoch_before = get_max_epoch(stack.api_client_uncached()).await;

    let reused_org_pk = generate_organization_key_pair(stack.now())
        .to_public_key()
        .to_untrusted();

    let anchor_org_pk =
        anchor_org_pk(&reused_org_pk.to_tofu_anchor(), stack.now()).expect("Make org pk");

    for _ in 0..5 {
        queries
            .insert_org_pk(&anchor_org_pk, stack.now())
            .await
            .expect("Insert reused org pk");

        let max_epoch_after_first = get_max_epoch(stack.api_client_uncached()).await;
        assert_eq!(max_epoch_before.0, max_epoch_after_first.0 - 1);
    }
}

async fn get_max_epoch(api_client: &ApiClient) -> Epoch {
    api_client.get_public_keys().await.unwrap().max_epoch
}

fn execute_query_as_task(
    organization_key_queries: OrganizationKeyQueries,
    now: DateTime<Utc>,
) -> JoinHandle<Vec<anyhow::Result<bool>>> {
    tokio::spawn(async move {
        let mut results: Vec<anyhow::Result<bool>> = vec![];

        for count in 0..100 {
            println!("Running ! {}", count);
            let org_pk = generate_organization_key_pair(now)
                .to_public_key()
                .to_untrusted();

            let anchor_org_pk =
                anchor_org_pk(&org_pk.to_tofu_anchor(), now).expect("Verify org pk");

            let query_result = organization_key_queries
                .insert_org_pk(&anchor_org_pk, now)
                .await;

            results.push(query_result);
        }

        results
    })
}
