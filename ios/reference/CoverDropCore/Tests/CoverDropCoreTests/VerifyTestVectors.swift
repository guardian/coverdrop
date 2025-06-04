@testable import CoverDropCore
import Sodium
import XCTest

final class VerifyTestVectors: XCTestCase {
    func testAnonymousBoxTest() throws {
        let recipientPk = try PublicEncryptionKey<JournalistMessaging>(key: FileHelper()
            .bytesFromFile(filePath: "vectors/anonymous_box/01_recipient_pk")!)

        let recipientSk = try SecretEncryptionKey<JournalistMessaging>(key: Box
            .SecretKey(FileHelper().bytesFromFile(filePath: "vectors/anonymous_box/02_recipient_sk")!))

        let message = try FileHelper().bytesFromFile(filePath: "vectors/anonymous_box/03_message")!

        let anonymousBox: AnonymousBox<[UInt8]> = try AnonymousBox(pkTagAndCiphertext: FileHelper()
            .bytesFromFile(filePath: "vectors/anonymous_box/04_anonymous_box")!)

        let actual = try AnonymousBox<[UInt8]>.decrypt(myPk: recipientPk, mySk: recipientSk, data: anonymousBox)

        XCTAssertEqual(message, actual)
    }

    func testTwoPartyBoxTest() throws {
        let senderPk = try PublicEncryptionKey<JournalistMessaging>(key: FileHelper()
            .bytesFromFile(filePath: "vectors/two_party_box/01_sender_pk")!)

        let recipientSk = try SecretEncryptionKey<User>(key: Box
            .SecretKey(FileHelper().bytesFromFile(filePath: "vectors/two_party_box/04_recipient_sk")!))

        let message = try FileHelper().bytesFromFile(filePath: "vectors/two_party_box/05_message")!

        let twoPartyBox: TwoPartyBox<[UInt8]> = try TwoPartyBox(tagCiphertextAndNonce: FileHelper()
            .bytesFromFile(filePath: "vectors/two_party_box/06_two_party_box")!)

        let actual = try TwoPartyBox<[UInt8]>.decrypt(senderPk: senderPk, recipientSk: recipientSk, data: twoPartyBox)

        XCTAssertEqual(message, actual)
    }

    func testMultiAnonymousBoxTest() throws {
        let recipient1Pk = try PublicEncryptionKey<CoverNodeMessaging>(key: FileHelper()
            .bytesFromFile(filePath: "vectors/multi_anonymous_box/01_recipient_1_pk")!)

        let recipient1Sk = try SecretEncryptionKey<CoverNodeMessaging>(key: Box
            .SecretKey(FileHelper().bytesFromFile(filePath: "vectors/multi_anonymous_box/02_recipient_1_sk")!))

        let recipient2Pk = try PublicEncryptionKey<CoverNodeMessaging>(key: FileHelper()
            .bytesFromFile(filePath: "vectors/multi_anonymous_box/03_recipient_2_pk")!)

        let recipient2Sk = try SecretEncryptionKey<CoverNodeMessaging>(key: Box
            .SecretKey(FileHelper().bytesFromFile(filePath: "vectors/multi_anonymous_box/04_recipient_2_sk")!))

        let recipient1KeyPair = EncryptionKeypair(publicKey: recipient1Pk, secretKey: recipient1Sk)
        let recipient2KeyPair = EncryptionKeypair(publicKey: recipient2Pk, secretKey: recipient2Sk)

        let message = try FileHelper().bytesFromFile(filePath: "vectors/multi_anonymous_box/05_message")!

        let multiAnonymousBox: MultiAnonymousBox<[UInt8]> = try MultiAnonymousBox(bytes: FileHelper()
            .bytesFromFile(filePath: "vectors/multi_anonymous_box/06_multi_anonymous_box")!)

        for keys in [recipient1KeyPair, recipient2KeyPair] {
            let actual = try MultiAnonymousBox<[UInt8]>.decrypt(
                recipientPk: keys.publicKey,
                recipientSk: keys.secretKey,
                data: multiAnonymousBox,
                numRecipients: 2
            )

            XCTAssertEqual(message, actual)
        }
    }
}
