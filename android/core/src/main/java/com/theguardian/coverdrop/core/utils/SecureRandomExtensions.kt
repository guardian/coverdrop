package com.theguardian.coverdrop.core.utils

import java.security.SecureRandom
import java.time.Duration
import kotlin.math.ln

/**
 * Creates a new [ByteArray] of length [len] that is filled with the next bytes available from this
 * [SecureRandom] source.
 */
internal fun SecureRandom.nextByteArray(len: Int): ByteArray {
    val array = ByteArray(len)
    nextBytes(array)
    return array
}

/**
 * Draws durations from a exponential distribution with the given expected mean (1/lambda).
 */
internal fun SecureRandom.nextDurationFromExponentialDistribution(
    expectedMeanDuration: Duration,
    atLeastDuration: Duration? = null,
    atMostDuration: Duration? = null
): Duration {
    require(expectedMeanDuration.seconds > 0)
    val lambda = 1.0 / expectedMeanDuration.seconds.toDouble()

    // The subtraction `1.0-...` ensures that we do not call `ln` with 0.0
    val x = -1.0 / lambda * ln(1.0 - nextDouble())

    return Duration.ofSeconds(x.toLong()).coerceIn(atLeastDuration, atMostDuration)
}
