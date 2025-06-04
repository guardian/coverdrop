package com.theguardian.coverdrop.core.integrationtests

import android.content.Context
import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.CoverDropPrivateDataRepository
import com.theguardian.coverdrop.core.CoverDropPublicDataRepository
import com.theguardian.coverdrop.core.createLibSodium
import com.theguardian.coverdrop.core.crypto.PassphraseWordList
import com.theguardian.coverdrop.core.encryptedstorage.EncryptedStorageImpl
import com.theguardian.coverdrop.core.encryptedstorage.selectEncryptedStorageConfiguration
import com.theguardian.coverdrop.core.persistence.CoverDropFileManager


internal fun createEncryptedStorageForTest(
    context: Context,
    fileManager: CoverDropFileManager
): EncryptedStorageImpl {
    val encryptedStorageConfiguration = selectEncryptedStorageConfiguration(
        useSecureElement = false,
        isForAutomatedTest = true
    )
    return EncryptedStorageImpl(
        context = context,
        libSodium = createLibSodium(),
        encryptedStorageConfiguration = encryptedStorageConfiguration,
        fileManager = fileManager,
        passphraseWordList = PassphraseWordList.createFromEffWordList(context),
    )
}

internal suspend fun assertSendingQueueRealMessageCount(
    publicDataRepository: CoverDropPublicDataRepository,
    privateDataRepository: CoverDropPrivateDataRepository,
    expected: Int
) {
    val storedMessageHints = privateDataRepository
        .getMailboxContent()!!
        .messageThreads
        .map { thread -> thread.messages.map { message -> message.privateSendingQueueHint } }
        .flatten()
    val hintsInPrivateSendingQueue = publicDataRepository
        .getPrivateSendingQueueHintsInQueue()
        .toSet()

    assertThat(storedMessageHints.intersect(hintsInPrivateSendingQueue)).hasSize(expected)
}
