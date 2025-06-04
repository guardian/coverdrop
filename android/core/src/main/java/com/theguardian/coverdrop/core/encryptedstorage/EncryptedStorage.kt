package com.theguardian.coverdrop.core.encryptedstorage

import com.theguardian.coverdrop.core.crypto.Passphrase
import com.theguardian.coverdrop.core.crypto.PassphraseWordList
import com.theguardian.coverdrop.core.persistence.MailboxContent

/**
 * The length of the serialized payload within the encrypted file. Most of this will be compressed
 * so that we can store more text in it.
 */
internal const val CONTENT_BLOB_LEN_BYTES = 512 * 1024 // 512 KiB


/**
 * [IEncryptedStorageSession] is a transparent token that the call site acquires to interact
 * with the storage. It internally contains the derived keys so that subsequent calls (e.g. to
 * store updated content) can be made without having to re-derive the keys which can take a long
 * time.
 */
interface IEncryptedStorageSession


/**
 * Exceptions thrown from the [IEncryptedStorage] implementations.
 */
abstract class EncryptedStorageException(message: String) : SecurityException(message)

/**
 * Thrown when the provided passphrase is not possible. For instance, the word is not part of the
 * word list.
 */
class EncryptedStorageBadPassphraseException :
    EncryptedStorageException("impossible passphrase; maybe a typo?")

/**
 * Thrown when the provided passphrase is possible, but does not unlock the storage. For instance,
 * the storage might not have been used before or the passphrase is wrong.
 */
class EncryptedStorageAuthenticationFailed :
    EncryptedStorageException("decryption failed; wrong passphrase?")


/**
 * Abstract class of the encrypted storage. Either variant is implemented as plausible deniable
 * storage.
 *
 * @see [EncryptedStorageWithPassword]
 * @see [EncryptedStorageWithSecureElement]
 */
internal abstract class IEncryptedStorage(
    protected val encryptedStorageConfiguration: EncryptedStorageConfiguration,
    private val passphraseWordList: PassphraseWordList,
) {

    /**
     * Called on every app start.
     */
    abstract fun onAppStart()

    /**
     * Generates a new passphrase for this [IEncryptedStorage] implementation.
     */
    fun generateNewRandomPassphrase(): Passphrase {
        return passphraseWordList.generatePassphrase(getPassphraseWordCount())
    }

    /**
     * Creates a new active session that does not check the passphrase against the existing storage.
     * It can be used to overwrite the existing storage (if any) with a new passphrase.
     */
    abstract fun createOrResetStorage(passphrase: Passphrase): IEncryptedStorageSession

    /**
     * Unlocks an active session by using the provided passphrase with the existing storage.
     * Successful return from this method does not imply that the passphrase is correct; this is
     * only known when the storage is subsequently accessed using [loadFromStorage].
     */
    abstract fun unlockSession(passphrase: Passphrase): IEncryptedStorageSession

    /**
     * Returns the content of the mailbox, decrypted using the provided [activeSession].
     */
    abstract fun loadFromStorage(activeSession: IEncryptedStorageSession): MailboxContent

    /**
     * Replaces the existing storage (if any) with [newMailboxContent] using the provided
     * [activeSession].
     */
    abstract fun saveToStorage(
        activeSession: IEncryptedStorageSession,
        newMailboxContent: MailboxContent
    )

    /**
     * The number of words expected for passphrases
     */
    fun getPassphraseWordCount() = encryptedStorageConfiguration.wordCount

    protected fun ensurePassphraseMeetsRequirements(passphrase: Passphrase) {
        if (!passphraseWordList.isValidPassphrase(passphrase)) {
            throw EncryptedStorageBadPassphraseException()
        }
    }
}
