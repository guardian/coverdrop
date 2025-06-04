@testable import CoverDropCore
import Sodium
import XCTest

final class TwoPartyBoxTests: XCTestCase {
    func testRoundTrip() throws {
        let input = "こんにちは"
        let myKeypair: EncryptionKeypair<User> = try EncryptionKeypair<User>.generateEncryptionKeypair()
        let recipientKeypair: EncryptionKeypair<JournalistMessaging> = try EncryptionKeypair<JournalistMessaging>
            .generateEncryptionKeypair()

        guard let encrypted = try? TwoPartyBox<String>.encrypt(
            recipientPk: recipientKeypair.publicKey,
            senderSk: myKeypair.secretKey,
            data: input
        ),
            let decrypted: String = try? TwoPartyBox<String>.decrypt(
                senderPk: recipientKeypair.publicKey,
                recipientSk: myKeypair.secretKey,
                data: encrypted
            ) else {
            XCTFail("Failed to encrypt/decrypt two party box")
            return
        }

        XCTAssertEqual(input, decrypted)
    }

    func testWorksViaUnchecked() throws {
        let input = "こんにちは"
        let myKeypair: EncryptionKeypair<User> = try EncryptionKeypair<User>.generateEncryptionKeypair()
        let recipientKeypair: EncryptionKeypair<JournalistMessaging> = try EncryptionKeypair<JournalistMessaging>
            .generateEncryptionKeypair()

        guard let encrypted: TwoPartyBox<String> =
            try? TwoPartyBox<String>.encrypt(
                recipientPk: recipientKeypair.publicKey,
                senderSk: myKeypair.secretKey,
                data: input
            ) else {
            XCTFail("Failed to encrypt two party box")
            return
        }

        let rawBytes = encrypted.tagCiphertextAndNonce

        let fromBytes: TwoPartyBox<String> = TwoPartyBox<String>.fromVecUnchecked(bytes: rawBytes)
        let decrypted: String = try TwoPartyBox<String>.decrypt(
            senderPk: recipientKeypair.publicKey,
            recipientSk: myKeypair.secretKey,
            data: fromBytes
        )

        XCTAssertEqual(input, decrypted)
    }

    func testFailsWhenUsingDifferentRecipientKey() throws {
        let input = "こんにちは"

        let myKeypair: EncryptionKeypair<User> = try EncryptionKeypair<User>.generateEncryptionKeypair()

        let intendedRecipientKeypair: EncryptionKeypair<JournalistMessaging> =
            try EncryptionKeypair<JournalistMessaging>.generateEncryptionKeypair()
        let otherRecipientKeypair: EncryptionKeypair<JournalistMessaging> = try EncryptionKeypair<JournalistMessaging>
            .generateEncryptionKeypair()

        let encrypted: TwoPartyBox<String> = try TwoPartyBox<String>.encrypt(
            recipientPk: intendedRecipientKeypair.publicKey,
            senderSk: myKeypair.secretKey,
            data: input
        )

        XCTAssertThrowsError(try TwoPartyBox<String>.decrypt(
            senderPk: otherRecipientKeypair.publicKey,
            recipientSk: myKeypair.secretKey,
            data: encrypted
        )) { error in
            XCTAssertEqual(error as! EncryptionError, EncryptionError.failedToDecrypt)
        }
    }

    func testFailsWhenUsingDifferentSenderKey() throws {
        let input = "こんにちは"

        let myKeypair: EncryptionKeypair<User> = try EncryptionKeypair<User>.generateEncryptionKeypair()
        let otherKeypair: EncryptionKeypair<User> = try EncryptionKeypair<JournalistMessaging>
            .generateEncryptionKeypair()

        let recipientKeypair: EncryptionKeypair<JournalistMessaging> = try EncryptionKeypair<JournalistMessaging>
            .generateEncryptionKeypair()

        let encrypted: TwoPartyBox<String> = try TwoPartyBox<String>.encrypt(
            recipientPk: recipientKeypair.publicKey,
            senderSk: myKeypair.secretKey,
            data: input
        )

        XCTAssertThrowsError(try TwoPartyBox<String>.decrypt(
            senderPk: recipientKeypair.publicKey,
            recipientSk: otherKeypair.secretKey,
            data: encrypted
        )) { error in
            XCTAssertEqual(error as! EncryptionError, EncryptionError.failedToDecrypt)
        }
    }
}
