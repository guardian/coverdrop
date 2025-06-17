package com.theguardian.coverdrop.core.encryptedstorage

import android.content.Context
import com.goterl.lazysodium.SodiumAndroid
import com.lambdapioneer.sloth.SlothException
import com.lambdapioneer.sloth.SlothLib
import com.lambdapioneer.sloth.crypto.PwHash
import com.lambdapioneer.sloth.impl.HiddenSlothCachedSecrets
import com.lambdapioneer.sloth.impl.HiddenSlothParams
import com.lambdapioneer.sloth.storage.OnDiskStorage
import com.theguardian.coverdrop.core.crypto.Passphrase
import com.theguardian.coverdrop.core.crypto.PassphraseKdf
import com.theguardian.coverdrop.core.crypto.PassphraseWordList
import com.theguardian.coverdrop.core.persistence.CoverDropFileManager
import com.theguardian.coverdrop.core.persistence.CoverDropFiles
import com.theguardian.coverdrop.core.persistence.CoverDropNamespace
import com.theguardian.coverdrop.core.persistence.MailboxContent

internal data class EncryptedStorageSecureElementBasedSession(
    val cachedSecrets: HiddenSlothCachedSecrets,
) : IEncryptedStorageSession

/**
 * Encrypted storage using the Sloth library and its plausibly-deniable encryption scheme
 * HiddenSloth. It uses the Secure Element (SE) to effectively rate-limit password guessing to the
 * maximum throughput of the SE.
 *
 * See: https://github.com/lambdapioneer/sloth
 */
internal class EncryptedStorageWithSecureElement(
    context: Context,
    slothLib: SlothLib,
    fileManager: CoverDropFileManager,
    namespace: CoverDropNamespace,
    encryptedStorageConfiguration: EncryptedStorageConfiguration.SecureElement,
    passphraseWordList: PassphraseWordList,
    private val libSodium: SodiumAndroid,
) : IEncryptedStorage(encryptedStorageConfiguration, passphraseWordList) {


    // we contain HiddenSloth into the encrypted storage folder in which it will create one
    // subfolder under the chosen identity and various files within there
    private val storage = OnDiskStorage(
        context = context,
        customBasePath = fileManager.path(CoverDropFiles.ENCRYPTED_STORAGE_DIRECTORY)
    )

    // HiddenSloth instance where the identifier informs the top-level sub-folder in our storage
    // and the name of the key stored in the Secure Element
    private val hiddenSloth = slothLib.getHiddenSlothInstance(
        identifier = namespace.value,
        storage = storage,
        params = HiddenSlothParams(
            payloadMaxLength = CONTENT_BLOB_LEN_BYTES,
            longSlothParams = encryptedStorageConfiguration.secureElementParameters.slothParameters
        )
    )

    override fun onAppStart() {
        hiddenSloth.onAppStart()
    }

    override fun createOrResetStorage(passphrase: Passphrase): IEncryptedStorageSession {
        ensurePassphraseMeetsRequirements(passphrase)

        val activeSession = EncryptedStorageSecureElementBasedSession(
            hiddenSloth.computeCachedSecrets(
                pw = passphrase.getPassphraseString(),
                authenticateStorage = false
            )
        )

        // Reset storage by storing an empty payload under the new passphrase
        saveToStorage(activeSession, MailboxContent.newEmptyMailbox(libSodium))

        return activeSession
    }

    override fun unlockSession(passphrase: Passphrase): IEncryptedStorageSession {
        ensurePassphraseMeetsRequirements(passphrase)

        try {
            val cachedSecrets = hiddenSloth.computeCachedSecrets(
                pw = passphrase.getPassphraseString(),
                authenticateStorage = true
            )
            return EncryptedStorageSecureElementBasedSession(cachedSecrets)
        } catch (e: SlothException) {
            throw EncryptedStorageAuthenticationFailed()
        }
    }

    override fun loadFromStorage(activeSession: IEncryptedStorageSession): MailboxContent {
        require(activeSession is EncryptedStorageSecureElementBasedSession)

        try {
            val paddedData = hiddenSloth.decryptFromStorageWithCachedSecrets(
                cachedSecrets = activeSession.cachedSecrets,
            )
            return MailboxContent.deserialize(paddedData)
        } catch (e: SlothException) {
            throw EncryptedStorageAuthenticationFailed()
        }
    }

    override fun saveToStorage(
        activeSession: IEncryptedStorageSession,
        newMailboxContent: MailboxContent
    ) {
        require(activeSession is EncryptedStorageSecureElementBasedSession)

        // serialize all content into a padded byte array
        val paddedData = newMailboxContent.serializeOrTruncate(
            paddedOutputSize = CONTENT_BLOB_LEN_BYTES
        )
        check(paddedData.size == CONTENT_BLOB_LEN_BYTES)

        hiddenSloth.encryptToStorageWithCachedSecrets(
            cachedSecrets = activeSession.cachedSecrets,
            data = paddedData
        )
    }
}

/**
 * Implementation for the [PwHash] of the Sloth library to use our existing Argon2 implementation.
 */
internal class Argon2PwHashBinding(
    private val libSodium: SodiumAndroid,
    private val params: SecureElementParameters
) : PwHash {
    override fun createSalt(): ByteArray {
        return PassphraseKdf.generateSalt()
    }

    override fun deriveHash(
        salt: ByteArray,
        password: CharArray,
        outputLengthInBytes: Int
    ): ByteArray {
        return PassphraseKdf.deriveKeyFromString(
            libSodium = libSodium,
            passphrase = password,
            salt = salt,
            keyLengthInBytes = outputLengthInBytes,
            params = params.passphraseKdfParameters
        )
    }
}
