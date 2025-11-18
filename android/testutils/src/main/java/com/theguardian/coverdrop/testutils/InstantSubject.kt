package com.theguardian.coverdrop.testutils

import com.google.common.truth.FailureMetadata
import com.google.common.truth.Subject
import com.google.common.truth.Truth.assertAbout
import java.time.Duration
import java.time.Instant


/**
 * Custom [InstantSubject] to compare two [Instant]s with a tolerance. This is required for most
 * Android tests, as there are subtle differences between different the APIs in terms of how
 * precise time is represented.
 */
class InstantSubject(
    metadata: FailureMetadata,
    private val actual: Instant?,
) : Subject(metadata, actual) {

    fun isCloseTo(expected: Instant, tolerance: Duration = Duration.ofMillis(1L)) {
        val isClose = instantsAreClose(a = actual!!, b = expected, tolerance = tolerance)
        if (!isClose) {
            failWithActual("expected to be close to", expected)
        }
    }

    companion object {
        private fun instants(): Factory<InstantSubject, Instant?> {
            return Factory<InstantSubject, Instant?> { metadata, actual ->
                InstantSubject(metadata, actual)
            }
        }

        fun instantsAreClose(
            a: Instant,
            b: Instant,
            tolerance: Duration = Duration.ofMillis(1L),
        ): Boolean {
            return Duration.between(a, b).abs() <= tolerance
        }

        fun assertThat(actual: Instant?): InstantSubject {
            return assertAbout(instants()).that(actual)
        }
    }
}
