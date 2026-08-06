package com.theguardian.coverdrop.core.crypto

import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.generated.MESSAGE_PADDING_LEN
import org.junit.Test
import java.time.ZonedDateTime


private val TIMESTAMP_NOW = ZonedDateTime.parse("2023-03-09T17:00:00Z").toInstant()

class DecryptedDeadDropMessageTest {

    @Test
    fun testParse_whenDeprecatedHandoverFlag_thenParsedAsUnknown() {
        // 0x01 is the deprecated handover type flag; it must not trigger any logic
        val bytes = ByteArray(1 + MESSAGE_PADDING_LEN)
        bytes[0] = 0x01

        val message = DecryptedDeadDropMessage.parse(
            bytes = bytes,
            remoteId = "j1",
            timestamp = TIMESTAMP_NOW,
        )

        assertThat(message).isInstanceOf(DecryptedDeadDropMessage.Unknown::class.java)
    }

    @Test
    fun testParse_whenUnknownFlag_thenParsedAsUnknown() {
        val bytes = ByteArray(1 + MESSAGE_PADDING_LEN)
        bytes[0] = 0x42

        val message = DecryptedDeadDropMessage.parse(
            bytes = bytes,
            remoteId = "j1",
            timestamp = TIMESTAMP_NOW,
        )

        assertThat(message).isInstanceOf(DecryptedDeadDropMessage.Unknown::class.java)
    }
}
