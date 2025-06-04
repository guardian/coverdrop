use aws_config::default_provider::credentials::DefaultCredentialsChain;
use aws_config::BehaviorVersion;
use aws_sdk_ses::config::Region;
use aws_sdk_ses::{
    types::{Body, Content, Destination, Message},
    Client,
};

#[derive(Clone)]
pub struct SesClient {
    inner: Client,
    from_email_address: String,
}

#[derive(Debug)]
pub struct SendEmailConfig {
    pub to: String,
    pub subject: String,
    pub reply_to: String,
    pub body: String,
}

impl SesClient {
    async fn build_credentials(profile: Option<String>) -> DefaultCredentialsChain {
        let mut builder = DefaultCredentialsChain::builder();
        if let Some(profile) = profile {
            builder = builder.profile_name(&profile);
        }

        builder.build().await
    }

    pub async fn new(
        region: String,
        profile: Option<String>,
        from_email_address: String,
    ) -> SesClient {
        let region = Region::new(region);
        let credentials_provider = SesClient::build_credentials(profile).await;

        let config = aws_sdk_ses::Config::builder()
            .behavior_version_latest()
            .region(region)
            .credentials_provider(credentials_provider)
            .build();

        let inner = Client::from_conf(config);

        SesClient {
            inner,
            from_email_address,
        }
    }

    /// When run in AWS, configuration can be loaded from the environment
    /// making parameters like `region` and `profile` redundant
    pub async fn new_in_aws(from_email_address: String) -> SesClient {
        let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        let config = aws_sdk_ses::config::Builder::from(&config).build();

        let inner = Client::from_conf(config);

        SesClient {
            inner,
            from_email_address,
        }
    }

    pub async fn send_email(&self, config: SendEmailConfig) -> anyhow::Result<()> {
        let destination = Destination::builder().to_addresses(config.to).build();
        let subject = Content::builder()
            .data(config.subject)
            .charset("UTF-8")
            .build()?;
        let body_text = Content::builder()
            .data(config.body)
            .charset("UTF-8")
            .build()?;
        let body = Body::builder().text(body_text).build();
        let message = Message::builder().subject(subject).body(body).build();

        self.inner
            .send_email()
            .source(&self.from_email_address)
            .reply_to_addresses(config.reply_to)
            .destination(destination)
            .message(message)
            .send()
            .await?;

        Ok(())
    }
}
