package com.theguardian.coverdrop.core.utils

import com.google.common.truth.Correspondence
import com.google.common.truth.FailureMetadata
import com.google.common.truth.Subject
import com.google.common.truth.Truth
import com.google.common.truth.Truth.assertAbout
import com.theguardian.coverdrop.core.persistence.StoredMessage
import com.theguardian.coverdrop.testutils.InstantSubject


/**
 * Custom [StoredMessageSubject] to compare two [StoredMessage]s.
 */
internal class StoredMessageSubject(
    metadata: FailureMetadata,
    private val actual: StoredMessage?,
) : Subject(metadata, actual) {

    /**
     * Asserts that the actual [StoredMessage] is equal to the expected one. Internally it uses the
     * [InstantSubject] to compare the timestamps.
     */
    fun isEqualTo(expected: StoredMessage) {
        Truth.assertThat(actual?.payload).isEqualTo(expected.payload)
        Truth.assertThat(actual?.type).isEqualTo(expected.type)
        Truth.assertThat(actual?.privateSendingQueueHint)
            .isEqualTo(expected.privateSendingQueueHint)
        InstantSubject.assertThat(actual?.timestamp).isCloseTo(expected.timestamp)
    }

    companion object {
        val STORED_MESSAGE_SUBJECT_COMPARATOR: Correspondence<in StoredMessage, in StoredMessage>? =
            Correspondence.from(
                { actual: StoredMessage?, expected: StoredMessage? ->
                    actual?.payload == expected?.payload &&
                            actual?.type == expected?.type &&
                            actual?.privateSendingQueueHint == expected?.privateSendingQueueHint &&
                            InstantSubject.instantsAreClose(
                                a = actual!!.timestamp,
                                b = expected!!.timestamp
                            )
                },
                "is equal to"
            )

        private fun storedMessages(): Factory<StoredMessageSubject, StoredMessage?> {
            return Factory<StoredMessageSubject, StoredMessage?> { metadata, actual ->
                StoredMessageSubject(metadata, actual)
            }
        }

        fun assertThat(actual: StoredMessage?): StoredMessageSubject {
            return assertAbout(storedMessages()).that(actual)
        }
    }
}
