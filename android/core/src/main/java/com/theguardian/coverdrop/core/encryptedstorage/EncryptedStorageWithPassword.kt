package com.theguardian.coverdrop.core.encryptedstorage

import com.goterl.lazysodium.SodiumAndroid
import com.theguardian.coverdrop.core.crypto.AesCipher
import com.theguardian.coverdrop.core.crypto.AesCipherResult
import com.theguardian.coverdrop.core.crypto.AesIv
import com.theguardian.coverdrop.core.crypto.AesKey
import com.theguardian.coverdrop.core.crypto.PASSPHRASE_KDF_SALT_LEN_BYTES
import com.theguardian.coverdrop.core.crypto.Passphrase
import com.theguardian.coverdrop.core.crypto.PassphraseKdf
import com.theguardian.coverdrop.core.crypto.PassphraseWordList
import com.theguardian.coverdrop.core.persistence.CoverDropFileManager
import com.theguardian.coverdrop.core.persistence.CoverDropFiles
import com.theguardian.coverdrop.core.persistence.MailboxContent
import com.theguardian.coverdrop.core.utils.nextByteArray
import java.io.File
import java.security.GeneralSecurityException
import java.security.SecureRandom

/**
 * Implementation of [IEncryptedStorage] that relies solely on the strength of the password and the
 * Argon2 password hashing function. It does not use the Secure Element and should only be chosen
 * on devices that cannot support Sloth.
 */
internal class EncryptedStorageWithPassword(
    private val libSodium: SodiumAndroid,
    private val fileManager: CoverDropFileManager,
    encryptedStorageConfiguration: EncryptedStorageConfiguration.PasswordOnly,
    passphraseWordList: PassphraseWordList
) : IEncryptedStorage(encryptedStorageConfiguration, passphraseWordList) {

    private val passphraseKdfParameters = encryptedStorageConfiguration.passphraseKdfParameters

    override fun onAppStart() {
        // if the storage file exists, recursively update the last-modified stamp of all files
        if (getStorageFile().exists()) {
            fileManager.touch(CoverDropFiles.ENCRYPTED_STORAGE_DIRECTORY)
            return
        }

        // otherwise, there is no file yet; create a new session with a randomly chosen passphrase
        val passphrase = generateNewRandomPassphrase()
        createOrResetStorage(passphrase)
    }

    override fun createOrResetStorage(passphrase: Passphrase): IEncryptedStorageSession {
        ensurePassphraseMeetsRequirements(passphrase)

        // generate new salt
        val saltBytes = SecureRandom().nextByteArray(PASSPHRASE_KDF_SALT_LEN_BYTES)
        getSaltFile().writeBytes(saltBytes)

        // derive K_user from passphrase
        val kUser = PassphraseKdf.deriveAesKeyFromString(
            libSodium = libSodium,
            passphrase = passphrase.getPassphraseString(),
            salt = saltBytes,
            params = passphraseKdfParameters
        )

        val activeSession = EncryptedStoragePasswordBasedSession(saltBytes, kUser)

        saveToStorage(activeSession, MailboxContent.newEmptyMailbox(libSodium))

        return activeSession
    }

    override fun unlockSession(passphrase: Passphrase): IEncryptedStorageSession {
        ensurePassphraseMeetsRequirements(passphrase)

        // read salt
        val saltBytes = getSaltFile().readBytes()

        // derive K_user from passphrase
        val kUser = PassphraseKdf.deriveAesKeyFromString(
            libSodium = libSodium,
            passphrase = passphrase.getPassphraseString(),
            salt = saltBytes,
            params = passphraseKdfParameters
        )

        return EncryptedStoragePasswordBasedSession(saltBytes, kUser)
    }

    override fun loadFromStorage(activeSession: IEncryptedStorageSession): MailboxContent {
        require(activeSession is EncryptedStoragePasswordBasedSession)

        // load state from disk
        val encryptedIvAndPayload = AesCipherResult(
            iv = AesIv(getIvFile().readBytes()),
            encryptedPayload = getStorageFile().readBytes()
        )

        try {
            // decrypt based with the key derived from the user password
            val paddedData = AesCipher.decryptAuthenticatedGcm(
                key = activeSession.kUser,
                aesCipherResult = encryptedIvAndPayload
            )

            return MailboxContent.deserialize(paddedData)
        } catch (e: GeneralSecurityException) {
            throw EncryptedStorageAuthenticationFailed()
        }
    }

    override fun saveToStorage(
        activeSession: IEncryptedStorageSession,
        newMailboxContent: MailboxContent
    ) {
        require(activeSession is EncryptedStoragePasswordBasedSession)

        // serialize all content into a padded byte array
        val paddedData = newMailboxContent.serializeOrTruncate(
            paddedOutputSize = CONTENT_BLOB_LEN_BYTES
        )
        check(paddedData.size == CONTENT_BLOB_LEN_BYTES)

        // encrypt based with the key derived from the user password
        val ivAndEncryptedPayload = AesCipher.encryptAuthenticatedGcm(
            key = activeSession.kUser,
            data = paddedData
        )

        getIvFile().writeBytes(ivAndEncryptedPayload.iv.bytes)
        getStorageFile().writeBytes(ivAndEncryptedPayload.encryptedPayload)
    }

    private fun getSaltFile() = getFile("salt")

    private fun getIvFile() = getFile("iv")

    private fun getStorageFile() = getFile("storage")

    private fun getFile(name: String) = File(
        fileManager.path(CoverDropFiles.ENCRYPTED_STORAGE_DIRECTORY),
        name
    )
}

@Suppress("ArrayInDataClass")
internal data class EncryptedStoragePasswordBasedSession(
    internal val saltBytes: ByteArray,
    internal val kUser: AesKey
) : IEncryptedStorageSession
