use std::path::Path;

use common::api::models::journalist_id::MAX_JOURNALIST_IDENTITY_LEN;
use common::api::models::messages::{
    FLAG_J2U_MESSAGE_TYPE_HANDOVER, FLAG_J2U_MESSAGE_TYPE_MESSAGE, RECIPIENT_TAG_BYTES_U2J_COVER,
};
use common::protocol::constants::*;
use convert_case::{Case, Casing};
use std::fs::File;
use std::io::{LineWriter, Write};

const ANDROID_PACKAGE_DECLARATION: &[u8] = b"package com.theguardian.coverdrop.core.generated\n\n";

const TOP_OF_FILE_COMMENT: &[u8] =
    r#"// DO NOT EDIT! This file is auto-generated from Rust code using the following command:
// `cargo run --bin admin generate-mobile-constants-files`.
//
// The Rust code is here: common/src/protocol/constants.rs

"#
    .as_bytes();

const IOS_CONSTANTS_STRUCT_START: &[u8] =
    b"// swiftlint:disable identifier_name\npublic enum Constants {\n";
const IOS_CONSTANTS_STRUCT_END: &[u8] = b"}\n\n// swiftlint:enable identifier_name";

pub fn generate_constant_files(android_path: &Path, ios_path: &Path) -> anyhow::Result<()> {
    generate_constants_file_android(android_path)?;
    generate_constants_file_ios(ios_path)?;
    Ok(())
}

macro_rules! format_android_constant_val {
    ($var:expr) => {{
        format_args!(
            "internal const val {:} = {:}\n",
            stringify!($var),
            format_number($var.try_into().unwrap()),
        )
    }};
}

macro_rules! format_android_constant_val_byte {
    ($var:expr) => {{
        format_args!(
            "internal const val {:}: Byte = 0x{:02x}\n",
            stringify!($var),
            $var,
        )
    }};
}

macro_rules! format_android_bytearray {
    ($var:expr) => {{
        format_args!(
            "internal val {:} = byteArrayOf({})\n",
            stringify!($var),
            format_bytes(&$var),
        )
    }};
}

fn generate_constants_file_android(path: &Path) -> anyhow::Result<()> {
    let file = File::create(path)?;
    let mut writer = LineWriter::new(file);

    writer.write_all(ANDROID_PACKAGE_DECLARATION)?;
    writer.write_all(TOP_OF_FILE_COMMENT)?;

    writer.write_fmt(format_android_constant_val!(
        JOURNALIST_PROVISIONING_KEY_VALID_DURATION_SECONDS
    ))?;

    writer.write_fmt(format_android_constant_val!(
        JOURNALIST_ID_KEY_VALID_DURATION_SECONDS
    ))?;

    writer.write_fmt(format_android_constant_val!(
        JOURNALIST_MSG_KEY_VALID_DURATION_SECONDS
    ))?;

    writer.write_fmt(format_android_constant_val!(
        COVERNODE_PROVISIONING_KEY_VALID_DURATION_SECONDS
    ))?;

    writer.write_fmt(format_android_constant_val!(
        COVERNODE_ID_KEY_VALID_DURATION_SECONDS
    ))?;

    writer.write_fmt(format_android_constant_val!(
        COVERNODE_MSG_KEY_VALID_DURATION_SECONDS
    ))?;

    writer.write_fmt(format_android_constant_val!(
        USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN
    ))?;
    writer.write_fmt(format_android_constant_val!(USER_TO_COVERNODE_MESSAGE_LEN))?;
    writer.write_fmt(format_android_constant_val!(
        USER_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN
    ))?;
    writer.write_fmt(format_android_constant_val!(USER_TO_JOURNALIST_MESSAGE_LEN))?;
    writer.write_fmt(format_android_constant_val!(
        JOURNALIST_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN
    ))?;
    writer.write_fmt(format_android_constant_val!(
        JOURNALIST_TO_COVERNODE_MESSAGE_LEN
    ))?;
    writer.write_fmt(format_android_constant_val!(
        JOURNALIST_TO_USER_ENCRYPTED_MESSAGE_LEN
    ))?;
    writer.write_fmt(format_android_constant_val!(JOURNALIST_TO_USER_MESSAGE_LEN))?;
    writer.write_fmt(format_android_constant_val!(MESSAGE_PADDING_LEN))?;
    writer.write_fmt(format_android_constant_val!(RECIPIENT_TAG_LEN))?;

    writer.write_fmt(format_android_constant_val!(
        MESSAGE_VALID_FOR_DURATION_IN_SECONDS
    ))?;
    writer.write_fmt(format_android_constant_val!(
        MESSAGE_EXPIRY_WARNING_IN_SECONDS
    ))?;
    writer.write_fmt(format_android_constant_val!(
        CLIENT_DEAD_DROP_CACHE_TTL_SECONDS
    ))?;
    writer.write_fmt(format_android_constant_val!(
        CLIENT_DEFAULT_DOWNLOAD_RATE_SECONDS
    ))?;
    writer.write_fmt(format_android_constant_val!(
        CLIENT_STATUS_DOWNLOAD_RATE_SECONDS
    ))?;
    writer.write_fmt(format_android_constant_val!(COVERNODE_WRAPPING_KEY_COUNT))?;

    writer.write_fmt(format_android_constant_val!(MAX_JOURNALIST_IDENTITY_LEN))?;
    writer.write_fmt(format_android_constant_val_byte!(
        FLAG_J2U_MESSAGE_TYPE_MESSAGE
    ))?;
    writer.write_fmt(format_android_constant_val_byte!(
        FLAG_J2U_MESSAGE_TYPE_HANDOVER
    ))?;
    writer.write_fmt(format_android_bytearray!(RECIPIENT_TAG_BYTES_U2J_COVER))?;

    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

macro_rules! format_ios_let_constant {
    ($var:expr) => {{
        format_args!(
            "    public static let {:} = {:}\n",
            (stringify!($var)).to_case(Case::Camel),
            format_number($var.try_into().unwrap()),
        )
    }};
}

macro_rules! format_ios_let_constant_byte {
    ($var:expr) => {{
        format_args!(
            "    public static let {:}: UInt8 = 0x{:02x}\n",
            (stringify!($var)).to_case(Case::Camel),
            $var,
        )
    }};
}

macro_rules! format_ios_byte_array {
    ($var:expr) => {{
        format_args!(
            "    public static let {:} = [{:}]\n",
            (stringify!($var)).to_case(Case::Camel),
            format_bytes(&$var),
        )
    }};
}

fn generate_constants_file_ios(path: &Path) -> anyhow::Result<()> {
    let file = File::create(path)?;
    let mut writer = LineWriter::new(file);

    writer.write_all(TOP_OF_FILE_COMMENT)?;

    writer.write_all(IOS_CONSTANTS_STRUCT_START)?;

    writer.write_fmt(format_ios_let_constant!(
        JOURNALIST_PROVISIONING_KEY_VALID_DURATION_SECONDS
    ))?;

    writer.write_fmt(format_ios_let_constant!(
        JOURNALIST_ID_KEY_VALID_DURATION_SECONDS
    ))?;

    writer.write_fmt(format_ios_let_constant!(
        JOURNALIST_MSG_KEY_VALID_DURATION_SECONDS
    ))?;

    writer.write_fmt(format_ios_let_constant!(
        COVERNODE_PROVISIONING_KEY_VALID_DURATION_SECONDS
    ))?;

    writer.write_fmt(format_ios_let_constant!(
        COVERNODE_ID_KEY_VALID_DURATION_SECONDS
    ))?;

    writer.write_fmt(format_ios_let_constant!(
        COVERNODE_MSG_KEY_VALID_DURATION_SECONDS
    ))?;

    writer.write_fmt(format_ios_let_constant!(
        USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN
    ))?;
    writer.write_fmt(format_ios_let_constant!(USER_TO_COVERNODE_MESSAGE_LEN))?;
    writer.write_fmt(format_ios_let_constant!(
        USER_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN
    ))?;
    writer.write_fmt(format_ios_let_constant!(USER_TO_JOURNALIST_MESSAGE_LEN))?;
    writer.write_fmt(format_ios_let_constant!(
        JOURNALIST_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN
    ))?;
    writer.write_fmt(format_ios_let_constant!(
        JOURNALIST_TO_COVERNODE_MESSAGE_LEN
    ))?;
    writer.write_fmt(format_ios_let_constant!(
        JOURNALIST_TO_USER_ENCRYPTED_MESSAGE_LEN
    ))?;
    writer.write_fmt(format_ios_let_constant!(JOURNALIST_TO_USER_MESSAGE_LEN))?;
    writer.write_fmt(format_ios_let_constant!(MESSAGE_PADDING_LEN))?;
    writer.write_fmt(format_ios_let_constant!(RECIPIENT_TAG_LEN))?;

    writer.write_fmt(format_ios_let_constant!(REAL_OR_COVER_BYTE_LEN))?;
    writer.write_fmt(format_ios_let_constant!(X25519_PUBLIC_KEY_LEN))?;
    writer.write_fmt(format_ios_let_constant!(X25519_SECRET_KEY_LEN))?;
    writer.write_fmt(format_ios_let_constant!(POLY1305_AUTH_TAG_LEN))?;
    writer.write_fmt(format_ios_let_constant!(TWO_PARTY_BOX_NONCE_LEN))?;

    writer.write_fmt(format_ios_let_constant!(
        MESSAGE_VALID_FOR_DURATION_IN_SECONDS
    ))?;
    writer.write_fmt(format_ios_let_constant!(MESSAGE_EXPIRY_WARNING_IN_SECONDS))?;
    writer.write_fmt(format_ios_let_constant!(MAX_BACKGROUND_DURATION_IN_SECONDS))?;
    writer.write_fmt(format_ios_let_constant!(CLIENT_DEAD_DROP_CACHE_TTL_SECONDS))?;
    writer.write_fmt(format_ios_let_constant!(
        CLIENT_DEFAULT_DOWNLOAD_RATE_SECONDS
    ))?;
    writer.write_fmt(format_ios_let_constant!(
        CLIENT_STATUS_DOWNLOAD_RATE_SECONDS
    ))?;

    writer.write_fmt(format_ios_let_constant!(COVERNODE_WRAPPING_KEY_COUNT))?;

    writer.write_fmt(format_ios_let_constant!(MAX_JOURNALIST_IDENTITY_LEN))?;
    writer.write_fmt(format_ios_let_constant_byte!(FLAG_J2U_MESSAGE_TYPE_MESSAGE))?;
    writer.write_fmt(format_ios_let_constant_byte!(
        FLAG_J2U_MESSAGE_TYPE_HANDOVER
    ))?;
    writer.write_fmt(format_ios_byte_array!(RECIPIENT_TAG_BYTES_U2J_COVER))?;

    writer.write_all(IOS_CONSTANTS_STRUCT_END)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

fn format_number(num: usize) -> String {
    // 10_000 is the threshold for most formatters to require a separator
    if num < 10_000 {
        num.to_string()
    } else {
        num.to_string()
            .as_bytes()
            .rchunks(3)
            .rev()
            .map(|x| std::str::from_utf8(x).unwrap())
            .collect::<Vec<_>>()
            .join("_")
    }
}

fn format_bytes(bytes: &[u8]) -> String {
    let mut result = Vec::<String>::new();
    for byte in bytes {
        result.push(format!("0x{:02x}", byte));
    }
    result.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(1), "1");
        assert_eq!(format_number(12), "12");
        assert_eq!(format_number(123), "123");
        assert_eq!(format_number(1234), "1234");
        assert_eq!(format_number(12345), "12_345");
        assert_eq!(format_number(123456), "123_456");
        assert_eq!(format_number(1234567), "1_234_567");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(&[0, 1, 2, 255]), "0x00, 0x01, 0x02, 0xff");
    }
}
