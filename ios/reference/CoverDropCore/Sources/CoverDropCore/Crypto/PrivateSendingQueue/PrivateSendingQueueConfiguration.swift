import Foundation

/// This configuration can be used to start a `PrivateSendingQueue` via a repository.
/// It also holds the default Coverdrop values in the `default` static property.
public struct PrivateSendingQueueConfiguration {
    let totalQueueSize: Int32
    let messageSize: Int32

    /// Default values for Coverdrop.
    public static let `default` = PrivateSendingQueueConfiguration(
        totalQueueSize: 8,
        messageSize: Int32(Constants.userToCovernodeEncryptedMessageLen)
    )
}
