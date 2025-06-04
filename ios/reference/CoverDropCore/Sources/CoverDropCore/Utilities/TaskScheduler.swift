import BackgroundTasks
import Foundation

// We add this task scheduler trait so that we can mock
// the various BGTask related functions, as these APIs are
// not available in unit tests or in functional tests with
// the emulator.

public protocol TaskScheduler {
    func register(
        forTaskWithIdentifier identifier: String,
        using queue: DispatchQueue?,
        launchHandler: @escaping (BGTask) -> Void
    ) -> Bool
    func submit(_ taskRequest: BGTaskRequest) throws
    func cancel(taskRequestWithIdentifier identifier: String)
    func cancelAllTaskRequests()
    func pendingTaskRequests() async -> [BGTaskRequest]
}

extension BGTaskScheduler: TaskScheduler {
    // No need to implement anything, BGTaskScheduler already provides these methods.
}
