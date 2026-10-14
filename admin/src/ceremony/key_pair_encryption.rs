use age::{secrecy::SecretString, Encryptor};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fs::{write, File},
    io::Write,
};

use common::{
    crypto::{
        keys::{
            role::Role,
            serde::{set_key_permissions, StorableKeyMaterial},
            signing::{SignedPublicSigningKey, SigningKeyPair},
            untrusted::signing::UntrustedSignedSigningKeyPair,
        },
        pbkdf::DEFAULT_PASSPHRASE_WORDS,
    },
    generators::PasswordGenerator,
    protocol::roles::Organization,
};

use super::*;

/// Takes an untrusted signing key pair and an output path, generates a passphrase,
/// encrypts the key pair with it using age, and writes the encrypted key pair to the specified output path.
/// Returns the generated passphrase.
fn age_encrypt_key_pair_file<R: Role + Serialize>(
    untrusted_key_pair: &UntrustedSignedSigningKeyPair<R>,
    output_path: &Path,
) -> anyhow::Result<String> {
    let password_generator = PasswordGenerator::from_eff_large_wordlist()?;
    let passphrase = password_generator.generate(DEFAULT_PASSPHRASE_WORDS);

    let key_pair_json = serde_json::to_vec(&untrusted_key_pair)?;

    let encryptor = Encryptor::with_user_passphrase(SecretString::from(passphrase.clone()));

    let mut writer = encryptor.wrap_output(File::create(output_path)?)?;
    writer.write_all(&key_pair_json)?;
    writer.finish()?;

    set_key_permissions(output_path);

    Ok(passphrase)
}

/// Encrypt a provisioning key pair with a freshly generated passphrase and prompt the user to move
/// the encrypted copy to the given USB device and the passphrase to the same bag.
///
/// Returns the path of the encrypted key pair file written to disk.
pub(super) fn age_encrypt_and_prompt_transfer_of_provisioning_key<
    R: Role + Serialize + DeserializeOwned,
>(
    untrusted_key_pair: &UntrustedSignedSigningKeyPair<R>,
    output_directory: &Path,
    usb_device: &UsbDevice,
    assume_yes: &AssumeYes,
) -> anyhow::Result<PathBuf> {
    let key_pair_display = R::display();
    let key_pair_description = format!("{} key pair", key_pair_display);

    let encrypted_file_path =
        output_directory.join(format!("{}.age", untrusted_key_pair.file_name()));

    let passphrase = age_encrypt_key_pair_file(untrusted_key_pair, &encrypted_file_path)?;

    let prompt = [
        &format!("The {key_pair_description} has been created. A copy has been encrypted and saved to disk at the following location:"),
        encrypted_file_path.to_str().unwrap(),
        &format!(
            "Copy the encrypted {key_pair_description} to removable device '{}'.",
            usb_device.description()
        ),
        "Have you completed this step?",
    ].join("\n");
    ask_user_to_confirm(&prompt, *assume_yes)?;

    let prompt = [
        &format!(
            "The {key_pair_description} has been age encrypted with the following passphrase: \n'{passphrase}'"
        ),
        &format!(
            "Write this passphrase down on a piece of paper, and put it in the same tamper-proof bag as '{}'",
            usb_device.description()
        ),
        "Have you completed this step?",
    ].join("\n");
    ask_user_to_confirm(&prompt, *assume_yes)?;

    Ok(encrypted_file_path)
}

/// Generate a passphrase, encrypt the org key pair with it using age and prompt
/// the user to move the encrypted key pair and the passphrase to removable devices.
///
/// The unencrypted key pair is never written to disk.
pub(super) fn age_encrypt_org_key_pair(
    org_key_pair_copy: Copy,
    org_key_pair: &SigningKeyPair<Organization, SignedPublicSigningKey<Organization>>,
    output_directory: &Path,
    assume_yes: &AssumeYes,
) -> anyhow::Result<()> {
    let untrusted_org_key_pair = org_key_pair.to_untrusted();

    let encrypted_org_key_pair_file = output_directory.join(format!(
        "copy{}_{}.age",
        org_key_pair_copy,
        untrusted_org_key_pair.file_name()
    ));

    let passphrase =
        age_encrypt_key_pair_file(&untrusted_org_key_pair, &encrypted_org_key_pair_file)?;

    println!(
        "Organization key pair {} encrypted with age and written to {}",
        org_key_pair_copy,
        encrypted_org_key_pair_file.to_str().unwrap()
    );

    // write passphrase to a file
    let passphrase_file =
        output_directory.join(format!("org_key_pair_passphrase_{}.txt", org_key_pair_copy));
    write(&passphrase_file, &passphrase)?;

    set_key_permissions(&passphrase_file);

    println!(
        "passphrase {} written to {}",
        org_key_pair_copy,
        passphrase_file.to_str().unwrap()
    );

    ask_user_to_confirm(
        &format!(
            "Move org key {} to removable device '{}' and passphrase {} to removable device '{}'",
            encrypted_org_key_pair_file.to_str().unwrap(),
            UsbDevice::OrgKeyPair(org_key_pair_copy.clone()).description(),
            passphrase_file.to_str().unwrap(),
            UsbDevice::OrgKeyPairPassword(org_key_pair_copy).description(),
        ),
        *assume_yes,
    )?;

    Ok(())
}
