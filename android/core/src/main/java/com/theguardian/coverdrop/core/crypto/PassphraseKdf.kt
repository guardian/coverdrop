package com.theguardian.coverdrop.core.crypto

import androidx.annotation.VisibleForTesting
import com.goterl.lazysodium.SodiumAndroid
import com.goterl.lazysodium.interfaces.PwHash
import com.lambdapioneer.sloth.utils.secureRandomBytes
import com.sun.jna.NativeLong
import com.theguardian.coverdrop.core.crypto.AesKey.Companion.AES_KEY_LEN_BYTES
import com.theguardian.coverdrop.core.utils.checkLibSodiumSuccess
import com.theguardian.coverdrop.core.utils.toByteArray
import java.util.Arrays

internal const val PASSPHRASE_KDF_SALT_LEN_BYTES: Int = PwHash.SALTBYTES

internal object PassphraseKdf {

    fun deriveKeyFromString(
        libSodium: SodiumAndroid,
        passphrase: CharArray,
        salt: ByteArray,
        keyLengthInBytes: Int,
        params: PassphraseKdfParameters,
    ): ByteArray =
        deriveKey(
            libSodium = libSodium,
            passwordChars = passphrase,
            salt = salt,
            keyLengthInBytes = keyLengthInBytes,
            params = params
        )

    fun deriveAesKeyFromString(
        libSodium: SodiumAndroid,
        passphrase: CharArray,
        salt: ByteArray,
        params: PassphraseKdfParameters,
    ): AesKey =
        AesKey(
            deriveKey(
                libSodium = libSodium,
                passwordChars = passphrase,
                salt = salt,
                keyLengthInBytes = AES_KEY_LEN_BYTES,
                params = params
            )
        )

    fun generateSalt(): ByteArray {
        return secureRandomBytes(PASSPHRASE_KDF_SALT_LEN_BYTES)
    }

    private fun deriveKey(
        libSodium: SodiumAndroid,
        passwordChars: CharArray,
        salt: ByteArray,
        keyLengthInBytes: Int,
        params: PassphraseKdfParameters,
    ): ByteArray {
        require(passwordChars.isNotEmpty())
        require(salt.size == PASSPHRASE_KDF_SALT_LEN_BYTES)
        require(keyLengthInBytes > 0)

        val outputHash = ByteArray(keyLengthInBytes)

        val passwordBytes = passwordChars.toByteArray()
        try {
            val res = libSodium.crypto_pwhash(
                /* outputHash = */ outputHash,
                /* outputHashLen = */ outputHash.size.toLong(),
                /* password = */ passwordBytes,
                /* passwordLen = */ passwordBytes.size.toLong(),
                /* salt = */ salt,
                /* opsLimit = */ params.opsLimit,
                /* memLimit = */ params.memLimit,
                /* alg = */ PwHash.Alg.PWHASH_ALG_ARGON2ID13.value
            )
            checkLibSodiumSuccess(res)
        } finally {
            Arrays.fill(passwordBytes, 0x00)
        }

        return outputHash
    }

}

// See: https://github.com/guardian/coverdrop/issues/329 for discussion of this value
private const val ARGON2ID_MEMLIMIT = 256 * 1024L * 1024L;

internal enum class PassphraseKdfParameters(
    internal val opsLimit: Long,
    internal val memLimit: NativeLong,
) {
    /**
     * Moderate passphrase KDF parameters to be used when there exists another measure to prevent
     * brute-force password guessing.
     */
    INTERACTIVE(
        opsLimit = PwHash.OPSLIMIT_INTERACTIVE,
        memLimit = NativeLong(ARGON2ID_MEMLIMIT),
    ),

    /**
     * High passphrase KDF parameters to be used when the security against brute-force attacks
     * relies solely on the strength of the KDF.
     */
    HIGH(
        opsLimit = PwHash.ARGON2ID_OPSLIMIT_SENSITIVE,
        memLimit = NativeLong(ARGON2ID_MEMLIMIT)
    ),

    /**
     * An extra low passphrase KDF parameters to be used for testing purposes only.
     */
    @VisibleForTesting
    TEST_INSECURE(
        opsLimit = PwHash.ARGON2ID_OPSLIMIT_MIN,
        memLimit = NativeLong(ARGON2ID_MEMLIMIT)
    ),
}
