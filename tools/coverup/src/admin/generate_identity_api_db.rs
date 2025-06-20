use std::path::Path;

use common::{
    protocol::keys::{
        load_anchor_org_pks, load_covernode_provisioning_key_pairs_with_parent,
        load_journalist_provisioning_key_pairs_with_parent_org_pks,
    },
    time,
};

pub async fn generate_identity_api_db(
    db_path: impl AsRef<Path>,
    password: &str,
    keys_path: impl AsRef<Path>,
    interactive: bool,
) -> anyhow::Result<()> {
    let now = time::now();

    let keys_path = keys_path.as_ref();

    let anchor_org_pks = load_anchor_org_pks(keys_path, now)?;

    if anchor_org_pks.is_empty() {
        anyhow::bail!("No anchor public keys found in {}", keys_path.display());
    }

    let covernode_provisioning_key_pairs =
        load_covernode_provisioning_key_pairs_with_parent(keys_path, &anchor_org_pks, now)?;

    if covernode_provisioning_key_pairs.is_empty() {
        anyhow::bail!(
            "No CoverNode provisioning keys found in {}",
            keys_path.display()
        );
    }

    let journalist_provisioning_key_pairs =
        load_journalist_provisioning_key_pairs_with_parent_org_pks(
            keys_path,
            &anchor_org_pks,
            now,
        )?;

    if journalist_provisioning_key_pairs.is_empty() {
        anyhow::bail!(
            "No journalist provisioning keys found in {}",
            keys_path.display()
        );
    }

    if interactive {
        println!(
            "Found {} anchor organization public keys",
            anchor_org_pks.len()
        );
        println!(
            "Found {} journalist provisioining key pair",
            journalist_provisioning_key_pairs.len()
        );
        println!(
            "Found {} covernode provisioining keys pairs",
            covernode_provisioning_key_pairs.len()
        );
        println!();
        println!("Does this sound right? [yN]");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        let input = input.trim().to_lowercase();

        if !input.is_empty() && input != "y" && input != "yes" {
            anyhow::bail!("Aborted by user");
        }
    }

    let db = identity_api_database::Database::open(db_path, password).await?;

    for anchor_org_pk in &anchor_org_pks {
        db.insert_anchor_organization_pk(anchor_org_pk, now).await?;
    }

    for (org_pk, journalist_provisioning_key_pair) in journalist_provisioning_key_pairs {
        db.insert_journalist_provisioning_key_pair(&org_pk, &journalist_provisioning_key_pair, now)
            .await?;
    }

    for (covernode_provisioning_key_pair, org_pk) in covernode_provisioning_key_pairs {
        db.insert_covernode_provisioning_key_pair(org_pk, &covernode_provisioning_key_pair, now)
            .await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use common::{
        crypto::keys::serde::StorableKeyMaterial,
        protocol::keys::{
            generate_covernode_provisioning_key_pair, generate_journalist_provisioning_key_pair,
            generate_organization_key_pair,
        },
        time,
    };
    use tempfile::TempDir;

    use super::generate_identity_api_db;

    #[tokio::test]
    async fn create_identity_api_db() {
        let tmp_dir = TempDir::new().expect("Create temp dir");

        let now = time::now();

        let anchor_org_pk = generate_organization_key_pair(now);

        let journalist_provisioning_key_pair =
            generate_journalist_provisioning_key_pair(&anchor_org_pk, now);

        let covernode_provisioning_key_pair =
            generate_covernode_provisioning_key_pair(&anchor_org_pk, now);

        anchor_org_pk
            .public_key()
            .to_untrusted()
            .save_to_disk(tmp_dir.path())
            .expect("save org pk");

        journalist_provisioning_key_pair
            .to_untrusted()
            .save_to_disk(tmp_dir.path())
            .expect("save journalist provisioning key pair");

        covernode_provisioning_key_pair
            .to_untrusted()
            .save_to_disk(tmp_dir.path())
            .expect("save covernode provisioning key pair");

        let db_path = tmp_dir.path().join("identity-api.db");

        generate_identity_api_db(&db_path, "testpassword", tmp_dir.path(), false)
            .await
            .expect("Create admin db");

        let db = identity_api_database::Database::open(&db_path, "testpassword")
            .await
            .expect("Open db");

        let now = time::now();

        let db_anchor_org_pks = db
            .select_anchor_organization_pks(now)
            .await
            .expect("Select org pks");

        assert!(db_anchor_org_pks
            .iter()
            .any(|db_org_pk| db_org_pk.key == anchor_org_pk.public_key().key));

        let db_journalist_provisioning_key_pairs = db
            .select_journalist_provisioning_key_pairs(now)
            .await
            .expect("select journalist provisioning key pairs");

        assert!(db_journalist_provisioning_key_pairs.iter().any(
            |db_journalist_provisioning_key_pair| db_journalist_provisioning_key_pair.secret_key
                == journalist_provisioning_key_pair.secret_key
        ));

        let db_covernode_provisioning_key_pairs = db
            .select_covernode_provisioning_key_pairs(now)
            .await
            .expect("select covernode provisioning key pairs");

        assert!(db_covernode_provisioning_key_pairs.iter().any(
            |db_covernode_provisioning_key_pair| db_covernode_provisioning_key_pair.secret_key
                == covernode_provisioning_key_pair.secret_key
        ));
    }
}
