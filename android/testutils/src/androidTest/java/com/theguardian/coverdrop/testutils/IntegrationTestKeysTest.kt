package com.theguardian.coverdrop.testutils

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.google.common.truth.Truth.assertThat
import org.junit.Test
import org.junit.runner.RunWith
import java.time.Instant

@RunWith(AndroidJUnit4::class)
class IntegrationTestKeysTest {

    private val context = InstrumentationRegistry.getInstrumentation().targetContext

    @Test
    fun testReadTimestampNow() {
        val testVectors = IntegrationTestKeys(context)
        val actual = testVectors.getNow()
        assertThat(actual).isGreaterThan(Instant.parse("2000-01-01T00:00:01Z"))
        assertThat(actual).isLessThan(Instant.now())
    }

    @Test
    fun testReadOrganizationKey() {
        val testVectors = IntegrationTestKeys(context)
        val actual = testVectors.getOrganisationKey()
        assertThat(actual.key).isNotEmpty()
        assertThat(actual.certificate).isNotEmpty()

        val now = testVectors.getNow()
        assertThat(actual.notValidAfter).isGreaterThan(now)
    }

    @Test
    fun testReadUserKeyPair() {
        val testVectors = IntegrationTestKeys(context)
        val actual = testVectors.getUserKeyPair()
        assertThat(actual.publicKey.key).isNotEmpty()
        assertThat(actual.secretKey).isNotEmpty()
    }
}
