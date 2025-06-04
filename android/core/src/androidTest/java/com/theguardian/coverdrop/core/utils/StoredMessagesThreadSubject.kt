package com.theguardian.coverdrop.core.utils

import com.google.common.truth.FailureMetadata
import com.google.common.truth.Subject
import com.google.common.truth.Subject.Factory
import com.google.common.truth.Truth
import com.google.common.truth.Truth.assertAbout
import com.theguardian.coverdrop.core.persistence.StoredMessage
import com.theguardian.coverdrop.core.persistence.StoredMessageThread
import com.theguardian.coverdrop.testutils.InstantSubject


/**
 * Custom [StoredMessagesThreadSubject] to compare two [StoredMessagesThread]s.
 */
internal class StoredMessageThreadsSubject(
    metadata: FailureMetadata,
    private val actual: StoredMessageThread?,
) : Subject(metadata, actual) {

    /**
     * Asserts that the actual [StoredMessagesThread] is equal to the expected one. Internally it uses the
     * [InstantSubject] to compare the timestamps.
     */
    fun isEqualTo(expected: StoredMessageThread) {
        Truth.assertThat(actual?.recipientId).isEqualTo(expected.recipientId)
        Truth.assertThat(actual?.messages)
            ?.comparingElementsUsing<StoredMessage, StoredMessage>(StoredMessageSubject.STORED_MESSAGE_SUBJECT_COMPARATOR)
            ?.containsExactlyElementsIn(expected.messages)
    }

    companion object {
        private fun storedMessagesThreads(): Factory<StoredMessageThreadsSubject, StoredMessageThread?> {
            return Factory<StoredMessageThreadsSubject, StoredMessageThread?> { metadata, actual ->
                StoredMessageThreadsSubject(metadata, actual)
            }
        }

        fun assertThat(actual: StoredMessageThread?): StoredMessageThreadsSubject {
            return assertAbout(storedMessagesThreads()).that(actual)
        }
    }
}
