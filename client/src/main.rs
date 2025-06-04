use clap::Parser;
use client::{
    cli::{Cli, Command},
    commands::{journalist::handle_journalist_command, user::handle_user_commands},
};
use common::{
    api::api_client::ApiClient, client::mailbox::user_mailbox::UserMailbox,
    generators::PasswordGenerator, time, FixedSizeMessageText,
};
use common::{aws::kinesis::client::KinesisClient, crypto::pbkdf::DEFAULT_PASSPHRASE_WORDS};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let api_client = ApiClient::new(cli.api_url);

    let result: anyhow::Result<()> = match cli.command {
        Command::CheckMessageLength { message } => {
            let original_len = message.len();
            let padded = FixedSizeMessageText::new(&message)?;

            println!("original\tcompressed\ttotal",);
            println!(
                "{}\t{}\t{}",
                original_len,
                padded.compressed_data_len()?,
                padded.total_len()
            );

            Ok(())
        }
        Command::GenerateUser {
            mut mailbox_path,
            password,
        } => {
            let password = password.map(anyhow::Ok).unwrap_or_else(|| {
                let password_generator = PasswordGenerator::from_eff_large_wordlist()?;
                anyhow::Ok(password_generator.generate(DEFAULT_PASSPHRASE_WORDS))
            })?;

            println!("{}", &password);

            if mailbox_path.is_dir() {
                mailbox_path.push("user");
            };

            if mailbox_path.extension().is_none() {
                mailbox_path.set_extension("mailbox");
            }

            let keys = api_client.get_public_keys().await?;
            let org_pks = keys.untrusted_org_pk_iter();

            UserMailbox::new(&password, org_pks, mailbox_path, time::now())?;

            Ok(())
        }
        Command::User {
            command,
            mailbox_path,
            password,
            password_path,
        } => handle_user_commands(mailbox_path, password, password_path, command, api_client).await,
        Command::Journalist {
            command,
            vault_path,
            password,
            password_path,
            kinesis_config,
            aws_config,
        } => {
            let kinesis_client = KinesisClient::new(
                &kinesis_config,
                &aws_config,
                vec![kinesis_config.journalist_stream.clone()],
            )
            .await;
            handle_journalist_command(
                vault_path,
                password,
                password_path,
                command,
                api_client,
                kinesis_client,
            )
            .await
        }
    };

    if let Err(error) = result {
        eprintln!("Failed to run command with error: {}", error);
        std::process::exit(1);
    }

    Ok(())
}
