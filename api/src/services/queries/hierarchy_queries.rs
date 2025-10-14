use std::collections::{hash_map::Entry, HashMap};

use chrono::{DateTime, Utc};
use common::{
    api::models::{covernode_id::CoverNodeIdentity, journalist_id::JournalistIdentity},
    backup::keys::{
        verify_backup_id_pk, verify_backup_msg_pk, BackupIdPublicKey, UntrustedBackupIdPublicKey,
    },
    protocol::keys::{
        verify_covernode_id_pk, verify_covernode_messaging_pk, verify_covernode_provisioning_pk,
        verify_journalist_id_pk, verify_journalist_messaging_pk, verify_journalist_provisioning_pk,
        verify_organization_pk, AnchorOrganizationPublicKey, BackupIdPublicKeyFamily,
        BackupIdPublicKeyFamilyList, BackupMessagingPublicKey, CoverDropPublicKeyHierarchy,
        CoverNodeIdPublicKey, CoverNodeIdPublicKeyFamily, CoverNodeIdPublicKeyFamilyList,
        CoverNodeMessagingPublicKey, CoverNodeProvisioningPublicKey,
        CoverNodeProvisioningPublicKeyFamily, CoverNodeProvisioningPublicKeyFamilyList,
        JournalistIdPublicKey, JournalistIdPublicKeyFamily, JournalistIdPublicKeyFamilyList,
        JournalistMessagingPublicKey, JournalistProvisioningPublicKey,
        JournalistProvisioningPublicKeyFamily, JournalistProvisioningPublicKeyFamilyList,
        OrganizationPublicKey, OrganizationPublicKeyFamily, OrganizationPublicKeyFamilyList,
        UntrustedBackupMessagingPublicKey, UntrustedCoverNodeIdPublicKey,
        UntrustedCoverNodeMessagingPublicKey, UntrustedCoverNodeProvisioningPublicKey,
        UntrustedJournalistIdPublicKey, UntrustedJournalistMessagingPublicKey,
        UntrustedJournalistProvisioningPublicKey, UntrustedOrganizationPublicKey,
    },
};
use serde_json::Value;
use sqlx::PgPool;

#[derive(Clone)]
pub struct HierarchyQueries {
    pool: PgPool,
}

impl HierarchyQueries {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn key_hierarchy(
        &self,
        anchor_org_pks: &[AnchorOrganizationPublicKey],
        now: DateTime<Utc>,
    ) -> anyhow::Result<(CoverDropPublicKeyHierarchy, i32)> {
        let mut conn = self.pool.acquire().await?;

        let hierarchy_rows = sqlx::query!(
            r#"
            SELECT
                org_pk_id AS "org_pk_id!: i32",
                org_pk_json AS "org_pk_json!: Value",
                covernode_provisioning_pk_id AS "covernode_provisioning_pk_id?: i32",
                covernode_provisioning_pk_json AS "covernode_provisioning_pk_json?: Value",
                covernode_id AS "covernode_id?: CoverNodeIdentity",
                covernode_id_pk_id AS "covernode_id_pk_id?: i32",
                covernode_id_pk_json AS "covernode_id_pk_json?: Value",
                covernode_msg_pk_id AS "covernode_msg_pk_id?: i32",
                covernode_msg_pk_json AS "covernode_msg_pk_json?: Value",
                journalist_provisioning_pk_id AS "journalist_provisioning_pk_id?: i32",
                journalist_provisioning_pk_json AS "journalist_provisioning_pk_json?: Value",
                journalist_id AS "journalist_id?: JournalistIdentity",
                journalist_id_pk_id AS "journalist_id_pk_id?: i32",
                journalist_id_pk_json AS "journalist_id_pk_json?: Value",
                journalist_msg_pk_id AS "journalist_msg_pk_id?: i32",
                journalist_msg_pk_json AS "journalist_msg_pk_json?: Value",
                backup_id_pk_id AS "backup_id_pk_id?: i32",
                backup_id_pk_json AS "backup_id_pk_json?: Value",
                backup_msg_pk_id AS "backup_msg_pk_id?: i32",
                backup_msg_pk_json AS "backup_msg_pk_json?: Value",
                -- This may seem like a convoluted way to do it, but we want to make sure that the keys returned match the epoch.
                -- If we made a second query, new keys could have been added and the epoch value changed.
                -- As this query is a candidate for refactoring, I will leave that optimization for a future TODO.
                (
                     SELECT MAX(epoch)
                        FROM (
                            SELECT MAX(epoch) AS epoch FROM organization_pks
                            UNION
                            SELECT MAX(epoch) AS epoch FROM covernode_provisioning_pks
                            UNION
                            SELECT MAX(epoch) AS epoch FROM covernode_id_pks
                            UNION
                            SELECT MAX(epoch) AS epoch FROM covernode_msg_pks
                            UNION
                            SELECT MAX(epoch) AS epoch FROM journalist_provisioning_pks
                            UNION
                            SELECT MAX(epoch) AS epoch FROM journalist_id_pks
                            UNION
                            SELECT MAX(epoch) AS epoch FROM journalist_msg_pks
                        ) max
                ) AS "max_epoch: i32"
            FROM
                -- We do a union here so that is easier to reason with the query for each side of the hierarchy
                (
                    -- This selects the covernode keys portion of the hierarchy
                    (
                        SELECT
                            organization_pks.id AS org_pk_id,
                            organization_pks.pk_json AS org_pk_json,
                            covernode_provisioning_pks.id AS covernode_provisioning_pk_id,
                            covernode_provisioning_pks.pk_json AS covernode_provisioning_pk_json,
                            covernode_id_pks.covernode_id AS covernode_id,
                            covernode_id_pks.id AS covernode_id_pk_id,
                            covernode_id_pks.pk_json AS covernode_id_pk_json,
                            covernode_msg_pks.id AS covernode_msg_pk_id,
                            covernode_msg_pks.pk_json AS covernode_msg_pk_json,
                            NULL AS journalist_provisioning_pk_id,
                            NULL AS journalist_provisioning_pk_json,
                            NULL AS journalist_id,
                            NULL AS journalist_id_pk_id,
                            NULL AS journalist_id_pk_json,
                            NULL AS journalist_msg_pk_id,
                            NULL AS journalist_msg_pk_json,
                            NULL AS backup_id_pk_id,
                            NULL AS backup_id_pk_json,
                            NULL AS backup_msg_pk_id,
                            NULL AS backup_msg_pk_json
                        FROM
                            organization_pks
                            LEFT JOIN covernode_provisioning_pks ON (
                                covernode_provisioning_pks.org_pk_id = organization_pks.id
                                AND covernode_provisioning_pks.not_valid_after > $1
                            )
                            LEFT JOIN covernode_id_pks ON (
                                covernode_id_pks.provisioning_pk_id = covernode_provisioning_pks.id
                                AND covernode_id_pks.not_valid_after > $1
                            )
                            LEFT JOIN covernode_msg_pks ON (
                                covernode_msg_pks.id_pk_id = covernode_id_pks.id
                                AND covernode_msg_pks.not_valid_after > $1
                            )
                        WHERE (organization_pks.pk_json->>'not_valid_after')::TIMESTAMPTZ > $1
                    )
                    UNION
                    -- This selects the journalist keys portion of the hierarchy
                    (
                        SELECT
                            organization_pks.id AS org_pk_id,
                            organization_pks.pk_json AS org_pk_json,
                            NULL AS covernode_provisioning_pk_id,
                            NULL AS covernode_provisioning_pk_json,
                            NULL AS covernode_id,
                            NULL AS covernode_id_pk_id,
                            NULL AS covernode_id_pk_json,
                            NULL AS covernode_msg_pk_id,
                            NULL AS covernode_msg_pk_json,
                            journalist_provisioning_pks.id AS journalist_provisioning_pk_id,
                            journalist_provisioning_pks.pk_json AS journalist_provisioning_pk_json,
                            journalist_id_pks.journalist_profile_id AS journalist_id,
                            journalist_id_pks.id AS journalist_id_pk_id,
                            journalist_id_pks.pk_json AS journalist_id_pk_json,
                            journalist_msg_pks.id AS journalist_msg_pk_id,
                            journalist_msg_pks.pk_json AS journalist_msg_pk_json,
                            backup_id_pks.id AS backup_id_pk_id,
                            backup_id_pks.pk_json AS backup_id_pk_json,
                            backup_msg_pks.id AS backup_msg_pk_id,
                            backup_msg_pks.pk_json AS backup_msg_pk_json
                        FROM
                            organization_pks
                            LEFT JOIN journalist_provisioning_pks ON (
                                journalist_provisioning_pks.org_pk_id = organization_pks.id
                                AND journalist_provisioning_pks.not_valid_after > $1
                            )
                            LEFT JOIN journalist_id_pks ON (
                                journalist_id_pks.provisioning_pk_id = journalist_provisioning_pks.id
                                AND journalist_id_pks.not_valid_after > $1
                            )
                            LEFT JOIN journalist_msg_pks ON (
                                journalist_msg_pks.id_pk_id = journalist_id_pks.id
                                AND journalist_msg_pks.not_valid_after > $1
                            )
                            LEFT JOIN backup_id_pks ON (
                                backup_id_pks.org_pk_id = organization_pks.id
                                AND backup_id_pks.not_valid_after > $1
                            )
                            LEFT JOIN backup_msg_pks ON (
                                backup_msg_pks.backup_id_pk_id = backup_id_pks.id
                                AND backup_msg_pks.not_valid_after > $1
                            )
                        WHERE (organization_pks.pk_json->>'not_valid_after')::TIMESTAMPTZ > $1
                    )
                ) AS keys
            "#, now
        )
        .fetch_all(&mut *conn)
        .await?;

        // A hashmap of the key ID to a tuple of the parent key ID and the key itself
        type ChildKeyMap<T> = HashMap<i32, (i32, T)>;

        // Lots of allocations here but it makes it easy for us to know if we need to re-verify a key

        // The org PK has no parent so it doesn't use the `ChildKeyMap`
        let mut org_pks: HashMap<i32, OrganizationPublicKey> = HashMap::new();

        let mut covernode_provisioning_pks: ChildKeyMap<CoverNodeProvisioningPublicKey> =
            HashMap::new();
        let mut covernode_id_pks: ChildKeyMap<CoverNodeIdPublicKey> = HashMap::new();
        let mut covernode_msg_pks: ChildKeyMap<CoverNodeMessagingPublicKey> = HashMap::new();
        let mut covernode_ids: ChildKeyMap<CoverNodeIdentity> = HashMap::new();

        let mut journalist_provisioning_pks: ChildKeyMap<JournalistProvisioningPublicKey> =
            HashMap::new();
        let mut journalist_id_pks: ChildKeyMap<JournalistIdPublicKey> = HashMap::new();
        let mut journalist_msg_pks: ChildKeyMap<JournalistMessagingPublicKey> = HashMap::new();
        let mut journalist_ids: ChildKeyMap<JournalistIdentity> = HashMap::new();

        let mut backup_id_pks: ChildKeyMap<BackupIdPublicKey> = HashMap::new();
        let mut backup_msg_pks: ChildKeyMap<BackupMessagingPublicKey> = HashMap::new();

        let mut epoch = 0;

        // For each row, for each column, feed each key to the maps
        // Because we can't guarantee that a full tree will always be present
        // we have to use LEFT JOIN. This means every node in our hierarchy rows
        // except the organization node will be an Option<T>.
        for row in hierarchy_rows {
            if let Some(record) = row.max_epoch {
                if record > epoch {
                    epoch = record
                }
            }

            if let Entry::Vacant(e) = org_pks.entry(row.org_pk_id) {
                let org_pk =
                    serde_json::from_value::<UntrustedOrganizationPublicKey>(row.org_pk_json)?;

                // If the key in the database is not in the list of trusted keys then we shouldn't serve
                // it to users. Note: we need to case the database key to the trusted organization role
                // in order to perform the comparison
                let Some(org_pk) = anchor_org_pks.iter().find_map(|anchor_org_pk| {
                    verify_organization_pk(&org_pk, anchor_org_pk, now).ok()
                }) else {
                    continue;
                };

                e.insert(org_pk);
            }

            if let Some(covernode_provisioning_pk_id) = row.covernode_provisioning_pk_id {
                if let Some(covernode_provisioning_pk_json) = row.covernode_provisioning_pk_json {
                    if let Entry::Vacant(e) =
                        covernode_provisioning_pks.entry(covernode_provisioning_pk_id)
                    {
                        let covernode_provisioning_pk =
                            serde_json::from_value::<UntrustedCoverNodeProvisioningPublicKey>(
                                covernode_provisioning_pk_json,
                            )?;

                        if let Some(verifying_key) = org_pks.get(&row.org_pk_id) {
                            if let Ok(covernode_provisioning_pk) = verify_covernode_provisioning_pk(
                                &covernode_provisioning_pk,
                                verifying_key,
                                now,
                            ) {
                                e.insert((row.org_pk_id, covernode_provisioning_pk));
                            }
                        }
                    }

                    if let Some(covernode_id_pk_id) = row.covernode_id_pk_id {
                        if let Some(covernode_id) = row.covernode_id {
                            if let Entry::Vacant(e) = covernode_ids.entry(covernode_id_pk_id) {
                                e.insert((covernode_id_pk_id, covernode_id));
                            }
                        }

                        if let Some(covernode_id_pk_json) = row.covernode_id_pk_json {
                            if let Entry::Vacant(e) = covernode_id_pks.entry(covernode_id_pk_id) {
                                let covernode_id_pk =
                                    serde_json::from_value::<UntrustedCoverNodeIdPublicKey>(
                                        covernode_id_pk_json,
                                    )?;

                                if let Some((_, verifying_key)) =
                                    &covernode_provisioning_pks.get(&covernode_provisioning_pk_id)
                                {
                                    if let Ok(covernode_id_pk) =
                                        verify_covernode_id_pk(&covernode_id_pk, verifying_key, now)
                                    {
                                        e.insert((covernode_provisioning_pk_id, covernode_id_pk));
                                    }
                                }
                            }
                        }

                        if let Some(covernode_msg_pk_id) = row.covernode_msg_pk_id {
                            if let Some(covernode_msg_pk_json) = row.covernode_msg_pk_json {
                                if let Entry::Vacant(e) =
                                    covernode_msg_pks.entry(covernode_msg_pk_id)
                                {
                                    let covernode_msg_pk = serde_json::from_value::<
                                        UntrustedCoverNodeMessagingPublicKey,
                                    >(
                                        covernode_msg_pk_json
                                    )?;

                                    if let Some((_, verifying_key)) =
                                        &covernode_id_pks.get(&covernode_id_pk_id)
                                    {
                                        if let Ok(covernode_msg_pk) = verify_covernode_messaging_pk(
                                            &covernode_msg_pk,
                                            verifying_key,
                                            now,
                                        ) {
                                            e.insert((covernode_id_pk_id, covernode_msg_pk));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(journalist_provisioning_pk_id) = row.journalist_provisioning_pk_id {
                if let Some(journalist_provisioning_pk_json) = row.journalist_provisioning_pk_json {
                    if let Entry::Vacant(e) =
                        journalist_provisioning_pks.entry(journalist_provisioning_pk_id)
                    {
                        let journalist_provisioning_pk =
                            serde_json::from_value::<UntrustedJournalistProvisioningPublicKey>(
                                journalist_provisioning_pk_json,
                            )?;

                        if let Some(verifying_key) = org_pks.get(&row.org_pk_id) {
                            if let Ok(journalist_provisioning_pk) =
                                verify_journalist_provisioning_pk(
                                    &journalist_provisioning_pk,
                                    verifying_key,
                                    now,
                                )
                            {
                                e.insert((row.org_pk_id, journalist_provisioning_pk));
                            }
                        }
                    }
                }

                if let Some(journalist_id_pk_id) = row.journalist_id_pk_id {
                    if let Some(journalist_id) = row.journalist_id {
                        if let Entry::Vacant(e) = journalist_ids.entry(journalist_id_pk_id) {
                            e.insert((journalist_id_pk_id, journalist_id));
                        }
                    }

                    if let Some(journalist_id_pk_json) = row.journalist_id_pk_json {
                        if let Entry::Vacant(e) = journalist_id_pks.entry(journalist_id_pk_id) {
                            let journalist_id_pk =
                                serde_json::from_value::<UntrustedJournalistIdPublicKey>(
                                    journalist_id_pk_json,
                                )?;

                            if let Some((_, verifying_key)) =
                                &journalist_provisioning_pks.get(&journalist_provisioning_pk_id)
                            {
                                if let Ok(journalist_id_pk) =
                                    verify_journalist_id_pk(&journalist_id_pk, verifying_key, now)
                                {
                                    e.insert((journalist_provisioning_pk_id, journalist_id_pk));
                                }
                            }
                        }
                    }
                    if let Some(journalist_msg_pk_id) = row.journalist_msg_pk_id {
                        if let Some(journalist_msg_pk_json) = row.journalist_msg_pk_json {
                            if let Entry::Vacant(e) = journalist_msg_pks.entry(journalist_msg_pk_id)
                            {
                                let journalist_msg_pk = serde_json::from_value::<
                                    UntrustedJournalistMessagingPublicKey,
                                >(
                                    journalist_msg_pk_json
                                )?;

                                if let Some((_, verifying_key)) =
                                    &journalist_id_pks.get(&journalist_id_pk_id)
                                {
                                    if let Ok(journalist_msg_pk) = verify_journalist_messaging_pk(
                                        &journalist_msg_pk,
                                        verifying_key,
                                        now,
                                    ) {
                                        e.insert((journalist_id_pk_id, journalist_msg_pk));
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(backup_id_pk_id) = row.backup_id_pk_id {
                if let Some(backup_id_pk_json) = row.backup_id_pk_json {
                    if let Entry::Vacant(e) = backup_id_pks.entry(backup_id_pk_id) {
                        let backup_id_pk = serde_json::from_value::<UntrustedBackupIdPublicKey>(
                            backup_id_pk_json,
                        )?;

                        if let Some(verifying_key) = org_pks.get(&row.org_pk_id) {
                            if let Ok(backup_id_pk) =
                                verify_backup_id_pk(&backup_id_pk, verifying_key, now)
                            {
                                e.insert((row.org_pk_id, backup_id_pk));
                            }
                        }
                    }

                    if let Some(backup_msg_pk_id) = row.backup_msg_pk_id {
                        if let Some(backup_msg_pk_json) = row.backup_msg_pk_json {
                            if let Entry::Vacant(e) = backup_msg_pks.entry(backup_msg_pk_id) {
                                let backup_msg_pk =
                                    serde_json::from_value::<UntrustedBackupMessagingPublicKey>(
                                        backup_msg_pk_json,
                                    )?;

                                if let Some((_, verifying_key)) =
                                    &backup_id_pks.get(&backup_id_pk_id)
                                {
                                    if let Ok(backup_msg_pk) =
                                        verify_backup_msg_pk(&backup_msg_pk, verifying_key, now)
                                    {
                                        e.insert((backup_id_pk_id, backup_msg_pk));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Now we've got our keys and their IDs mapped out we can convert them to a tree

        let mut org_hierarchy = OrganizationPublicKeyFamilyList::empty();

        for (org_db_id, org_pk) in org_pks {
            let mut covernode_keys = CoverNodeProvisioningPublicKeyFamilyList::empty();
            let mut journalist_provisioning_pk_family_list =
                JournalistProvisioningPublicKeyFamilyList::empty();
            let mut backup_id_pk_family_list = BackupIdPublicKeyFamilyList::empty();

            // Construct CoverNode key hierarchy
            for (covernode_provisioning_db_id, (parent_org_db_id, covernode_provisioning_pk)) in
                &covernode_provisioning_pks
            {
                if org_db_id == *parent_org_db_id {
                    let mut covernode_id_keys =
                        HashMap::<CoverNodeIdentity, CoverNodeIdPublicKeyFamilyList>::new();

                    for (
                        covernode_id_db_id,
                        (parent_covernode_provisioning_db_id, covernode_id_pk),
                    ) in &covernode_id_pks
                    {
                        if covernode_provisioning_db_id == parent_covernode_provisioning_db_id {
                            let mut covernode_msg_keys: Vec<CoverNodeMessagingPublicKey> = vec![];

                            if let Some(covernode_id) =
                                covernode_ids.get(covernode_id_db_id).map(|(_, id)| id)
                            {
                                // The messaging key ID is not used for building the hierarchy, it is solely for deduplication
                                // So for rebuilding the tree we just need the .values()
                                for (parent_covernode_id_db_id, covernode_msg_pk) in
                                    covernode_msg_pks.values()
                                {
                                    if covernode_id_db_id == parent_covernode_id_db_id {
                                        covernode_msg_keys.push(covernode_msg_pk.clone());
                                    }
                                }

                                let entry = CoverNodeIdPublicKeyFamily::new(
                                    covernode_id_pk.clone(),
                                    covernode_msg_keys,
                                );
                                match covernode_id_keys.get_mut(covernode_id) {
                                    Some(id_pk_family) => id_pk_family.insert(entry),
                                    None => {
                                        let list = CoverNodeIdPublicKeyFamilyList::new(vec![entry]);
                                        let covernode_id = covernode_id.clone();
                                        covernode_id_keys.insert(covernode_id, list);
                                    }
                                }
                            }
                        }
                    }

                    let covernode_provisioning_pk_family =
                        CoverNodeProvisioningPublicKeyFamily::new(
                            covernode_provisioning_pk.clone(),
                            covernode_id_keys,
                        );
                    covernode_keys.insert(covernode_provisioning_pk_family);
                }
            }

            // Construct journalist key hierarchy

            for (journalist_provisioning_db_id, (parent_org_db_id, journalist_provisioning_pk)) in
                &journalist_provisioning_pks
            {
                if org_db_id == *parent_org_db_id {
                    let mut journalist_id_keys =
                        HashMap::<JournalistIdentity, JournalistIdPublicKeyFamilyList>::new();

                    for (
                        journalist_id_db_id,
                        (parent_journalist_provisioning_db_id, journalist_id_pk),
                    ) in &journalist_id_pks
                    {
                        if journalist_provisioning_db_id == parent_journalist_provisioning_db_id {
                            let mut journalist_msg_keys: Vec<JournalistMessagingPublicKey> = vec![];

                            if let Some(journalist_id) =
                                journalist_ids.get(journalist_id_db_id).map(|(_, id)| id)
                            {
                                // The messaging key ID is not used for building the hierarchy, it is solely for deduplication
                                // So for rebuilding the tree we just need the .values()
                                for (parent_journalist_id_db_id, journalist_msg_pk) in
                                    journalist_msg_pks.values()
                                {
                                    if journalist_id_db_id == parent_journalist_id_db_id {
                                        journalist_msg_keys.push(journalist_msg_pk.clone());
                                    }
                                }

                                let entry = JournalistIdPublicKeyFamily::new(
                                    journalist_id_pk.clone(),
                                    journalist_msg_keys,
                                );

                                match journalist_id_keys.get_mut(journalist_id) {
                                    Some(id_pk_family) => id_pk_family.insert(entry),
                                    None => {
                                        let list =
                                            JournalistIdPublicKeyFamilyList::new(vec![entry]);
                                        let journalist_id = journalist_id.clone();
                                        journalist_id_keys.insert(journalist_id, list);
                                    }
                                }
                            }
                        }
                    }

                    let journalist_provisioning_pk_family =
                        JournalistProvisioningPublicKeyFamily::new(
                            journalist_provisioning_pk.clone(),
                            journalist_id_keys,
                        );
                    journalist_provisioning_pk_family_list
                        .insert(journalist_provisioning_pk_family);
                }
            }

            // Construct backup key hierarchy

            for (backup_id_pk_db_id, (org_pk_id, backup_id_pk)) in &backup_id_pks {
                // We gotta get the db id of the backup id pk, and for each of those, we need a new family entry
                if org_db_id == *org_pk_id {
                    let mut backup_msg_keys: Vec<BackupMessagingPublicKey> = vec![];

                    // The messaging key ID is not used for building the hierarchy, it is solely for deduplication
                    // So for rebuilding the tree we just need the .values()
                    for (parent_backup_id_db_id, backup_msg_pk) in backup_msg_pks.values() {
                        if backup_id_pk_db_id == parent_backup_id_db_id {
                            backup_msg_keys.push(backup_msg_pk.clone());
                        }
                    }
                    let entry = BackupIdPublicKeyFamily::new(backup_id_pk.clone(), backup_msg_keys);

                    backup_id_pk_family_list.insert(entry);
                }
            }

            let org_pk_family = OrganizationPublicKeyFamily::new(
                org_pk,
                covernode_keys,
                journalist_provisioning_pk_family_list,
                Some(backup_id_pk_family_list),
            );

            org_hierarchy.insert(org_pk_family);
        }

        Ok((org_hierarchy, epoch))
    }
}
