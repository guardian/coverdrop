use std::path::PathBuf;

use common::api::{
    api_client::ApiClient,
    forms::{
        PostCoverNodeProvisioningPublicKeyForm, PostJournalistProvisioningPublicKeyForm,
        COVERNODE_PROVISIONING_KEY_FORM_FILENAME, JOURNALIST_PROVISIONING_KEY_FORM_FILENAME,
    },
};
use tokio::fs;

pub async fn post_journalist_provisioning_key_pair(
    form_path: PathBuf,
    api_client: ApiClient,
) -> anyhow::Result<()> {
    // TODO the form type should be in the type system, then we won't need to pass the file name to save_to_disk
    let json =
        fs::read_to_string(&form_path.join(JOURNALIST_PROVISIONING_KEY_FORM_FILENAME)).await?;
    let form: PostJournalistProvisioningPublicKeyForm = serde_json::from_str(&json)?;

    api_client.post_journalist_provisioning_pk(form).await?;

    println!("✅ Journalist provisioning public key form successfully posted to API");

    Ok(())
}

pub async fn post_covernode_provisioning_key_pair(
    form_path: PathBuf,
    api_client: ApiClient,
) -> anyhow::Result<()> {
    // TODO the form type should be in the type system, then we won't need to pass the file name to save_to_disk
    let json =
        fs::read_to_string(&form_path.join(COVERNODE_PROVISIONING_KEY_FORM_FILENAME)).await?;
    let form: PostCoverNodeProvisioningPublicKeyForm = serde_json::from_str(&json)?;

    api_client.post_covernode_provisioning_pk(form).await?;

    println!("✅ CoverNode provisioning public key form successfully posted to API");

    Ok(())
}
