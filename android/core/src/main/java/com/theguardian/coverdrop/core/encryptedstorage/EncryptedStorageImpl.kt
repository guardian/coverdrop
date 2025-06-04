package com.theguardian.coverdrop.core.encryptedstorage

import android.annotation.SuppressLint
import android.content.Context
import com.goterl.lazysodium.SodiumAndroid
import com.lambdapioneer.sloth.SlothLib
import com.theguardian.coverdrop.core.crypto.Passphrase
import com.theguardian.coverdrop.core.crypto.PassphraseWordList
import com.theguardian.coverdrop.core.persistence.CoverDropFileManager
import com.theguardian.coverdrop.core.persistence.CoverDropNamespace
import com.theguardian.coverdrop.core.persistence.MailboxContent

// The `SuppressLint` is safe here. SlothLib requires at least Android P and we only use it if the
// `SecureElementAvailabilityCache.internalCheckIsAvailable` returns true which internally checks
// for Android P.
@SuppressLint("NewApi")
internal class EncryptedStorageImpl(
    context: Context,
    libSodium: SodiumAndroid,
    fileManager: CoverDropFileManager,
    encryptedStorageConfiguration: EncryptedStorageConfiguration,
    passphraseWordList: PassphraseWordList,
    namespace: CoverDropNamespace = CoverDropNamespace.LIVE,
) : IEncryptedStorage(encryptedStorageConfiguration, passphraseWordList) {

    private val delegate: IEncryptedStorage

    init {
        delegate = when (encryptedStorageConfiguration) {
            is EncryptedStorageConfiguration.SecureElement -> {
                val slothLib = SlothLib(
                    pwHash = Argon2PwHashBinding(
                        libSodium = libSodium,
                        params = encryptedStorageConfiguration.secureElementParameters
                    )
                )
                slothLib.init(context)

                EncryptedStorageWithSecureElement(
                    context = context,
                    slothLib = slothLib,
                    fileManager = fileManager,
                    namespace = namespace,
                    encryptedStorageConfiguration = encryptedStorageConfiguration,
                    passphraseWordList = passphraseWordList,
                    libSodium = libSodium,
                )
            }

            is EncryptedStorageConfiguration.PasswordOnly -> {
                EncryptedStorageWithPassword(
                    libSodium = libSodium,
                    fileManager = fileManager,
                    encryptedStorageConfiguration = encryptedStorageConfiguration,
                    passphraseWordList = passphraseWordList
                )
            }
        }
    }

    override fun onAppStart() =
        delegate.onAppStart()

    override fun createOrResetStorage(passphrase: Passphrase) =
        delegate.createOrResetStorage(passphrase)

    override fun unlockSession(passphrase: Passphrase) =
        delegate.unlockSession(passphrase)

    override fun loadFromStorage(activeSession: IEncryptedStorageSession) =
        delegate.loadFromStorage(activeSession)

    override fun saveToStorage(
        activeSession: IEncryptedStorageSession,
        newMailboxContent: MailboxContent
    ) = delegate.saveToStorage(activeSession, newMailboxContent)
}
