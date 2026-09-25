package com.theguardian.coverdrop.core.crypto

import androidx.test.platform.app.InstrumentationRegistry
import com.google.common.truth.Truth.assertThat
import org.junit.Test
import java.time.Instant


class CertificateDataTest {
    private val context = InstrumentationRegistry.getInstrumentation().context

    @Test
    fun testEncryptionKeyWithExpiryCertificateData_whenTestVectors_ThenMatches() {
        val testVectors = CryptoTestVectors(context, "certificate_data")

        val pk = testVectors.readPublicEncryptionKey("01_pk")
        val notValidAfter = testVectors.readInstant("02_not_valid_after")
        val timestamp = testVectors.readTimestampBigEndian("03_timestamp_bytes")
        val certificateData = testVectors.readFile("04_certificate_data")

        // ensure timestamp conversion is compatible
        assertThat(notValidAfter.epochSecond).isEqualTo(timestamp)

        val actual = EncryptionKeyWithExpiryCertificateData.from(pk, notValidAfter)
        assertThat(actual.asBytes()).isEqualTo(certificateData)
    }

    @Test
    fun testIdKeyCertificateData_whenBuilt_thenMatchesLayout() {
        val key = PublicSigningKey(ByteArray(ED25519_PUBLIC_KEY_BYTES) { it.toByte() })
        val notValidAfter = Instant.parse("2025-11-27T16:25:13Z")
        val role = IdKeyRole.JOURNALIST_ID
        val identity = "static_test_journalist"

        val actual = IdKeyCertificateData.from(key, notValidAfter, role, identity).asBytes()

        // See: `id_key_certificate_data.rs`
        val tag = "ID_KEY_CERT_DATA".toByteArray(Charsets.US_ASCII)
        val keyAndExpiry = SigningKeyCertificateData.from(key, notValidAfter).asBytes()
        val roleBytes = role.entityName.toByteArray(Charsets.UTF_8)
        val identityBytes = identity.toByteArray(Charsets.UTF_8)

        assertThat(tag).hasLength(16)
        assertThat(actual).hasLength(16 + 32 + 8 + 4 + roleBytes.size + 4 + identityBytes.size)

        var offset = 0
        assertThat(actual.copyOfRange(offset, offset + 16)).isEqualTo(tag)
        offset += 16
        assertThat(actual.copyOfRange(offset, offset + 40)).isEqualTo(keyAndExpiry)
        offset += 40
        assertThat(actual.copyOfRange(offset, offset + 4)).isEqualTo(byteArrayOf(0, 0, 0, roleBytes.size.toByte()))
        offset += 4
        assertThat(actual.copyOfRange(offset, offset + roleBytes.size)).isEqualTo(roleBytes)
        offset += roleBytes.size
        assertThat(actual.copyOfRange(offset, offset + 4)).isEqualTo(byteArrayOf(0, 0, 0, identityBytes.size.toByte()))
        offset += 4
        assertThat(actual.copyOfRange(offset, actual.size)).isEqualTo(identityBytes)
    }

    @Test
    fun testIdKeyCertificateData_whenDifferentRole_thenDifferentBytes() {
        val key = PublicSigningKey(ByteArray(ED25519_PUBLIC_KEY_BYTES) { it.toByte() })
        val notValidAfter = Instant.parse("2025-11-27T16:25:13Z")
        val identity = "covernode_001"

        val journalist = IdKeyCertificateData.from(key, notValidAfter, IdKeyRole.JOURNALIST_ID, identity)
        val coverNode = IdKeyCertificateData.from(key, notValidAfter, IdKeyRole.COVERNODE_ID, identity)

        assertThat(journalist.asBytes()).isNotEqualTo(coverNode.asBytes())
    }
}
