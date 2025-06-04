package com.theguardian.coverdrop.core.crypto

import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.createLibSodium
import com.theguardian.coverdrop.core.persistence.StoredMessage
import com.theguardian.coverdrop.core.persistence.StoredMessageThread
import com.theguardian.coverdrop.core.persistence.randomStoredMessageThread
import org.junit.Test
import java.time.ZonedDateTime
import kotlin.random.Random


private val TIMESTAMP_NOW = ZonedDateTime.parse("2023-03-09T17:00:00Z").toInstant()
private val TIMESTAMP_AFTER = ZonedDateTime.parse("2023-03-09T18:00:00Z").toInstant()

@Suppress("UsePropertyAccessSyntax")
class DeadDropProcessorTest {

    private val libSodium = createLibSodium()
    private val instance = DeadDropProcessor(libSodium)

    @Test
    fun testMerge_whenBothEmpty_thenReturnsEmpty() {
        val existing = emptyList<StoredMessageThread>()
        val new = emptyList<DecryptedDeadDropMessage>()

        val output = instance.mergeExistingThreadsWithNewMessages(existing, new)
        assertThat(output).isEmpty()
    }

    @Test
    fun testMerge_whenExistingEmpty_thenMatchesNewOnes() {
        val random = Random(0)
        val existing = listOf(randomStoredMessageThread(random))
        val new = emptyList<DecryptedDeadDropMessage>()

        val output = instance.mergeExistingThreadsWithNewMessages(existing, new)
        assertThat(output).isEqualTo(existing)
    }

    @Test
    fun testMerge_whenNewOnesEmpty_thenMatchesExistingOnes() {
        val existing = emptyList<StoredMessageThread>()
        val new = listOf(DecryptedDeadDropMessage.Text("j1", TIMESTAMP_NOW, "hello"))

        val output = instance.mergeExistingThreadsWithNewMessages(existing, new)

        val expected = listOf(
            StoredMessageThread(
                recipientId = "j1",
                messages = listOf(
                    StoredMessage.remote(timestamp = TIMESTAMP_NOW, message = "hello"),
                )
            )
        )
        assertThat(output).isEqualTo(expected)

    }

    @Test
    fun testMerge_whenBothNonEmptyAndNewConversations_thenMatchesExpectations() {
        val existing = listOf(
            StoredMessageThread(
                recipientId = "j1",
                messages = listOf(
                    StoredMessage.localForTest(timestamp = TIMESTAMP_NOW, message = "hi"),
                )
            ),
        )
        val new = listOf(
            DecryptedDeadDropMessage.Text("j1", TIMESTAMP_NOW, "hello"), // adds to existing
            DecryptedDeadDropMessage.Text("j2", TIMESTAMP_NOW, "some"), // creates new one
        )

        val output = instance.mergeExistingThreadsWithNewMessages(existing, new)

        val expected = listOf(
            StoredMessageThread(
                recipientId = "j1",
                messages = listOf(
                    StoredMessage.localForTest(timestamp = TIMESTAMP_NOW, message = "hi"),
                    StoredMessage.remote(timestamp = TIMESTAMP_NOW, message = "hello"),
                )
            ),
            StoredMessageThread(
                recipientId = "j2",
                messages = listOf(
                    StoredMessage.remote(timestamp = TIMESTAMP_NOW, message = "some"),
                )
            )
        )
        assertThat(output).isEqualTo(expected)
    }

    @Test
    fun testMerge_whenAddingAnAlreadyExistingMessage_thenNotAdded() {
        val existing = listOf(
            StoredMessageThread(
                recipientId = "j1",
                messages = listOf(
                    StoredMessage.remote(timestamp = TIMESTAMP_NOW, message = "hi"),
                )
            ),
        )

        // adding a new message with the same timestamp
        val new = listOf(
            DecryptedDeadDropMessage.Text("j1", TIMESTAMP_NOW, "hi"), // already exists
        )
        val output = instance.mergeExistingThreadsWithNewMessages(existing, new)

        val expected = listOf(
            StoredMessageThread(
                recipientId = "j1",
                messages = listOf(
                    StoredMessage.remote(timestamp = TIMESTAMP_NOW, message = "hi"),
                )
            ),
        )
        assertThat(output).isEqualTo(expected)
    }

    @Test
    fun testMerge_whenAddingAnAlreadyExistingMessageButWithNewerTimestamp_thenAdded() {
        val existing = listOf(
            StoredMessageThread(
                recipientId = "j1",
                messages = listOf(
                    StoredMessage.remote(timestamp = TIMESTAMP_NOW, message = "hi"),
                )
            ),
        )

        val new = listOf(
            DecryptedDeadDropMessage.Text("j1", TIMESTAMP_AFTER, "hi"), // already exists
        )
        val output = instance.mergeExistingThreadsWithNewMessages(existing, new)

        val expected = listOf(
            StoredMessageThread(
                recipientId = "j1",
                messages = listOf(
                    StoredMessage.remote(timestamp = TIMESTAMP_NOW, message = "hi"),
                    StoredMessage.remote(timestamp = TIMESTAMP_AFTER, message = "hi"),
                )
            ),
        )
        assertThat(output).isEqualTo(expected)
    }
}
