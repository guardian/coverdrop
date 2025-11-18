package com.theguardian.coverdrop.core.persistence

import android.content.Context
import android.util.Log
import androidx.annotation.VisibleForTesting
import com.theguardian.coverdrop.core.crypto.CoverDropPrivateSendingQueue
import com.theguardian.coverdrop.core.utils.IClock
import com.theguardian.coverdrop.core.utils.splitAt
import java.io.File
import java.io.FileNotFoundException
import java.security.MessageDigest

private const val FILE_MANAGER_COVERDROP_DIR = "coverdrop"
private const val FILE_MANAGER_CHECKSUM_DIGEST = "SHA-256"
private const val FILE_MANAGER_CHECKSUM_DIGEST_LEN = 32

internal enum class CoverDropFiles(val filename: String, val isDirectory: Boolean = false) {
    // all json files are cached responses from the API; these typically don't have a migration
    // path, as we can simply re-fetch them
    STATUS_EVENT_V1("status_event_v1.json"),
    DEAD_DROPS_V1("dead_drops_v1.json"),
    PUBLISHED_KEYS_V1("published_keys_v1.json"),

    // the private sending queue is accessed by both the messaging vault and the background service;
    // we have to be careful with migrations here, as we don't want to lose any messages
    PRIVATE_SENDING_QUEUE_V2("private_sending_queue_v2.blob"),

    // encrypted storage (managed by the Sloth library)
    ENCRYPTED_STORAGE_DIRECTORY("encrypted_storage", isDirectory = true),
}

/**
 * We consider checksum mismatches a [FileNotFoundException], because the calling code should
 * treat it as such.
 */
internal class ChecksumMismatchException : FileNotFoundException()

/**
 * A migration function for [DeprecatedCoverDropFiles]. The first argument it the old, deprecated
 * file; the second argument is the namespaced base directory for the new file.
 */
typealias DeprecatedFileMigrationFunction = (File, File) -> Unit

private fun deleteOldFile(oldFile: File, namespacedBaseDir: File) {
    oldFile.delete()
}

/**
 * All deprecated files will be removed during initialization of the [CoverDropFileManager]. If
 * any migration is needed, that needs to happen in there.
 */
internal enum class DeprecatedCoverDropFiles(
    val filename: String,
    val migration: DeprecatedFileMigrationFunction,
) {
    // See: https://github.com/guardian/coverdrop/issues/2114, migration to smaller queue length
    PRIVATE_SENDING_QUEUE_V0(filename = "private_sending_queue.blob", migration = ::deleteOldFile),

    // See: https://github.com/guardian/coverdrop/issues/2349, migration to reliable persistence
    PRIVATE_SENDING_QUEUE_V1(
        filename = "private_sending_queue_v1.blob",
        migration = { oldFile, namespacedBaseDir ->
            val data = oldFile.readBytes()
            val newFile = File(namespacedBaseDir, CoverDropFiles.PRIVATE_SENDING_QUEUE_V2.filename)

            try {
                // if it deserializes without throwing, it's a valid file
                CoverDropPrivateSendingQueue.fromBytes(data)
                newFile.writeWithChecksum(data)
            } catch (e: Exception) {
                // any errors indicate a corrupted file; we are happy with simply deleting it
            } finally {
                oldFile.delete()
            }
        },
    ),

    // See: https://github.com/guardian/coverdrop/issues/2349, migration to reliable persistence
    STATUS_EVENT(filename = "status_event.json", migration = ::deleteOldFile),

    // See: https://github.com/guardian/coverdrop/issues/2349, migration to reliable persistence
    DEAD_DROPS(filename = "dead_drops.json", migration = ::deleteOldFile),

    // See: https://github.com/guardian/coverdrop/issues/2349, migration to reliable persistence
    PUBLISHED_KEYS(filename = "published_keys.json", migration = ::deleteOldFile),
}

internal class CoverDropFileManager(
    context: Context,
    private val clock: IClock,
    namespace: CoverDropNamespace = CoverDropNamespace.LIVE,
) {
    private val coverDropBaseDir = File(context.filesDir, FILE_MANAGER_COVERDROP_DIR)
    private val namespacedBaseDir = File(coverDropBaseDir, namespace.value)

    init {
        initialize()
    }

    /**
     * Ensures that all directories exist
     */
    fun initialize() {
        ensureFolders()
        ensureMigrations()
    }

    private fun ensureFolders() {
        namespacedBaseDir.mkdirs()
        check(namespacedBaseDir.exists())

        // create sub-folders
        for (file in CoverDropFiles.entries) {
            if (file.isDirectory) {
                val path = path(file)
                path.mkdirs()
                check(path.exists())
            }
        }
    }

    @VisibleForTesting(otherwise = VisibleForTesting.PRIVATE)
    internal fun ensureMigrations() {
        // migrate deprecated files if they exist; `.entries` is guaranteed to return items in
        // the same order they are declared in the source code
        val entries = DeprecatedCoverDropFiles.entries
        for (deprecatedFile in entries) {
            val oldFile = path(deprecatedFile)
            if (oldFile.exists()) {
                // This log statement is safe, as it is run on initialization and independent of any
                // interaction with the feature itself.
                Log.d("CoverDropFileManager", "Migrating: $oldFile")
                deprecatedFile.migration(oldFile, namespacedBaseDir)
            }
        }
    }

    /**
     * Returns `true` iff the given [file] exists for this file managerx.
     */
    fun exists(file: CoverDropFiles): Boolean {
        return path(file).exists()
    }

    /**
     * Overwrites the contents of [file] with [data]. Will create the file if it does not exist.
     *
     * Internals: the data is written to a copy-on-write file first and then renamed to the actual
     * file. This ensures that the file is never in an inconsistent state.
     */
    fun write(file: CoverDropFiles, data: ByteArray) {
        require(!file.isDirectory)

        val cowPath = cowPath(file)
        cowPath.writeWithChecksum(data)

        val path = path(file)
        check(cowPath.renameTo(path))
    }

    /**
     * Returns the contents of [file]. Throws [FileNotFoundException] if [file] does not exist.
     *
     * If the file appears corrupted (checksum mismatch), it will throw a [ChecksumMismatchException].
     */
    @Throws(FileNotFoundException::class, ChecksumMismatchException::class)
    fun read(file: CoverDropFiles): ByteArray {
        require(!file.isDirectory)
        return path(file).readWithChecksum()
    }

    /**
     * Updates the last-modified timestamp of the given [file] by reading and rewriting its
     * contents. If the given file is a directory, all its children are updated.
     */
    fun touch(file: CoverDropFiles) {
        val path = path(file)
        if (file.isDirectory) {
            for (it in path.walkBottomUp()) {
                it.setLastModified(clock.now().toEpochMilli())
            }
        } else {
            path.setLastModified(clock.now().toEpochMilli())
        }
    }

    /**
     * Deletes the given [file] if it exists. If the given file is a directory, all its children
     * are deleted.
     */
    fun delete(file: CoverDropFiles) {
        if (!exists(file)) {
            return
        }

        if (file.isDirectory) {
            path(file).deleteRecursively()
        } else {
            path(file).delete()
        }
    }

    /**
     * The file system path for the given [file].
     */
    fun path(file: CoverDropFiles) = File(namespacedBaseDir, file.filename)

    /**
     * The copy-on-write system path for the given [file]. It's the [path] with a `.cow` extension.
     */
    private fun cowPath(file: CoverDropFiles) = File(namespacedBaseDir, "${file.filename}.cow")

    /**
     * The file system path for the given [file].
     */
    @VisibleForTesting(otherwise = VisibleForTesting.PRIVATE)
    internal fun path(file: DeprecatedCoverDropFiles) = File(namespacedBaseDir, file.filename)

    @VisibleForTesting(otherwise = VisibleForTesting.PRIVATE)
    internal fun getNamespacedBaseDir() = namespacedBaseDir
}

/**
 * Write the given [data] to the file with a checksum as a prefix at the beginning.
 */
@VisibleForTesting(otherwise = VisibleForTesting.PRIVATE)
internal fun File.writeWithChecksum(data: ByteArray) {
    val checksum = checksum(data)
    this.writeBytes(checksum)
    this.appendBytes(data)
}

/**
 * Reads the file and returns the data after verifying the checksum.
 */
@Throws(ChecksumMismatchException::class)
@VisibleForTesting(otherwise = VisibleForTesting.PRIVATE)
internal fun File.readWithChecksum(): ByteArray {
    val content = readBytes()
    if (content.size < FILE_MANAGER_CHECKSUM_DIGEST_LEN) {
        throw ChecksumMismatchException()
    }

    val (checksum, data) = content.splitAt(FILE_MANAGER_CHECKSUM_DIGEST_LEN)
    val expectedChecksum = checksum(data)

    if (!expectedChecksum.contentEquals(checksum)) {
        throw ChecksumMismatchException()
    }
    return data
}

/**
 * Returns the checksum of the given [data] using the digest indicated
 * by [FILE_MANAGER_CHECKSUM_DIGEST].
 */
@VisibleForTesting(otherwise = VisibleForTesting.PRIVATE)
internal fun checksum(
    data: ByteArray,
    digest: MessageDigest = MessageDigest.getInstance(FILE_MANAGER_CHECKSUM_DIGEST)
): ByteArray {
    return digest.digest(data)
}
