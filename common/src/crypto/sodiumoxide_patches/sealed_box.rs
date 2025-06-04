use sodiumoxide::crypto::box_::PublicKey;
use sodiumoxide::crypto::sealedbox::SEALBYTES;

/// A modified version of the sodiumoxide version which checks the internal return value. The
/// error handling matches those of other related methods that do error checking.
///
/// Copied and patched from: curve25519blake2bxsalsa20poly1305.rs
pub fn seal(m: &[u8], pk: &PublicKey) -> Result<Vec<u8>, ()> {
    let mut c = vec![0u8; m.len() + SEALBYTES];
    let ret = unsafe {
        ffi::crypto_box_seal(
            c.as_mut_ptr(),
            m.as_ptr(),
            m.len() as libc::c_ulonglong,
            pk.0.as_ptr(),
        )
    };
    if ret == 0 {
        Ok(c)
    } else {
        Err(())
    }
}

mod ffi {
    extern "C" {
        pub fn crypto_box_seal(
            c: *mut libc::c_uchar,
            m: *const libc::c_uchar,
            mlen: libc::c_ulonglong,
            pk: *const libc::c_uchar,
        ) -> libc::c_int;
    }
}

#[cfg(test)]
mod tests {
    use sodiumoxide::crypto::box_;
    use sodiumoxide::randombytes::randombytes;

    #[test]
    fn round_trip() {
        let m = randombytes(42);
        let (pk, sk) = box_::gen_keypair();

        // encrypt with original and patched function
        let c_original = sodiumoxide::crypto::sealedbox::seal(&m, &pk);
        let c_patched = crate::crypto::sodiumoxide_patches::sealed_box::seal(&m, &pk).unwrap();

        // note that the ciphertexts will not match due to the ephemeral key
        assert_ne!(c_original, c_patched);

        // decrypted plaintexts match initial message
        let m_original = sodiumoxide::crypto::sealedbox::open(&c_original, &pk, &sk);
        assert_eq!(m_original, Ok(m.clone()));

        let m_patched = sodiumoxide::crypto::sealedbox::open(&c_patched, &pk, &sk);
        assert_eq!(m_patched, Ok(m.clone()));
    }
}
