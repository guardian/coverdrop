package com.theguardian.coverdrop.core.crypto

import com.goterl.lazysodium.SodiumAndroid
import com.theguardian.coverdrop.core.utils.base64Encode
import com.theguardian.coverdrop.core.utils.checkLibSodiumSuccess

/**
 * A representation that matches the format of the key to be added to the imprint. We compute
 * the SHA-512 digest of the public key, truncate to the first 128 bit, and encode the result
 * in Base64 (blocks of 6 characters)
 */
fun getHumanReadableDigest(sodiumAndroid: SodiumAndroid, key: PublicSigningKey): String {
    val digest = ByteArray(512 / 8)

    val keyBytes = key.bytes
    val res = sodiumAndroid.crypto_hash_sha512(digest, keyBytes, keyBytes.size.toLong())
    checkLibSodiumSuccess(res)

    val base64 = digest.copyOf(128 / 8).base64Encode()
    check(base64.length == 22)

    val chunked = base64.chunked(6).joinToString(" ")
    return chunked
}
