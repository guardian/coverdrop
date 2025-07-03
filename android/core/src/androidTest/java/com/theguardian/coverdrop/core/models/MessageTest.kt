package com.theguardian.coverdrop.core.models

import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.utils.DefaultClock
import org.junit.Test
import java.time.Duration

class MessageTest {

    @Test
    fun testGetExpiryState_whenOlderThan14Days_thenExpired() {
        val now = DefaultClock().now()
        val message = Message.sent(
            message = "test",
            timestamp = now - Duration.ofDays(14) - Duration.ofSeconds(1)
        )
        assertThat(message.getExpiryState(now)).isEqualTo(ExpiryState.Expired)
    }

    @Test
    fun testGetExpiryState_when13DaysOld_thenSoonToBeExpired() {
        val now = DefaultClock().now()
        val message = Message.sent(
            message = "test",
            timestamp = now - Duration.ofDays(13)
        )

        val actual = message.getExpiryState(now)
        assertThat(actual).isInstanceOf(ExpiryState.SoonToBeExpired::class.java)

        val soonToBeExpiredActual = actual as ExpiryState.SoonToBeExpired
        assertThat(soonToBeExpiredActual.expiresAt)
            .isEqualTo(now + Duration.ofDays(1))
        assertThat(soonToBeExpiredActual.getTimeRemainingInHours(now))
            .isEqualTo(24)
    }

    @Test
    fun testGetExpiryState_when11DaysOld_thenFresh() {
        val now = DefaultClock().now()
        val message = Message.sent(
            message = "test",
            timestamp = now - Duration.ofDays(11)
        )
        assertThat(message.getExpiryState(now)).isEqualTo(ExpiryState.Fresh)
    }
}
