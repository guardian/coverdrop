use std::io::{Read, Write};
use std::mem::size_of;

use flate2::read::GzDecoder;
use flate2::{write::GzEncoder, Compression};
use serde::{Deserialize, Serialize};
use sodiumoxide::randombytes::randombytes;

use crate::crypto::Encryptable;
use crate::protocol::constants::MESSAGE_PADDING_LEN;
use crate::Error;

/// The default size of `PaddedCompressedString` for use in the CoverDrop protocol
pub type FixedSizeMessageText = PaddedCompressedString<MESSAGE_PADDING_LEN>;

/// A string of UTF-8 which has been compressed using GZip then padded to a specified length.
///
/// This is useful when you want to be able to send variable length messages using a fixed size buffer.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent, deny_unknown_fields)]
pub struct PaddedCompressedString<const PAD_TO: LengthHeader>(Vec<u8>);

// Given the use case of this structure, we're never likely to need more than 2^16 bytes.
// Can totally change it later on.
// We could also parametize the PaddedCompressedString type by the type of the length header

/// The type of the header which indicates how long the **compressed** string is.
/// By keeping this small we save extra bytes for the users message
type LengthHeader = u16;

impl<const PAD_TO: LengthHeader> PaddedCompressedString<PAD_TO> {
    const HEADER_SIZE: usize = size_of::<LengthHeader>();

    /// The total length of this buffer, used when allocating arrays which need `usize`.
    pub const TOTAL_LEN: usize = PAD_TO as usize;

    pub fn new(text: &str) -> Result<PaddedCompressedString<PAD_TO>, Error> {
        // Allocate at the expected size up front.
        let mut buf: Vec<u8> = Vec::with_capacity(Self::TOTAL_LEN);

        // Reserve bytes which we'll need later for writing the size
        // This saves us from prepending later on, O(n), horrifying!
        buf.resize(Self::HEADER_SIZE, 0x0);

        let mut compression = GzEncoder::new(buf, Compression::default());
        compression.write_all(text.as_bytes())?;

        let mut buf = compression.finish()?;

        // Write the size of the compressed bytes into the header we reserved previously
        let compressed_size = LengthHeader::try_from(buf.len() - Self::HEADER_SIZE)
            .map_err(|_| Error::CompressedStringTooLong(buf.len() as f32 / PAD_TO as f32))?; // The compressed string is >2^16 bytes
        let compressed_size_bytes = compressed_size.to_be_bytes();
        buf[..Self::HEADER_SIZE].copy_from_slice(&compressed_size_bytes[..Self::HEADER_SIZE]);

        if buf.len() > Self::TOTAL_LEN {
            return Err(Error::CompressedStringTooLong(
                buf.len() as f32 / PAD_TO as f32,
            ));
        }

        // pad with random bytes to match specified length
        let mut padding_bytes = randombytes(Self::TOTAL_LEN - buf.len());
        buf.append(&mut padding_bytes);

        Ok(PaddedCompressedString(buf))
    }

    /// Construct a `PaddedCompressedString` from a given vector of bytes.
    /// Asserts that the vector is the correct size but does not check the contents is
    /// valid compressed UTF-8
    pub fn from_vec_unchecked(bytes: Vec<u8>) -> PaddedCompressedString<PAD_TO> {
        assert_eq!(bytes.len(), Self::TOTAL_LEN);
        PaddedCompressedString(bytes)
    }

    pub fn to_string(&self) -> Result<String, Error> {
        let buf: &[u8] = &self.0;

        let compressed_size = self.compressed_data_len()?;
        let compressed_bytes = &buf[Self::HEADER_SIZE..compressed_size + Self::HEADER_SIZE];
        let mut decoder = GzDecoder::new(compressed_bytes);

        let mut text = String::with_capacity(compressed_size * 4);
        decoder.read_to_string(&mut text)?;

        // The maximum compression ratio is ~1000:1. This is our (256 byte) messages would not
        // decode to output larger than 256 KiB. Nevertheless, we assume that everything with a
        // compression ratio larger than 100:1 is suspicious for natural text and we drop it.
        // See: https://github.com/guardian/coverdrop/issues/112
        let decompression_ratio = text.len() / compressed_size;
        if decompression_ratio > 100 {
            return Err(Error::DecompressionRatioTooHigh);
        }

        Ok(text)
    }

    pub fn total_len(&self) -> usize {
        self.0.len()
    }

    pub fn compressed_data_len(&self) -> Result<usize, Error> {
        let buf: &[u8] = &self.0;

        let compressed_size_bytes: [u8; size_of::<LengthHeader>()] = buf[..Self::HEADER_SIZE]
            .try_into()
            .map_err(|_| Error::InvalidPaddedCompressedString)?;
        Ok(LengthHeader::from_be_bytes(compressed_size_bytes) as usize)
    }

    pub fn padding_len(&self) -> Result<usize, Error> {
        Ok(self.total_len() - self.compressed_data_len()? - Self::HEADER_SIZE)
    }

    pub fn fill_level(&self) -> Result<f32, Error> {
        let max_compressed_data_len = self.total_len() - Self::HEADER_SIZE;
        Ok(self.compressed_data_len()? as f32 / max_compressed_data_len as f32)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl<const PAD_TO: LengthHeader> Encryptable for PaddedCompressedString<PAD_TO> {
    fn as_unencrypted_bytes(&self) -> &[u8] {
        &self.0
    }

    fn from_unencrypted_bytes(bytes: Vec<u8>) -> Result<Self, Error> {
        Ok(PaddedCompressedString(bytes))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Error, FixedSizeMessageText, PaddedCompressedString};

    #[test]
    fn successfully_roundtrip() -> Result<(), Error> {
        let expected = "hello world";

        const TARGET_LEN: u16 = 64;

        let pcs = PaddedCompressedString::<TARGET_LEN>::new(expected)?;

        assert_eq!(
            TARGET_LEN as usize,
            pcs.total_len(),
            "Check expected length"
        );

        let actual = pcs.to_string()?;

        assert_eq!(
            expected, actual,
            "Check the text is the same after roundtripping"
        );

        Ok(())
    }

    #[test]
    fn is_always_the_same_size() -> Result<(), Error> {
        let messages = vec![
            "a",
            "this is a small message",
            "this is a longer message with a few extra words",
        ];

        const TARGET_LEN: u16 = 128;

        for message in messages {
            let pcs = PaddedCompressedString::<TARGET_LEN>::new(message)?;

            assert_eq!(pcs.total_len(), TARGET_LEN as usize);
        }

        Ok(())
    }

    #[test]
    fn will_error_if_string_is_too_long() {
        let message = r#"Lorem ipsum dolor sit amet, consectetur adipiscing elit.
            Donec hendrerit mauris nibh, et blandit ex venenatis ut. Nullam nec lorem enim.
            Nam dignissim, metus in pulvinar luctus, eros mi congue libero, non dignissim nisi nunc vitae mi.
            Proin sagittis diam quis est posuere luctus. Vivamus vitae lectus neque.
            Morbi et mollis libero, vitae vestibulum lorem.
            Etiam ornare enim vel sem placerat, nec tempus massa fringilla. Nam eu nibh at nulla aliquet mattis.
            Praesent hendrerit lacinia tempus. Vivamus molestie diam nisi, in finibus libero dictum et.
            Quisque condimentum consequat elit, in tempor augue posuere non. 
            Nunc porttitor, leo eu mollis tincidunt, libero nisi fermentum libero, sed feugiat sem purus a ante.
            Donec condimentum aliquam augue, sit amet aliquet felis vehicula non.
            Quisque urna dolor, accumsan non ullamcorper sodales, fermentum ac mi."#;

        let pcs = PaddedCompressedString::<128>::new(message);

        assert!(matches!(pcs, Err(Error::CompressedStringTooLong(_))));
    }

    #[test]
    fn if_decompression_ratio_too_high_then_error() -> Result<(), Error> {
        let message = "a".repeat(10000);
        let pcs = PaddedCompressedString::<128>::new(&message)?;
        let actual = pcs.to_string();
        assert!(matches!(actual, Err(Error::DecompressionRatioTooHigh)));
        Ok(())
    }

    #[test]
    fn fixed_sized_text_of_empty_string_always_succeeds() -> Result<(), Error> {
        FixedSizeMessageText::new("")?;
        Ok(())
    }

    #[test]
    fn nondeterministic_is_padded_with_non_zero_bytes() -> Result<(), Error> {
        let pcs = PaddedCompressedString::<512>::new("")?;

        let suffix = &pcs.as_bytes()[100..];
        assert!(suffix.len() >= 100);
        assert!(suffix.iter().filter(|&&x| x == 0).count() < 10);

        Ok(())
    }
}
