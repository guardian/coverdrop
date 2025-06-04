@testable import CoverDropCore
import Sodium
import XCTest

final class UserToJournalistMessageTests: XCTestCase {
    // TODO: skip until we have a simple way to get the journalist key with the test vector
    func skipped_testRoundTrip() async throws {
        _ = PublicKeysHelper.shared.testKeys

        let encryptedStorage = EncryptedStorage.createForTesting()
        _ = try await encryptedStorage.onAppStart(config: StaticConfig.devConfig)

        guard let recipientEncryptionPublicKey = await PublicKeysHelper.shared.getTestJournalistMessageKey() else {
            XCTFail("Failed to get key")
            return
        }
        let recipientSecretKeyFile = try PublicKeysHelper.shared.getTestJournalistMessageSecretKey()
        let recipientEncryptionSecretKey = SecretEncryptionKey<JournalistMessaging>(key: recipientSecretKeyFile.key)

        let covernodeMessageEncryptionSecretKey = try SecretEncryptionKey<CoverNodeMessaging>(key: PublicKeysHelper
            .shared.getTestCovernodeMessageSecretKey().key)

        let userMessageEncryptionKeyPair: EncryptionKeypair<User> = try EncryptionKeypair<User>
            .generateEncryptionKeypair()

        let message = "This is a test"

        let encrypted = try UserToCoverNodeMessage.createMessage(
            message: message,
            recipientPublicKey: recipientEncryptionPublicKey,
            verifiedPublicKeys: PublicKeysHelper.shared.testKeys,
            userPublicKey: userMessageEncryptionKeyPair.publicKey,
            tag: RecipientTag(tag: [1, 2, 3, 4])
        )

        if encrypted.bytes.count != Constants.userToCovernodeEncryptedMessageLen {
            throw EncryptionError.failedToEncrypt
        }

        let coverNodeKeys: [CoverNodeIdentity: CoverNodeMessagingPublicKey] = PublicKeysHelper.shared.testKeys
            .mostRecentCoverNodeMessagingKeysFromAllHierarchies()
        // lets decrypt - note this code is not in its own function as we
        // don't decrypt the message responses this way, as they are in a different format
        // this is just to confirm we encrypted correctly.
        for coverNodeMessagingKey in coverNodeKeys.values {
            let decryptedOuterMessage: UserToCoverNodeMessageData = try MultiAnonymousBox<UserToCoverNodeMessageData>
                .decrypt(
                    recipientPk: coverNodeMessagingKey.key,
                    recipientSk: covernodeMessageEncryptionSecretKey,
                    data: encrypted,
                    numRecipients: 2
                )

            let ciphertextWithoutRecipientTag = decryptedOuterMessage.userToJournalistMessage

            let decryptedInnerMessage = try AnonymousBox<UserToJournalistMessageData>.decrypt(
                myPk: recipientEncryptionPublicKey.key,
                mySk: recipientEncryptionSecretKey,
                data: ciphertextWithoutRecipientTag
            )

            let plainText = try decryptedInnerMessage.paddedCompressedString.toString()
            XCTAssertEqual(plainText, message)
        }
    }
}
