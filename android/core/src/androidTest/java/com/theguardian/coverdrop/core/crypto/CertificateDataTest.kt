package com.theguardian.coverdrop.core.crypto

import androidx.test.platform.app.InstrumentationRegistry
import com.google.common.truth.Truth.assertThat
import org.junit.Test


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
}
