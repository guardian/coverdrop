package com.theguardian.coverdrop.core.persistence

import androidx.test.platform.app.InstrumentationRegistry
import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.crypto.COVERDROP_PRIVATE_SENDING_QUEUE_ITEM_SIZE
import com.theguardian.coverdrop.core.crypto.CoverDropPrivateSendingQueue
import com.theguardian.coverdrop.core.crypto.PrivateSendingQueueItem
import com.theguardian.coverdrop.core.utils.DefaultClock
import com.theguardian.coverdrop.core.utils.nextByteArray
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertThrows
import org.junit.Before
import org.junit.Test
import java.security.SecureRandom
import kotlin.experimental.xor

class CoverDropFileManagerTest {

    private val context = InstrumentationRegistry.getInstrumentation().targetContext
    private val instance = CoverDropFileManager(context, DefaultClock(), CoverDropNamespace.TEST)

    @Before
    fun setup() {
        instance.getNamespacedBaseDir().deleteRecursively()
        instance.initialize()
    }

    @Test
    fun testWhenWriteFile_thenReadsBackCorrectly_caseEmpty() {
        val file = CoverDropFiles.STATUS_EVENT_V1
        val bytes = ByteArray(0)

        instance.write(file, bytes)
        val retrieved = instance.read(file)
        assertThat(retrieved).isEqualTo(bytes)
    }

    @Test
    fun testWhenWriteFile_thenReadsBackCorrectly_caseLarge() {
        val file = CoverDropFiles.STATUS_EVENT_V1
        val bytes = ByteArray(10 * 1024 * 1024) // 10 MiB

        instance.write(file, bytes)
        val retrieved = instance.read(file)
        assertThat(retrieved).isEqualTo(bytes)
    }

    @Test
    fun whenWriteFile_andReadAndWrite_thenOk() {
        val file = CoverDropFiles.STATUS_EVENT_V1
        val bytes = ByteArray(10 * 1024) // 10 KiB
        instance.write(file, bytes)

        // sanity check: do a no-op rewrite
        val path = instance.path(file)
        path.writeBytes(path.readBytes())
        val retrieved = instance.read(file)
        assertThat(retrieved).isEqualTo(bytes)
    }

    @Test
    fun whenWriteFile_andModifyOneByte_thenThrows() {
        val file = CoverDropFiles.STATUS_EVENT_V1
        val bytes = ByteArray(10 * 1024) // 10 KiB
        instance.write(file, bytes)

        // case 1: we modify the first byte
        val path = instance.path(file)
        path.writeBytes(path.readBytes().mapIndexed { index, byte ->
            if (index == 0) byte xor 0x01 else byte
        }.toByteArray())
        assertThrows(ChecksumMismatchException::class.java) { instance.read(file) }
    }

    @Test
    fun whenWriteFile_andModifyByTruncatingLastByte_thenThrows() {
        val file = CoverDropFiles.STATUS_EVENT_V1
        val bytes = ByteArray(10 * 1024) // 10 KiB
        instance.write(file, bytes)

        // case 2: we truncate the last byte
        val path = instance.path(file)
        path.writeBytes(path.readBytes().dropLast(1).toByteArray())
        assertThrows(ChecksumMismatchException::class.java) { instance.read(file) }
    }

    @Test
    fun whenReadingAnAllEmptyFile_thenThrows() {
        val file = CoverDropFiles.STATUS_EVENT_V1
        val path = instance.path(file)
        path.writeBytes(ByteArray(0))
        assertThrows(ChecksumMismatchException::class.java) { instance.read(file) }
    }

    @Test
    fun testMigration_issue2114_whenOldPrivateSendingQueueExists_thenRemoved() {
        val file = DeprecatedCoverDropFiles.PRIVATE_SENDING_QUEUE_V0
        val path = instance.path(file)
        path.writeBytes(ByteArray(0))
        assertThat(path.exists()).isTrue()
        instance.ensureMigrations()
        assertThat(path.exists()).isFalse()
    }

    @Test
    fun testMigration_issue2349_whenOldJsonFilesExist_thenRemoved() {
        val files = listOf(
            DeprecatedCoverDropFiles.STATUS_EVENT,
            DeprecatedCoverDropFiles.DEAD_DROPS,
            DeprecatedCoverDropFiles.PUBLISHED_KEYS,
        )
        files.forEach { file ->
            val path = instance.path(file)
            path.writeBytes(ByteArray(0))
            assertThat(path.exists()).isTrue()
        }
        instance.ensureMigrations()
        files.forEach { file ->
            val path = instance.path(file)
            assertThat(path.exists()).isFalse()
        }
    }

    private fun createCoverItem() = PrivateSendingQueueItem(
        bytes = SecureRandom().nextByteArray(COVERDROP_PRIVATE_SENDING_QUEUE_ITEM_SIZE)
    )

    @Test
    fun testMigration_issue2349_whenOldPsqExists_thenMigrated_caseValid() = runBlocking {
        val psq = CoverDropPrivateSendingQueue.empty(::createCoverItem)
        val oldFile = DeprecatedCoverDropFiles.PRIVATE_SENDING_QUEUE_V1
        val oldPath = instance.path(oldFile)
        oldPath.writeBytes(psq.serialize())

        // check we can read it back
        val retrieved = CoverDropPrivateSendingQueue.fromBytes(oldPath.readBytes())
        assertThat(retrieved).isEqualTo(psq)

        // migrate (this should hit the valid PSQ path)
        instance.ensureMigrations()

        // check the old file is gone
        assertThat(oldPath.exists()).isFalse()

        // check the new file is there and identical
        val newBytes = instance.read(CoverDropFiles.PRIVATE_SENDING_QUEUE_V2)
        val newPsq = CoverDropPrivateSendingQueue.fromBytes(newBytes)
        assertThat(newPsq).isEqualTo(psq)
    }

    @Test
    fun testMigration_issue2349_whenOldPsqExists_thenMigrated_caseCorrupt() = runBlocking {
        val psq = CoverDropPrivateSendingQueue.empty(::createCoverItem)
        val oldFile = DeprecatedCoverDropFiles.PRIVATE_SENDING_QUEUE_V1
        val oldPath = instance.path(oldFile)
        oldPath.writeBytes(psq.serialize())

        // check we can read it back
        val retrieved = CoverDropPrivateSendingQueue.fromBytes(oldPath.readBytes())
        assertThat(retrieved).isEqualTo(psq)

        // truncate the file which causes a deserialization error
        oldPath.writeBytes(oldPath.readBytes().sliceArray(0 until 100))

        // check that reading causes an exception
        assertThrows(Exception::class.java) { CoverDropPrivateSendingQueue.fromBytes(oldPath.readBytes()) }

        // migrate (this should hit the valid PSQ path)
        instance.ensureMigrations()

        // check the old file is gone
        assertThat(oldPath.exists()).isFalse()

        // check the new file is also not being created
        val newPath = instance.path(CoverDropFiles.PRIVATE_SENDING_QUEUE_V2)
        assertThat(newPath.exists()).isFalse()
    }
}
