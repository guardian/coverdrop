package com.theguardian.coverdrop.core.crypto

import com.theguardian.coverdrop.core.crypto.PrivateSendingQueue.Companion.fromBytes
import com.theguardian.coverdrop.core.generated.USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN
import com.theguardian.coverdrop.core.utils.getByteArray
import com.theguardian.coverdrop.core.utils.nextByteArray
import java.nio.ByteBuffer
import java.security.SecureRandom


internal const val COVERDROP_PRIVATE_SENDING_QUEUE_N = 8
internal const val COVERDROP_PRIVATE_SENDING_QUEUE_ITEM_SIZE =
    USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN

/**
 * The length of the [PrivateSendingQueueSecret] in bytes. This is set to match our security level
 * of 128 bits.
 */
internal const val PRIVATE_SENDING_QUEUE_SECRET_LEN_BYTES = 16 // 128 bit

/**
 * The length of the [PrivateSendingQueueHint] in bytes. This is set to match our security level
 * of 128 bits.
 */
internal const val PRIVATE_SENDING_QUEUE_HINT_SIZE_BYTES = 16 // 128 bit

internal const val CURRENT_ITEMS_INT_BYTES = 4
internal const val ITEM_SIZE_INT_BYTES = 4

/**
 * A [PrivateSendingQueue] that enforces the use of the [COVERDROP_PRIVATE_SENDING_QUEUE_N] and
 * [COVERDROP_PRIVATE_SENDING_QUEUE_ITEM_SIZE] parameters.
 */
internal class CoverDropPrivateSendingQueue private constructor(
    n: Int,
    itemSize: Int,
    initialItemsAndHints: List<Pair<PrivateSendingQueueItem, PrivateSendingQueueHint>> = listOf(),
) : PrivateSendingQueue(
    n = n,
    itemSize = itemSize,
    initialItemsAndHints = initialItemsAndHints
) {

    init {
        require(n == COVERDROP_PRIVATE_SENDING_QUEUE_N)
        require(itemSize == COVERDROP_PRIVATE_SENDING_QUEUE_ITEM_SIZE)
    }

    companion object {
        /**
         * Creates a new [CoverDropPrivateSendingQueue] that is initially filled with `size` cover
         * items.
         */
        internal suspend fun empty(createCoverItem: suspend () -> PrivateSendingQueueItem): CoverDropPrivateSendingQueue {
            // create initial cover items
            val initialItemsAndHints =
                mutableListOf<Pair<PrivateSendingQueueItem, PrivateSendingQueueHint>>()
            val secureRandom = SecureRandom()
            repeat(COVERDROP_PRIVATE_SENDING_QUEUE_N) {
                val itemAndHint = Pair(
                    createCoverItem(),
                    PrivateSendingQueueHint.newFromRandom(secureRandom)
                )
                initialItemsAndHints.add(itemAndHint)
            }

            return CoverDropPrivateSendingQueue(
                n = COVERDROP_PRIVATE_SENDING_QUEUE_N,
                itemSize = COVERDROP_PRIVATE_SENDING_QUEUE_ITEM_SIZE,
                initialItemsAndHints = initialItemsAndHints,
            )
        }

        /**
         * Deserializes a [CoverDropPrivateSendingQueue] from a [ByteArray] that was previously
         * created with [serialize].
         */
        internal fun fromBytes(
            bytes: ByteArray,
        ): CoverDropPrivateSendingQueue = fromBytes(bytes, ::CoverDropPrivateSendingQueue)
    }

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (javaClass != other?.javaClass) return false
        return super.equals(other)
    }

    @Suppress("RedundantOverride")
    override fun hashCode() = super.hashCode()
}

/**
 * A [PrivateSendingQueue] is a data structure to store a mix of real and cover items. An
 * adversary cannot tell from a single snapshot how many real and how many cover items are
 * included. However, a caller that uses a consistent secret will be able to tell how many real
 * messages are currently stored. Also, it ensures that real messages that are enqueued are placed
 * before all cover messages.
 */
internal open class PrivateSendingQueue internal constructor(
    internal val n: Int,
    private val itemSize: Int,
    initialItemsAndHints: List<Pair<PrivateSendingQueueItem, PrivateSendingQueueHint>> = listOf(),
) {
    private val mStorage = mutableListOf<PrivateSendingQueueItem>()
    private val mHints = mutableListOf<PrivateSendingQueueHint>()
    private val mSecureRandom = SecureRandom()

    init {
        // set to initial items (if any); this is only called when initialized through deserialize
        require(initialItemsAndHints.size <= n)
        initialItemsAndHints.forEach { addItemAndHint(it.first, it.second) }

        assertInvariants()
    }

    /**
     * Returns the front-most item of the queue. If there were any real messages in the buffer,
     * they would be at the front and returned before any cover messages. Afterwards the buffer
     * is filled up to `self.size` again.
     *
     * The [createCoverItem] function is called to create a new cover item. The generated cover
     * item must be of length [itemSize].
     */
    suspend fun dequeue(createCoverItem: suspend () -> PrivateSendingQueueItem): PrivateSendingQueueItem {
        // pop both internal queues
        val item = mStorage.removeAt(0)
        mHints.removeAt(0)

        // and fill-up both
        addCoverItemAndHint(createCoverItem)

        assertInvariants()
        return item
    }


    /**
     * Calls [dequeue] for all real messages in the queue. Afterwards the queue only contains
     * cover messages.
     */
    suspend fun clear(
        secret: PrivateSendingQueueSecret,
        createCoverItem: suspend () -> PrivateSendingQueueItem,
    ) {
        val fillLevel = getRealMessageCount(secret)
        for (i in 0 until fillLevel) {
            dequeue(createCoverItem)
        }
    }

    /**
     * Returns the front-most item of the queue without removing it.
     */
    fun peek(): PrivateSendingQueueItem {
        return mStorage.first()
    }

    /**
     * Returns the current number of real messages. This requires that the same [secret] is used
     * for both [getRealMessageCount] and [enqueue].
     */
    fun getRealMessageCount(secret: PrivateSendingQueueSecret): Int {
        var fillLevel = 0
        for (itemAndHint in mStorage.zip(mHints)) {
            val item = itemAndHint.first
            val hint = itemAndHint.second

            val hmac = hmac(secret, item)
            if (hmac != hint) break
            fillLevel += 1
        }
        return fillLevel
    }

    /**
     * Enqueues a new message. If the same [secret] is used for all calls to [enqueue], it
     * guarantees that: (a) the real messages are returned FIFO and (b) they are returned before
     * any cover messages.
     *
     * However, if different [secret] values are used, existing real messages are not detected and
     * will be overwritten.
     *
     * @return the hint for the message which can be stored with the persisted messages to later
     * decide whether a message has been already sent off or is still pending.
     */
    fun enqueue(
        secret: PrivateSendingQueueSecret,
        item: PrivateSendingQueueItem,
    ): PrivateSendingQueueHint {
        val fillLevel = getRealMessageCount(secret)
        check(fillLevel < n) { "The queue is full" }

        val hint = hmac(secret, item)

        mStorage[fillLevel] = item
        mHints[fillLevel] = hint

        assertInvariants()

        return hint
    }

    /**
     * Returns all current hints. This is useful to check them against hints of known messages to
     * decide whether a message has been already sent off or is still pending.
     */
    fun allHints(): List<PrivateSendingQueueHint> {
        return mHints
    }

    internal companion object {

        /**
         * Deserializes a [PrivateSendingQueue] from a [ByteArray] that was previously
         * created with [serialize].
         */
        internal fun <T> fromBytes(
            bytes: ByteArray,
            constructor: (Int, Int, List<Pair<PrivateSendingQueueItem, PrivateSendingQueueHint>>) -> T,
        ): T where T : PrivateSendingQueue {
            val buffer = ByteBuffer.wrap(bytes)

            require(buffer.remaining() >= CURRENT_ITEMS_INT_BYTES + ITEM_SIZE_INT_BYTES)
            val n = buffer.getInt()
            val itemSize = buffer.getInt()

            require(buffer.remaining() >= n * itemSize)
            val storage = List(n) {
                PrivateSendingQueueItem(
                    bytes = buffer.getByteArray(itemSize)
                )
            }

            require(buffer.remaining() == n * PRIVATE_SENDING_QUEUE_HINT_SIZE_BYTES)
            val hints = List(n) {
                PrivateSendingQueueHint(
                    bytes = buffer.getByteArray(PRIVATE_SENDING_QUEUE_HINT_SIZE_BYTES)
                )
            }

            require(buffer.remaining() == 0)

            return constructor(n, itemSize, storage.zip(hints))
        }
    }

    /**
     * Serializes all internal state into a [ByteArray] that can later be used with [fromBytes].
     */
    internal fun serialize(): ByteArray {
        val expectedSize = CURRENT_ITEMS_INT_BYTES +
                ITEM_SIZE_INT_BYTES +
                (n * itemSize) +
                (n * PRIVATE_SENDING_QUEUE_HINT_SIZE_BYTES)
        val buffer = ByteBuffer.allocate(expectedSize)

        buffer.putInt(n)
        buffer.putInt(itemSize)
        mStorage.forEach { buffer.put(it.bytes) }
        mHints.forEach { buffer.put(it.bytes) }

        val array = buffer.array()
        check(array.size == expectedSize)

        return array
    }


    private suspend fun addCoverItemAndHint(createCoverItem: suspend () -> PrivateSendingQueueItem) {
        val coverItem = createCoverItem()
        require(coverItem.bytes.size == itemSize)

        addItemAndHint(
            item = coverItem,
            hint = PrivateSendingQueueHint.newFromRandom(mSecureRandom)
        )
    }

    private fun addItemAndHint(item: PrivateSendingQueueItem, hint: PrivateSendingQueueHint) {
        require(item.bytes.size == itemSize)

        mStorage.add(item)
        mHints.add(hint)
    }

    private fun assertInvariants() {
        check(mStorage.size == n)
        check(mHints.size == n)
        check(mStorage.all { it.bytes.size == itemSize })
        check(mHints.all { it.bytes.size == PRIVATE_SENDING_QUEUE_HINT_SIZE_BYTES })
    }

    /**
     * Computes the HMAC for the given [item] using the given [secret]. The result is truncated
     * to [PRIVATE_SENDING_QUEUE_HINT_SIZE_BYTES] bytes.
     */
    private fun hmac(
        secret: PrivateSendingQueueSecret,
        item: PrivateSendingQueueItem,
    ): PrivateSendingQueueHint {
        val hmac = hmacSha256(secret.bytes, item.bytes)
        return PrivateSendingQueueHint(
            bytes = hmac.sliceArray(0 until PRIVATE_SENDING_QUEUE_HINT_SIZE_BYTES)
        )
    }

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (javaClass != other?.javaClass) return false

        other as PrivateSendingQueue

        if (n != other.n) return false
        if (itemSize != other.itemSize) return false
        if (mStorage != other.mStorage) return false
        if (mHints != other.mHints) return false

        return true
    }

    override fun hashCode(): Int {
        var result = n
        result = 31 * result + itemSize
        result = 31 * result + mStorage.hashCode()
        result = 31 * result + mHints.hashCode()
        return result
    }
}

/**
 * Secret (key) for the [PrivateSendingQueue] that is used to derive the [PrivateSendingQueueHint]
 * for the individual items.
 */
internal data class PrivateSendingQueueSecret(val bytes: ByteArray) {
    init {
        require(bytes.size == PRIVATE_SENDING_QUEUE_SECRET_LEN_BYTES)
    }

    companion object {
        internal fun fromSecureRandom(secureRandom: SecureRandom = SecureRandom()) =
            PrivateSendingQueueSecret(
                bytes = secureRandom.nextByteArray(PRIVATE_SENDING_QUEUE_SECRET_LEN_BYTES)
            )

        internal fun deserialize(bytes: ByteArray): PrivateSendingQueueSecret {
            return PrivateSendingQueueSecret(bytes)
        }
    }

    internal fun serialize(): ByteArray {
        return bytes
    }

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (javaClass != other?.javaClass) return false

        other as PrivateSendingQueueSecret

        return bytes.contentEquals(other.bytes)
    }

    override fun hashCode(): Int {
        return bytes.contentHashCode()
    }
}

/**
 * Hints for the individual [PrivateSendingQueueItem]
 */
internal data class PrivateSendingQueueHint(val bytes: ByteArray) {
    init {
        require(bytes.size == PRIVATE_SENDING_QUEUE_HINT_SIZE_BYTES)
    }

    companion object {
        fun newFromRandom(secureRandom: SecureRandom): PrivateSendingQueueHint {
            return PrivateSendingQueueHint(
                bytes = secureRandom.nextByteArray(PRIVATE_SENDING_QUEUE_HINT_SIZE_BYTES)
            )
        }
    }

    override fun equals(other: Any?): Boolean {
        if (javaClass != other?.javaClass) return false
        return bytes.contentEquals((other as PrivateSendingQueueHint).bytes)
    }

    override fun hashCode(): Int {
        return bytes.contentHashCode()
    }
}

/**
 * Item for the [PrivateSendingQueue]. It does not perform internal validation of the [bytes]
 * payload. Instead this is delayed to the methods of the [PrivateSendingQueue] as the concrete
 * implementation holds the information of the expected item size.
 */
internal data class PrivateSendingQueueItem(val bytes: ByteArray) {

    override fun equals(other: Any?): Boolean {
        if (javaClass != other?.javaClass) return false
        return bytes.contentEquals((other as PrivateSendingQueueItem).bytes)
    }

    override fun hashCode(): Int {
        return bytes.contentHashCode()
    }
}
