@testable import CoverDropCore
import Sodium
import XCTest

final class MultiAnonymousBoxTests: XCTestCase {
    func testEncryptDecrypt_whenSameKeys_thenActualMatchesOriginal() throws {
        let recipientKeypair: EncryptionKeypair<CoverNodeMessaging> = try EncryptionKeypair<CoverNodeMessaging>
            .generateEncryptionKeypair()
        let originalMessage = "안녕하세요"

        guard let encrypted: MultiAnonymousBox<String> = try? MultiAnonymousBox<String>.encrypt(
            recipientPks: [recipientKeypair.publicKey],
            data: originalMessage
        ) else {
            XCTFail("Failed to encrypt anonymous box")
            return
        }

        guard let decrypted = try? MultiAnonymousBox<String>.decrypt(
            recipientPk: recipientKeypair.publicKey,
            recipientSk: recipientKeypair.secretKey,
            data: encrypted,
            numRecipients: 1
        ) else {
            XCTFail("Failed to decrypt anonymous box")
            return
        }

        XCTAssertEqual(originalMessage, decrypted)
    }

    func testEncryptDecrypt_whenFlippingBitInCiphertext_thenDecryptFails() throws {
        let recipientKeypair: EncryptionKeypair<CoverNodeMessaging> = try EncryptionKeypair<CoverNodeMessaging>
            .generateEncryptionKeypair()
        let originalMessage = "안녕하세요"

        guard let encrypted: MultiAnonymousBox<String> = try? MultiAnonymousBox<String>.encrypt(
            recipientPks: [recipientKeypair.publicKey],
            data: originalMessage
        ) else {
            XCTFail("Failed to encrypt anonymous box")
            return
        }

        var encryptedVar = encrypted

        encryptedVar.bytes[0] = encryptedVar.bytes[0] ^ 0x01

        // this fails and throws
        XCTAssertThrowsError(try MultiAnonymousBox<String>.decrypt(
            recipientPk: recipientKeypair.publicKey,
            recipientSk: recipientKeypair.secretKey,
            data: encryptedVar,
            numRecipients: 1
        )) { error in
            XCTAssertEqual(error as! MultiAnonymousBoxError, MultiAnonymousBoxError.decryptWithSecretBoxFailed)
        }
    }

    func testEncryptDecrypt_whenMultipleRecipients_thenAllDecryptCorrectly() throws {
        let numRecipients = 2
        let recipientKeypairs: [EncryptionKeypair<CoverNodeMessaging>] = try Array(1 ... numRecipients)
            .compactMap { _ in
                try EncryptionKeypair<CoverNodeMessaging>.generateEncryptionKeypair()
            }
        let originalMessage = "안녕하세요"

        guard let encrypted: MultiAnonymousBox<String> = try? MultiAnonymousBox<String>.encrypt(
            recipientPks: recipientKeypairs.map { $0.publicKey },
            data: originalMessage
        ) else {
            XCTFail("Failed to encrypt anonymous box")
            return
        }

        for recipient in recipientKeypairs {
            let decrypted = try MultiAnonymousBox<String>.decrypt(
                recipientPk: recipient.publicKey,
                recipientSk: recipient.secretKey,
                data: encrypted,
                numRecipients: numRecipients
            )
            XCTAssertEqual(originalMessage, decrypted)
        }
    }
}
