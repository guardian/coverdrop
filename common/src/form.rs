use std::{fs, marker::PhantomData};

use chrono::{DateTime, Duration, Utc};
use hex_buffer_serde::Hex;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::{
    base64::{Base64, Standard},
    formats::Padded,
    serde_as,
};

use crate::crypto::{
    keys::{
        role::Role,
        serde::{PublicSigningKeyHex, SignatureHex},
        signing::{
            traits::{self, PublicSigningKey},
            SignedSigningKeyPair,
        },
        Ed25519PublicKey,
    },
    Signature,
};

pub const DEFAULT_FORM_TTL: Duration = Duration::hours(1);

#[serde_as]
#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Form<FormData, SignerRole>
where
    FormData: Serialize,
    SignerRole: Role,
{
    // The body represents some form data that has been turned to JSON and is stored as a byte array.
    // We use a byte array because something like a `String` would imply some semantics about the
    // data that we don't want to imply.
    //
    // JSON isn't great here since it introduces a lot of overhead. We could investigate something else
    // such as msgpack
    //
    // This is the data that is signed.
    #[serde_as(as = "Base64<Standard, Padded>")]
    body: Vec<u8>,
    #[serde(with = "SignatureHex")]
    signature: Signature<Vec<u8>>,

    // Timestamp (not used)
    created_at: DateTime<Utc>,

    // expiration timestamp
    not_valid_after: DateTime<Utc>,

    // The public key of the signer. This does not use our cryptographic type system to enforce
    // that the correct verifying key is looked up from an appropriate key repository
    //
    // It also has the side benefit that we don't have to be particular about what kind of key
    // we're using, e.g. is the key signed or unsigned.
    #[serde(with = "PublicSigningKeyHex")]
    signing_pk: Ed25519PublicKey,

    // Since our signature is actually over a serialized form of the `FormData` we need to
    // keep track of it using a phantom data marker.
    #[serde(skip)]
    form_marker: PhantomData<FormData>,
    #[serde(skip)]
    role_marker: PhantomData<SignerRole>,
}

impl<FormData, SignerRole> Form<FormData, SignerRole>
where
    FormData: Serialize + DeserializeOwned,
    SignerRole: Role,
{
    // Creates a new byte vector filled with the data we want to be signed
    //
    // Note: I tested reusing the body Vec and truncating once we've done the
    // signing or verification, but performance didn't really improve.
    // Creating a new buffer is a bit cleaner imo.
    //
    // This is perhaps due to the fact that the serde library make the vector capacity fit exactly
    // during deserialization so we needed a reallocation anyway.
    #[inline]
    fn new_signing_buf(
        body: &[u8],
        created_at: DateTime<Utc>,
        not_valid_after: DateTime<Utc>,
    ) -> Vec<u8> {
        let created_at_str = created_at.to_rfc3339();
        let not_valid_after_str = not_valid_after.to_rfc3339();

        let mut buf =
            Vec::with_capacity(body.len() + created_at_str.len() + not_valid_after_str.len());
        buf.extend_from_slice(body);
        buf.extend_from_slice(created_at_str.as_bytes());
        buf.extend_from_slice(not_valid_after_str.as_bytes());

        buf
    }

    // Slightly wonky name which allows different typedefs to use `new`
    pub fn new_from_form_data(
        form: FormData,
        signing_key_pair: &SignedSigningKeyPair<SignerRole>,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let body = serde_json::to_vec(&form)?;
        let not_valid_after = now + DEFAULT_FORM_TTL;
        let buf = Self::new_signing_buf(&body, now, not_valid_after);
        Ok(Self {
            body,
            signature: signing_key_pair.sign(&buf),
            signing_pk: signing_key_pair.raw_public_key(),
            created_at: now,
            not_valid_after,
            form_marker: PhantomData,
            role_marker: PhantomData,
        })
    }

    pub fn new_from_form_data_custom_ttl(
        form: FormData,
        signing_key_pair: &SignedSigningKeyPair<SignerRole>,
        form_ttl: Duration,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let body = serde_json::to_vec(&form)?;
        let not_valid_after = now + form_ttl;
        let buf = Self::new_signing_buf(&body, now, not_valid_after);
        Ok(Self {
            body,
            signature: signing_key_pair.sign(&buf),
            signing_pk: signing_key_pair.raw_public_key(),
            created_at: now,
            not_valid_after,
            form_marker: PhantomData,
            role_marker: PhantomData,
        })
    }

    /// The public key that signed this form. It's important that this key is checked
    /// against a key repository to ensure that the key is valid and trusted.
    pub fn signing_pk(&self) -> &Ed25519PublicKey {
        &self.signing_pk
    }

    pub fn to_verified_form_data(
        &self,
        verifying_pk: &impl traits::PublicSigningKey<SignerRole>,
        now: DateTime<Utc>,
    ) -> anyhow::Result<FormData> {
        // Compare the signing key to the verifying key to ensure that they are the same.
        // This allows us to trust the signing key since the `verifying_pk` is has already
        // been verified, and the type system enforces that.
        //
        // If the client which created this form is lying about the `signing_pk` then the
        // signature check below will fail.
        if verifying_pk.raw_public_key() != self.signing_pk {
            anyhow::bail!("Form's signing key does not match the verifying key");
        }

        // Is this form submission sufficiently fresh
        if now > self.not_valid_after {
            anyhow::bail!("Form is not sufficiently fresh");
        }

        let buf = Self::new_signing_buf(&self.body, self.created_at, self.not_valid_after);

        verifying_pk.verify(&buf, &self.signature, now)?;

        Ok(serde_json::from_slice(&self.body)?)
    }

    pub fn not_valid_after(&self) -> DateTime<Utc> {
        self.not_valid_after
    }

    // TODO the form type should be in the type system, then we won't need to pass in the file name
    pub fn save_to_disk(&self, path: impl AsRef<std::path::Path>) -> anyhow::Result<()> {
        let json = serde_json::to_string(self)?;

        fs::write(&path, json)?;

        Ok(())
    }
}
