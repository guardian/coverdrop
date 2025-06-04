package com.theguardian.coverdrop.core

import com.theguardian.coverdrop.core.crypto.CoverDropPrivateSendingQueue
import com.theguardian.coverdrop.core.crypto.PrivateSendingQueueHint
import com.theguardian.coverdrop.core.crypto.PrivateSendingQueueItem
import com.theguardian.coverdrop.core.crypto.PrivateSendingQueueSecret
import com.theguardian.coverdrop.core.persistence.PublicStorage
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.util.concurrent.locks.ReentrantReadWriteLock
import kotlin.concurrent.read
import kotlin.concurrent.write

internal interface ICoverMessageFactory {
    suspend fun createCoverMessage(): PrivateSendingQueueItem
}

/**
 * Wraps a PrivateSendingQueue and takes care of persisting it to [PublicStorage].
 */
internal class PrivateSendingQueueProvider internal constructor(
    private val publicStorage: PublicStorage,
) {
    private lateinit var privateSendingQueue: CoverDropPrivateSendingQueue

    /**
     * We manually lock the PrivateSendingQueue as it potentially can be accessed concurrently from
     * both the UI queuing new messages and the background thread dequeuing them.
     */
    private val lock = ReentrantReadWriteLock()

    /**
     * Loads the existing queue from disk or creates a new one if none exists. In both cases the
     * queue is persisted to disk after loading.
     */
    suspend fun initialize(coverMessageFactory: ICoverMessageFactory) {
        lock.write {
            val privateSendingQueueBytes = publicStorage.readPrivateSendingQueueBytes()
            privateSendingQueue = if (privateSendingQueueBytes == null) {
                CoverDropPrivateSendingQueue.empty(coverMessageFactory::createCoverMessage)
            } else {
                CoverDropPrivateSendingQueue.fromBytes(privateSendingQueueBytes)
            }
            saveToDisk()
        }
    }

    /**
     * See [CoverDropPrivateSendingQueue.enqueue]. Persists the queue to disk after enqueuing.
     */
    suspend fun enqueue(
        secret: PrivateSendingQueueSecret,
        item: PrivateSendingQueueItem,
    ): PrivateSendingQueueHint {
        return lock.write {
            val hint = privateSendingQueue.enqueue(secret, item)
            saveToDisk()
            hint
        }
    }

    /**
     * See [CoverDropPrivateSendingQueue.peek].
     */
    fun peek(): PrivateSendingQueueItem {
        return lock.read { privateSendingQueue.peek() }
    }

    /**
     * See [CoverDropPrivateSendingQueue.allHints].
     */
    fun allHints(): List<PrivateSendingQueueHint> {
        return lock.read { privateSendingQueue.allHints() }
    }

    /**
     * See [CoverDropPrivateSendingQueue.dequeue]. Persists the queue to disk after dequeuing.
     */
    suspend fun dequeue(coverMessageFactory: ICoverMessageFactory): PrivateSendingQueueItem {
        return lock.write {
            val item = privateSendingQueue.dequeue(coverMessageFactory::createCoverMessage)
            saveToDisk()
            item
        }
    }

    /**
     * See [CoverDropPrivateSendingQueue.clear]. Persists the queue to disk after clearing.
     */
    suspend fun clear(
        secret: PrivateSendingQueueSecret,
        coverMessageFactory: ICoverMessageFactory,
    ) {
        lock.write {
            privateSendingQueue.clear(secret, coverMessageFactory::createCoverMessage)
            saveToDisk()
        }
    }

    private suspend fun saveToDisk() {
        withContext(Dispatchers.Default) {
            publicStorage.writePrivateSendingQueueBytes(privateSendingQueue.serialize())
        }
    }
}
