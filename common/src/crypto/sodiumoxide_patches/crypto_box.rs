use sodiumoxide::crypto::box_::{Nonce, PublicKey, SecretKey, MACBYTES};

/// A modified version of the sodiumoxide version which checks the internal return value. The
/// error handling matches those of other related methods that do error checking.
///
/// Copied and patched from: curve25519xsalsa20poly1305.rs
pub fn seal(m: &[u8], n: &Nonce, pk: &PublicKey, sk: &SecretKey) -> Result<Vec<u8>, ()> {
    let clen = m.len() + MACBYTES;
    let mut c = Vec::with_capacity(clen);
    let ret = unsafe {
        c.set_len(clen);
        ffi::crypto_box_easy(
            c.as_mut_ptr(),
            m.as_ptr(),
            m.len() as u64,
            n.0.as_ptr(),
            pk.0.as_ptr(),
            sk.0.as_ptr(),
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
        pub fn crypto_box_easy(
            c: *mut libc::c_uchar,
            m: *const libc::c_uchar,
            mlen: libc::c_ulonglong,
            n: *const libc::c_uchar,
            pk: *const libc::c_uchar,
            sk: *const libc::c_uchar,
        ) -> libc::c_int;
    }
}

#[cfg(test)]
mod tests {
    use sodiumoxide::crypto::box_::{gen_keypair, gen_nonce};
    use sodiumoxide::randombytes::randombytes;

    #[test]
    fn round_trip() {
        let m = randombytes(42);
        let (pk1, sk1) = gen_keypair();
        let (pk2, sk2) = gen_keypair();
        let n = gen_nonce();

        // encrypt with original and patched function
        let c_original = sodiumoxide::crypto::box_::seal(&m, &n, &pk1, &sk2);
        let c_patched =
            crate::crypto::sodiumoxide_patches::crypto_box::seal(&m, &n, &pk1, &sk2).unwrap();

        // ciphertexts match
        assert_eq!(c_original, c_patched);

        // decrypted plaintexts match initial message
        let m_original = sodiumoxide::crypto::box_::open(&c_original, &n, &pk2, &sk1);
        assert_eq!(m_original, Ok(m.clone()));

        let m_patched = sodiumoxide::crypto::box_::open(&c_patched, &n, &pk2, &sk1);
        assert_eq!(m_patched, Ok(m.clone()));
    }
}
