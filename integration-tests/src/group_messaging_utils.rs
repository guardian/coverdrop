use common::api::models::sentinel_id::SentinelIdentity;
use common::protocol::keys::OrganizationPublicKeyFamilyList;
use coverdrop_service::JournalistCoverDropService;
use group_messaging_service::GroupMessagingService;
use journalist_vault::GroupId;
use journalist_vault::GroupMessage;
use journalist_vault::JournalistVault;

use crate::api_wrappers::generate_test_journalist;
use crate::secrets::MAILBOX_PASSWORD;
use crate::stack::CoverDropStack;

pub async fn create_client(
    stack: &CoverDropStack,
    journalist_id_str: &str,
) -> (
    GroupMessagingService,
    JournalistVault,
    JournalistCoverDropService,
) {
    let sentinel_id_str = format!("{}_sentinel", journalist_id_str);

    generate_test_journalist(
        stack.api_client_cached(),
        stack.keys_path(),
        stack.temp_dir_path(),
        stack.now(),
        Some(journalist_id_str.to_string()),
        Some(sentinel_id_str),
    )
    .await;

    let vault_path = stack
        .temp_dir_path()
        .join(format!("{}.vault", journalist_id_str));

    let vault = JournalistVault::open(
        &vault_path,
        MAILBOX_PASSWORD,
        common::clap::Stage::Development,
    )
    .await
    .expect("Load journalist vault");

    let group_messaging_service = GroupMessagingService::new(
        stack.delivery_service_url().await.clone(),
        &vault,
        stack.now(),
    )
    .await
    .expect("Create group messaging service");

    let journalist_coverdrop_service =
        JournalistCoverDropService::new(stack.api_client_cached(), &vault);

    (group_messaging_service, vault, journalist_coverdrop_service)
}

/// Receive messages for a client, assert the number received and total stored,
/// mark all as read, and return previously unread messages.
pub async fn receive_and_return_unread(
    client: &GroupMessagingService,
    group_id: &GroupId,
    public_keys: &OrganizationPublicKeyFamilyList,
    expected_received: usize,
    expected_total_stored: usize,
) -> Vec<GroupMessage> {
    let received = client
        .receive_and_store_messages(public_keys)
        .await
        .expect("Failed to receive messages");
    assert_eq!(
        received.len(),
        expected_received,
        "Expected {} to receive {} messages, got {}",
        client.client_id().await,
        expected_received,
        received.len()
    );

    let groups = client
        .get_groups_and_messages()
        .await
        .expect("Failed to get groups and messages");
    let group = groups
        .iter()
        .find(|g| g.id() == group_id)
        .expect("Group not found");
    let stored = group.messages();
    let num_stored = stored.len();
    assert_eq!(
        num_stored,
        expected_total_stored,
        "Expected {} to have {} stored messages, got {}",
        client.client_id().await,
        expected_total_stored,
        num_stored
    );

    let unread = stored
        .iter()
        .filter(|m| !m.read)
        .cloned()
        .collect::<Vec<_>>();
    let unread_ids = unread.iter().map(|m| m.id).collect::<Vec<_>>();

    if !unread_ids.is_empty() {
        client
            .update_group_messages_read_status(group_id, unread_ids, true)
            .await
            .expect("Failed to update message read status");
    }

    unread
}

/// Assert that a client's group has the expected display name, description, and members.
pub async fn assert_group_info(
    client: &GroupMessagingService,
    group_id: &GroupId,
    expected_name: &str,
    expected_description: &str,
    expected_members: &[&SentinelIdentity],
) {
    let groups = client
        .get_groups_and_messages()
        .await
        .expect("Failed to get groups and messages");
    let group = groups
        .iter()
        .find(|g| g.id() == group_id)
        .unwrap_or_else(|| panic!("Expected group with id '{}'", group_id));
    assert_eq!(group.display_name(), expected_name);
    assert_eq!(group.description(), expected_description);
    let members = group.members();
    assert_eq!(
        members.len(),
        expected_members.len(),
        "Expected {} members, got {}",
        expected_members.len(),
        members.len()
    );
    for expected in expected_members {
        assert!(
            members.contains(expected),
            "Expected member {:?} not found in group members",
            expected
        );
    }
}
