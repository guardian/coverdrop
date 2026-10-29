use chrono::{DateTime, Utc};
use common::{
    api::{
        api_client::ApiClient,
        models::dead_drops::{
            DeadDropId, JournalistToUserDeadDropSignatureDataV2,
            SerializedJournalistToUserDeadDropMessages, SerializedUserToJournalistDeadDropMessages,
            UnpublishedJournalistToUserDeadDrop, UnpublishedUserToJournalistDeadDrop,
            UserToJournalistDeadDropSignatureDataV2,
        },
    },
    epoch::Epoch,
    protocol::keys::CoverNodeIdKeyPair,
};
use integration_tests::stack::{CoverDropStack, StackProfile};

// Dead drops are stored with microsecond precision. A and B share a millisecond and C sits exactly on the
// following millisecond boundary.
const CREATED_AT_A: &str = "2026-10-28T13:31:00.123456Z";
const CREATED_AT_B: &str = "2026-10-28T13:31:00.123999Z";
const CREATED_AT_C: &str = "2026-10-28T13:31:00.124000Z";
const CREATED_AT_D: &str = "2026-10-28T13:31:05.000000Z";

const EVERYTHING: [&str; 4] = [CREATED_AT_A, CREATED_AT_B, CREATED_AT_C, CREATED_AT_D];

// The client uses the legacy `ids_greater_than` endpoint when the user to journalist cursor is exactly the epoch
// so we start one millisecond after it.
const AFTER_EPOCH: &str = "1970-01-01T00:00:00.001Z";

/// Posts dead drops with chosen `created_at` timestamps, signed with the stack's CoverNode identity key, and checks
/// which of them the API returns for cursors of different precision. This covers both the journalist to user and
/// user to journalist endpoints.
#[tokio::test]
async fn dead_drop_created_after_cursor_is_compared_at_millisecond_precision() {
    integration_tests::utils::init_logger();

    let stack = CoverDropStack::builder(StackProfile::CoverDropOnly)
        .build()
        .await;

    let api_client = stack.api_client_uncached();
    let covernode_id_key_pair = &stack.keys().covernode_id_key_pair;

    // Nothing has been published yet so the results below only contain what this test posts
    assert_created_after(api_client, AFTER_EPOCH, &[]).await;

    // Posted out of order to show that the order of the results comes from `created_at` rather than insertion
    for (id, created_at) in [
        (4, CREATED_AT_D),
        (2, CREATED_AT_B),
        (1, CREATED_AT_A),
        (3, CREATED_AT_C),
    ] {
        post_journalist_to_user_dead_drop(
            api_client,
            covernode_id_key_pair,
            id,
            created_at.parse().unwrap(),
        )
        .await;
        post_user_to_journalist_dead_drop(
            api_client,
            covernode_id_key_pair,
            id,
            created_at.parse().unwrap(),
        )
        .await;
    }

    // everything after the epoch, whatever the precision of a cursor that is before the first dead drop
    assert_created_after(api_client, AFTER_EPOCH, &EVERYTHING).await;
    assert_created_after(api_client, "2026-10-28T13:31:00Z", &EVERYTHING).await;
    assert_created_after(api_client, "2026-10-28T13:31:00.122999Z", &EVERYTHING).await;

    // A client holding A must not be sent A again. B shares A's millisecond so is also excluded.
    let after_a = [CREATED_AT_C, CREATED_AT_D];
    assert_created_after(api_client, CREATED_AT_A, &after_a).await;
    // A client that only retains milliseconds holds A as .123
    assert_created_after(api_client, "2026-10-28T13:31:00.123Z", &after_a).await;
    assert_created_after(api_client, "2026-10-28T13:31:00.123456789Z", &after_a).await;

    // A dead drop created exactly on a millisecond boundary is excluded by a cursor at that boundary
    assert_created_after(api_client, CREATED_AT_C, &[CREATED_AT_D]).await;

    // Nothing is returned once a client has the latest dead drop, whichever precision it holds it at
    assert_created_after(api_client, CREATED_AT_D, &[]).await;
    assert_created_after(api_client, "2026-10-28T13:31:05.000Z", &[]).await;
    assert_created_after(api_client, "2026-10-28T13:31:06Z", &[]).await;
}

async fn post_journalist_to_user_dead_drop(
    api_client: &ApiClient,
    covernode_id_key_pair: &CoverNodeIdKeyPair,
    id: u8,
    created_at: DateTime<Utc>,
) {
    // The data only needs to be distinct, the API does not look inside it
    let data = SerializedJournalistToUserDeadDropMessages::from_vec_unchecked(vec![id]);
    let signature_data = JournalistToUserDeadDropSignatureDataV2::new(&data, created_at);
    let signature = covernode_id_key_pair.sign(&signature_data);

    api_client
        .post_journalist_to_user_dead_drop(&UnpublishedJournalistToUserDeadDrop::new(
            data, created_at, signature,
        ))
        .await
        .expect("Post journalist to user dead drop");
}

async fn post_user_to_journalist_dead_drop(
    api_client: &ApiClient,
    covernode_id_key_pair: &CoverNodeIdKeyPair,
    id: u8,
    created_at: DateTime<Utc>,
) {
    let epoch = Epoch(1);
    let data = SerializedUserToJournalistDeadDropMessages::from_vec_unchecked(vec![id]);
    let signature_data = UserToJournalistDeadDropSignatureDataV2::new(&data, created_at, epoch);
    let signature = covernode_id_key_pair.sign(&signature_data);

    api_client
        .post_user_to_journalist_dead_drop(&UnpublishedUserToJournalistDeadDrop::new(
            data, signature, created_at, epoch,
        ))
        .await
        .expect("Post user to journalist dead drop");
}

/// Checks that both dead drop endpoints return exactly the `expected` dead drops, oldest first, for the `cursor`.
async fn assert_created_after(api_client: &ApiClient, cursor: &str, expected: &[&str]) {
    let created_after = cursor.parse().unwrap();
    let expected: Vec<DateTime<Utc>> = expected
        .iter()
        .map(|timestamp| timestamp.parse().unwrap())
        .collect();
    let journalist_to_user: Vec<DateTime<Utc>> = api_client
        .pull_journalist_to_user_dead_drops(created_after)
        .await
        .expect("Pull journalist to user dead drops")
        .dead_drops
        .iter()
        .map(|dead_drop| dead_drop.created_at)
        .collect();
    assert_eq!(
        journalist_to_user, expected,
        "journalist to user dead drops with cursor {cursor}"
    );

    let user_to_journalist: Vec<DateTime<Utc>> = api_client
        .pull_user_to_journalist_dead_drops(DeadDropId::default(), created_after, None)
        .await
        .expect("Pull user to journalist dead drops")
        .dead_drops
        .iter()
        .map(|dead_drop| dead_drop.created_at)
        .collect();
    assert_eq!(
        user_to_journalist, expected,
        "user to journalist dead drops with cursor {cursor}"
    );
}
