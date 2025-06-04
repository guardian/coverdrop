package com.theguardian.coverdrop.core.crypto

import androidx.test.platform.app.InstrumentationRegistry
import com.theguardian.coverdrop.core.createLibSodium
import org.junit.Test
import java.security.SignatureException
import kotlin.experimental.xor


class SignatureTest {
    private val libSodium = createLibSodium()
    private val context = InstrumentationRegistry.getInstrumentation().context

    @Test
    fun testSigningVerifying_whenMatchingKeys_thenNoThrow() {
        val keyPair = SigningKeyPair.new(libSodium)
        val message = SignableVector.fromString("hello world")

        val signature = Signature.sign(
            libSodium = libSodium,
            signingSk = keyPair.secretSigningKey,
            data = message,
        )

        Signature.verifyOrThrow(
            libSodium = libSodium,
            signingPk = keyPair.publicSigningKey,
            data = message,
            signature = signature,
        )
    }

    @Test(expected = SignatureException::class)
    fun testSigningVerifying_whenFlippingBit_thenVerifyThrows() {
        val keyPair = SigningKeyPair.new(libSodium)
        val message = SignableVector.fromString("hello world")

        val signature = Signature.sign(
            libSodium = libSodium,
            signingSk = keyPair.secretSigningKey,
            data = message,
        )

        signature.bytes[0] = signature.bytes[0].xor(0x01)

        // this will fail and throws
        Signature.verifyOrThrow(
            libSodium = libSodium,
            signingPk = keyPair.publicSigningKey,
            data = message,
            signature = signature,
        )
    }

    @Test
    fun testSigningVerifying_whenUsingTestVectors_thenNoThrow() {
        val testVectors = CryptoTestVectors(context, "signature")
        val keyPair = SigningKeyPair(
            publicSigningKey = testVectors.readPublicSigningKey("01_pk"),
            secretSigningKey = testVectors.readSecretSigningKey("02_sk"),
        )
        val message = testVectors.readSignableVector("03_message")
        val signature = Signature<SignableVector>(testVectors.readFile("04_signature"))

        Signature.verifyOrThrow(
            libSodium = libSodium,
            signingPk = keyPair.publicSigningKey,
            data = message,
            signature = signature,
        )
    }
}
