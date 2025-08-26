use crate::Error;
use serde::{Deserialize, Serialize};
use sodiumoxide::randombytes::randombytes;
use std::mem::size_of;

/// A vector of bytes that is padded to a specific length. The length is signaled using a prefix
/// header [LengthHeader] which indicates the length of the actual data in the vector.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent, deny_unknown_fields)]
pub struct PaddedByteVector(Vec<u8>);

type LengthHeader = u32;

impl PaddedByteVector {
    const HEADER_SIZE: usize = size_of::<LengthHeader>();

    /// Create a new `PaddedByteVector` from a vector of bytes, padding it to the specified length.
    /// The `pad_to` parameter specifies the total length of the padded byte vector including
    /// the header.
    pub fn new(payload: Vec<u8>, pad_to: u32) -> Result<PaddedByteVector, Error> {
        let pad_to: usize = usize::try_from(pad_to).expect("usize >= u32");
        let required_space = Self::HEADER_SIZE + payload.len();
        if required_space > pad_to {
            return Err(Error::PaddedByteVectorNotEnoughSpace(
                pad_to as u64,
                required_space as u64,
            ));
        }

        // Allocate at the expected size up front.
        let mut buf: Vec<u8> = Vec::with_capacity(pad_to);

        // Add the header of the size of the byte array to the beginning of the buffer
        let header = LengthHeader::try_from(payload.len())
            .map_err(|_| Error::PaddedContentTooLarge(payload.len() as u64))?;
        let header_bytes = header.to_be_bytes();
        buf.extend(header_bytes);

        // Consume bytes into padded byte vector as the main payload
        buf.extend(payload);

        // Pad with random bytes to match specified length
        let padding_size = pad_to - buf.len();
        let padding_bytes = randombytes(padding_size);
        buf.extend(padding_bytes);

        assert_eq!(
            buf.len(),
            pad_to,
            "Buffer length does not match specified padding length"
        );
        Ok(PaddedByteVector(buf))
    }

    pub fn into_unpadded(self) -> Result<Vec<u8>, Error> {
        let byte_array_size = self.payload_len()?;
        let payload = &self.0[Self::HEADER_SIZE..Self::HEADER_SIZE + byte_array_size];
        Ok(payload.to_vec())
    }

    pub fn total_len(&self) -> usize {
        self.0.len()
    }

    pub fn payload_len(&self) -> Result<usize, Error> {
        if self.0.len() < Self::HEADER_SIZE {
            return Err(Error::PaddedByteArrayInvalid);
        }
        let compressed_size_bytes: [u8; size_of::<LengthHeader>()] = self.0[..Self::HEADER_SIZE]
            .try_into()
            .map_err(|_| Error::PaddedByteArrayInvalid)?;
        Ok(LengthHeader::from_be_bytes(compressed_size_bytes) as usize)
    }

    pub fn padding_len(&self) -> Result<usize, Error> {
        Ok(self.total_len() - self.payload_len()? - Self::HEADER_SIZE)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// A variant of the [PaddedByteVector] that is dynamically padded to the next larger multiple of
/// the given `pad_to_step_size`. This is useful for cases where the payload size is not known
/// in advance, and we want to ensure (also statically at compile-time) that the padded byte
/// vector is always a multiple of a certain size.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent, deny_unknown_fields)]
pub struct SteppingPaddedByteVector<const PAD_TO_STEP_SIZE: usize>(PaddedByteVector);

impl<const PAD_TO_STEP_SIZE: usize> SteppingPaddedByteVector<PAD_TO_STEP_SIZE> {
    /// Create a new `SteppingPaddedByteVector` from a vector of bytes, padding it to the next
    /// larger multiple of `PAD_TO_STEP_SIZE` that fits both the header and the payload.
    pub fn new(payload: Vec<u8>) -> Result<Self, Error> {
        let minimum_size = payload.len() + PaddedByteVector::HEADER_SIZE;
        let pad_to_step_size = PAD_TO_STEP_SIZE;

        // Calculate the next multiple of `pad_to_step_size` that is greater than or equal to `minimum_size`.
        let pad_to = minimum_size.div_ceil(pad_to_step_size) * pad_to_step_size;
        let pad_to: u32 =
            u32::try_from(pad_to).map_err(|_| Error::PaddedContentTooLarge(minimum_size as u64))?;

        let padded = PaddedByteVector::new(payload, pad_to)?;
        Ok(SteppingPaddedByteVector(padded))
    }

    /// Prefer this method over `PaddedByteVector::from` to ensure the byte vector is a multiple of
    /// the step size. This can give additional type safety when deserializing.
    pub fn from(bytes: Vec<u8>) -> Result<Self, Error> {
        if bytes.len() % PAD_TO_STEP_SIZE != 0 {
            return Err(Error::PaddedByteVectorNotMultipleOfStepSize);
        }
        Ok(SteppingPaddedByteVector(PaddedByteVector(bytes)))
    }

    pub fn into_unpadded(self) -> Result<Vec<u8>, Error> {
        if self.0.total_len() % PAD_TO_STEP_SIZE != 0 {
            return Err(Error::PaddedByteVectorNotMultipleOfStepSize);
        }
        self.0.into_unpadded()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;

    #[test]
    fn test_round_trip_success() -> Result<(), Error> {
        let payload = b"Hello, World!".to_vec();
        let pad_to = 64;

        let padded = PaddedByteVector::new(payload.clone(), pad_to)?;
        let extracted = padded.into_unpadded()?;

        assert_eq!(payload, extracted);
        Ok(())
    }

    #[test]
    fn test_round_trip_empty_payload() -> Result<(), Error> {
        let payload = Vec::new();
        let pad_to = 32;

        let padded = PaddedByteVector::new(payload.clone(), pad_to)?;
        let extracted = padded.into_unpadded()?;

        assert_eq!(payload, extracted);
        Ok(())
    }

    #[test]
    fn test_payload_len_correct() -> Result<(), Error> {
        let payload = b"test payload".to_vec();
        let expected_len = payload.len();
        let pad_to = 64;

        let padded = PaddedByteVector::new(payload, pad_to)?;

        assert_eq!(padded.payload_len()?, expected_len);
        Ok(())
    }

    #[test]
    fn test_as_bytes_returns_full_buffer() -> Result<(), Error> {
        let payload = b"data".to_vec();
        let pad_to = 50;

        let padded = PaddedByteVector::new(payload, pad_to)?;

        assert_eq!(padded.as_bytes().len(), pad_to as usize);
        Ok(())
    }

    #[test]
    fn test_error_not_enough_space_1() {
        let payload = vec![1u8; 100];
        let pad_to = 2; // Too small to fit header

        let result = PaddedByteVector::new(payload, pad_to);
        assert!(matches!(
            result,
            Err(Error::PaddedByteVectorNotEnoughSpace(2, _))
        ));
    }

    #[test]
    fn test_error_not_enough_space_2() {
        let payload = vec![1u8; 100];
        let pad_to = 50; // Too small to fit header + payload

        let result = PaddedByteVector::new(payload, pad_to);
        assert!(matches!(
            result,
            Err(Error::PaddedByteVectorNotEnoughSpace(50, _))
        ));
    }

    #[test]
    fn test_exact_fit_no_padding() -> Result<(), Error> {
        let payload = vec![1u8; 10];
        let pad_to = (PaddedByteVector::HEADER_SIZE + payload.len()) as u32;

        let padded = PaddedByteVector::new(payload.clone(), pad_to)?;
        assert_eq!(padded.total_len(), pad_to as usize);
        assert_eq!(padded.payload_len()?, payload.len());
        assert_eq!(padded.padding_len()?, 0);

        let extracted = padded.into_unpadded()?;
        assert_eq!(payload, extracted);
        Ok(())
    }

    #[test]
    fn test_error_invalid_padded_byte_array() {
        let invalid_bytes = vec![0u8; 2]; // Less than HEADER_SIZE (4 bytes)
        let invalid_padded = PaddedByteVector(invalid_bytes);

        let result = invalid_padded.payload_len();
        assert!(matches!(result, Err(Error::PaddedByteArrayInvalid)));
    }

    #[test]
    fn test_multiple_different_payloads_same_total_length() -> Result<(), Error> {
        let payloads = vec![
            b"a".to_vec(),
            b"hello".to_vec(),
            b"this is a longer message".to_vec(),
        ];
        let pad_to = 100;

        for payload in payloads {
            let padded = PaddedByteVector::new(payload.clone(), pad_to)?;
            assert_eq!(padded.total_len(), pad_to as usize);

            let extracted = padded.into_unpadded()?;
            assert_eq!(payload, extracted);
        }
        Ok(())
    }

    #[test]
    fn test_stepping_padded_byte_vector() -> Result<(), Error> {
        const PAD_TO_STEP_SIZE: usize = 16;
        let payload = b"test payload".to_vec(); // 12 bytes + 4 bytes for header = 16 bytes total

        let padded = SteppingPaddedByteVector::<PAD_TO_STEP_SIZE>::new(payload.clone())?;
        assert_eq!(padded.0.total_len(), 16);

        let unpadded = padded.into_unpadded()?;
        assert_eq!(payload, unpadded);

        Ok(())
    }

    #[test]
    fn test_stepping_padded_byte_vector_all_padded_to_steps() -> Result<(), Error> {
        const PAD_TO_STEP_SIZE: usize = 16;

        // create vector payloads between 5 and 40 bytes
        let payloads: Vec<Vec<u8>> = (5..=40).map(|len| randombytes(len)).collect();

        for payload in payloads {
            let padded = SteppingPaddedByteVector::<PAD_TO_STEP_SIZE>::new(payload.clone())?;

            // Length is a multiple of PAD_TO_STEP_SIZE
            assert_eq!(padded.0.total_len() % PAD_TO_STEP_SIZE, 0);

            let unpadded = padded.into_unpadded()?;
            assert_eq!(payload, unpadded);
        }
        Ok(())
    }
}
