package com.theguardian.coverdrop.core.persistence

import androidx.test.platform.app.InstrumentationRegistry
import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.api.GsonApiJsonAdapter
import com.theguardian.coverdrop.core.crypto.COVERDROP_PRIVATE_SENDING_QUEUE_ITEM_SIZE
import com.theguardian.coverdrop.core.crypto.CoverDropPrivateSendingQueue
import com.theguardian.coverdrop.core.crypto.PrivateSendingQueueItem
import com.theguardian.coverdrop.core.crypto.PrivateSendingQueueSecret
import com.theguardian.coverdrop.core.utils.DefaultClock
import com.theguardian.coverdrop.core.utils.nextByteArray
import com.theguardian.coverdrop.core.utils.padTo
import com.theguardian.coverdrop.testutils.InstantSubject
import com.theguardian.coverdrop.testutils.IntegrationTestVectors
import com.theguardian.coverdrop.testutils.TestScenario
import kotlinx.coroutines.runBlocking
import org.junit.Before
import org.junit.Test
import java.security.SecureRandom
import java.time.Instant

class PublicStorageTest {

    private val context = InstrumentationRegistry.getInstrumentation().targetContext
    val clock = DefaultClock()
    private val instance = PublicStorage(
        context = context,
        clock = clock,
        fileManager = CoverDropFileManager(context, clock, CoverDropNamespace.TEST)
    )

    @Before
    fun setUp() {
        instance.deleteAll()
    }

    @Test
    fun testWriteReadPublishedKeys_whenStored_thenSubsequentReadIsEqual() {
        val json = IntegrationTestVectors(context, TestScenario.Minimal).readJson("published_keys")
        val original = GsonApiJsonAdapter().parsePublishedPublicKeys(json)

        instance.writePublishedKeys(original)
        val retrieved = instance.readPublishedKeys()

        assertThat(retrieved).isEqualTo(original)
    }

    @Test
    fun testWriteReadPublishedDeadDrops_whenStored_thenSubsequentReadIsEqual() {
        val json = IntegrationTestVectors(context, TestScenario.Minimal).readJson("user_dead_drops")
        val original = GsonApiJsonAdapter().parsePublishedDeadDrops(json)

        instance.writeDeadDrops(original)
        val retrieved = instance.readDeadDrops()

        assertThat(retrieved).isEqualTo(original)
    }

    @Test
    fun testWriteReadPublishedKeysUpdate_whenReadWriteRead_thenFirstNullAndAfterwardsSavedValue() {
        assertThat(instance.readPublishedKeysLastUpdate()).isNull()

        val instant = Instant.now()
        instance.writePublishedKeysLastUpdate(instant)

        InstantSubject.assertThat(instance.readPublishedKeysLastUpdate()).isCloseTo(instant)
    }

    @Test
    fun testWriteReadPublishedDeadDropsUpdate_whenReadWriteRead_thenFirstNullAndAfterwardsSavedValue() {
        assertThat(instance.readPublishedDeadDropsLastUpdate()).isNull()

        val instant = Instant.now()
        instance.writePublishedDeadDropsUpdate(instant)

        InstantSubject.assertThat(instance.readPublishedDeadDropsLastUpdate()).isCloseTo(instant)
    }

    @Test
    fun testWriteReadPrivateSendingQueue_whenReadWriteRead_thenFirstNullAndAfterwardsSavedValue(): Unit =
        runBlocking {
            assertThat(instance.readPrivateSendingQueueBytes()).isNull()

            fun createCoverItem() = PrivateSendingQueueItem(
                bytes = SecureRandom().nextByteArray(COVERDROP_PRIVATE_SENDING_QUEUE_ITEM_SIZE)
            )

            val originalQueue = CoverDropPrivateSendingQueue.empty(::createCoverItem)
            val secret = PrivateSendingQueueSecret("secret__secret__".toByteArray())
            val msg = PrivateSendingQueueItem(
                bytes = "msg".toByteArray().padTo(COVERDROP_PRIVATE_SENDING_QUEUE_ITEM_SIZE)
            )
            originalQueue.enqueue(secret, msg)

            instance.writePrivateSendingQueueBytes(originalQueue.serialize())

            val retrievedValue = instance.readPrivateSendingQueueBytes()!!
            val retrievedQueue = CoverDropPrivateSendingQueue.fromBytes(retrievedValue)
            assertThat(retrievedQueue.getRealMessageCount(secret)).isEqualTo(1)
            assertThat(retrievedQueue.dequeue(::createCoverItem)).isEqualTo(msg)
        }

    @Test
    fun testWriteReadPendingBackgroundWork_whenReadWriteRead_thenFirstFalseAndAfterwardsSavedValue() {
        assertThat(instance.readBackgroundWorkPending()).isEqualTo(false)

        instance.writeBackgroundWorkPending(true)
        assertThat(instance.readBackgroundWorkPending()).isEqualTo(true)

        instance.writeBackgroundWorkPending(false)
        assertThat(instance.readBackgroundWorkPending()).isEqualTo(false)
    }

    @Test
    fun testWriteReadBackgroundJobLastRun_whenReadWriteRead_thenFirstNullAndAfterwardsSavedValue() {
        assertThat(instance.readBackgroundJobLastRun()).isNull()

        val instant = Instant.now()
        instance.writeBackgroundJobLastRun(instant)

        InstantSubject.assertThat(instance.readBackgroundJobLastRun()).isCloseTo(instant)
    }
}
