use aws_sdk_s3::Client;

pub async fn get_object_as_string(
    s3_client: Client,
    bucket: String,
    key: &str,
) -> anyhow::Result<String> {
    let output_get_response = s3_client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await?;

    // parse body of response into string
    let output_text_bytes = output_get_response.body.collect().await?;
    let bytes_vec = output_text_bytes.into_bytes().to_vec();

    let file_contents = std::str::from_utf8(bytes_vec.as_ref())
        .map_err(|e| anyhow::anyhow!("Failed to convert output to string: {:?}", e))?;

    Ok(file_contents.to_string())
}
