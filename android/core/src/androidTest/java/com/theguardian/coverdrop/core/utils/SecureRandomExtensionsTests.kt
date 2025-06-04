package com.theguardian.coverdrop.core.utils

import com.google.common.truth.Truth.assertThat
import org.junit.Test
import java.security.SecureRandom
import java.time.Duration


class SecureRandomExtensionsTests {

    @Test
    fun testNextByteArray_whenNonEmptyLength_thenAppearsPlausible() {
        val secureRandom = SecureRandom()
        val l = 100
        val expectedMeanRange = -32..+32  // JVM bytes are signed
        val expectedZerosRange = 0..10

        val arr1 = secureRandom.nextByteArray(l)
        assertThat(arr1).isNotEmpty()
        assertThat(arr1.filter { it == 0.toByte() }.size).isIn(expectedZerosRange)
        assertThat(arr1.sum().toDouble() / l.toDouble()).isIn(expectedMeanRange)

        val arr2 = secureRandom.nextByteArray(l)
        assertThat(arr2).isNotEmpty()
        assertThat(arr2.filter { it == 0.toByte() }.size).isIn(expectedZerosRange)
        assertThat(arr2.sum().toDouble() / l.toDouble()).isIn(expectedMeanRange)

        assertThat(arr1).isNotEqualTo(arr2)
    }

    @Test
    fun testExponentialDuration_whenGivenExpectedMean_thenObservedMeanAndVarianceMatches() {
        val secureRandom = SecureRandom()
        val n = 10_000
        val expectedMeanDuration = Duration.ofMinutes(30)

        val samples = List(n) {
            secureRandom.nextDurationFromExponentialDistribution(expectedMeanDuration = expectedMeanDuration)
        }

        val mean = samples.fold(Duration.ZERO) { a, b -> a + b }.dividedBy(n.toLong())
        val tolerance = Duration.ofMinutes(1)
        assertThat(mean).isAtLeast(expectedMeanDuration - tolerance)
        assertThat(mean).isAtMost(expectedMeanDuration + tolerance)

        val variance = samples.map { (it - mean).toMinutes() }.sumOf { it * it }.toDouble() / n
        val expectedVarianceMinutes = 1.0 / ((1.0 / 30.0) * (1.0 / 30.0))
        assertThat(variance).isAtLeast(0.8 * expectedVarianceMinutes)
        assertThat(variance).isAtMost(1.2 * expectedVarianceMinutes)
    }

    @Test
    fun testExponentialDuration_whenGivenBounds_thenResultsAlwaysWithin() {
        val secureRandom = SecureRandom()
        val n = 100
        val expectedMeanDuration = Duration.ofMinutes(30)
        val lowerBound = Duration.ofMinutes(40)
        val upperBound = Duration.ofMinutes(45)

        val samples = List(n) {
            secureRandom.nextDurationFromExponentialDistribution(
                expectedMeanDuration = expectedMeanDuration,
                atLeastDuration = lowerBound,
                atMostDuration = upperBound
            )
        }

        samples.forEach {
            assertThat(it).isAtLeast(lowerBound)
            assertThat(it).isAtMost(upperBound)
        }
    }
}

