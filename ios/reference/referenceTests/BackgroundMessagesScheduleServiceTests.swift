import BackgroundTasks
import Foundation

@testable import CoverDropCore
import XCTest

// Step 1: Define a protocol for BGTaskScheduler

class MockTaskScheduler: TaskScheduler {
    var isTaskRegistered = false
    var submittedTaskRequests: [BGTaskRequest] = []
    var cancelledTaskIdentifiers: [String] = []

    func register(forTaskWithIdentifier _: String, using _: DispatchQueue?,
                  launchHandler _: @escaping (BGTask) -> Void) -> Bool {
        isTaskRegistered = true
        // Simulate successful registration
        return true
    }

    func submit(_ taskRequest: BGTaskRequest) throws {
        submittedTaskRequests.append(taskRequest)
    }

    func cancel(taskRequestWithIdentifier identifier: String) {
        cancelledTaskIdentifiers.append(identifier)
    }

    func cancelAllTaskRequests() {
        submittedTaskRequests.removeAll()
    }

    func pendingTaskRequests() async -> [BGTaskRequest] {
        return submittedTaskRequests
    }
}

public struct ConfigWithoutBackgroundTask: CoverDropConfig {
    public var envType: EnvType = .prod
    public var withSecureDns: Bool = true

    public var passphraseWordCount = 3

    public let apiBaseUrl = "https://secure-messaging-api.guardianapis.com/v1"
    public let messageBaseUrl = "https://secure-messaging-msg.guardianapis.com"

    public let cacheEnabled = true

    public let maxBackgroundDurationInSeconds = Constants.maxBackgroundDurationInSeconds
    public var minDurationBetweenBackgroundRunsInSecs = 60 * 60
    public var numMessagesPerBackgroundRun = 2
    public var backgroundTaskEnabled = false
}

final class BackgroundMessageScheduleServiceTests: XCTestCase {
    override func setUp() async throws {
        // remove UserDefaults keys so they do not intefer with future test runs
        BackgroundMessageSendState.clearAllState()
        BackgroundTaskService.hasBeenRegistered = false
    }

    func testOnAppStartCalledMultipleTimesWithoutScheduleBackgroundTask() async throws {
        // OnApp start is called multiple times without background being called,
        // This should keep rescheduling background jobs that may never run
        // and background work pending will always be true
        let context = IntegrationTestScenarioContext(scenario: .minimal)
        let publicDataRepository = try context.getPublicDataRepositoryWithVerifiedKeys()
        let taskScheduler = MockTaskScheduler()

        BackgroundTaskService.registerBackgroundSendJob(
            config: context.config,
            bgTaskScheduler: taskScheduler
        )

        var pendingTasks = await taskScheduler.pendingTaskRequests()
        try await BackgroundMessageScheduleService.onAppForeground(
            bgTaskScheduler: taskScheduler,
            publicDataRepository: publicDataRepository,
            config: StaticConfig.devConfig
        )
        pendingTasks = await taskScheduler.pendingTaskRequests()
        XCTAssertFalse(pendingTasks.isEmpty)
        XCTAssertTrue(BackgroundMessageSendState.readBackgroundWorkPending()!)

        try await BackgroundMessageScheduleService.onAppForeground(
            bgTaskScheduler: taskScheduler,
            publicDataRepository: publicDataRepository,
            config: StaticConfig.devConfig
        )
        pendingTasks = await taskScheduler.pendingTaskRequests()
        XCTAssertFalse(pendingTasks.isEmpty)

        XCTAssertTrue(BackgroundMessageSendState.readBackgroundWorkPending()!)
    }

    func testOnAppStartCalledThenScheduleBackgroundTaskIsCalled() async throws {
        // OnApp start is called, then on background is called
        // A background job is first scheduled for 10 mins time and background is false,
        // but the background call then overwrites the pending task, and sets the schedule for exponential time,
        // and updates background work pending to true

        let context = IntegrationTestScenarioContext(scenario: .minimal)
        let publicDataRepository = try context.getPublicDataRepositoryWithVerifiedKeys()
        let taskScheduler = MockTaskScheduler()

        BackgroundTaskService.registerBackgroundSendJob(
            config: context.config,
            bgTaskScheduler: taskScheduler
        )

        var pendingTasks = await taskScheduler.pendingTaskRequests()
        try await BackgroundMessageScheduleService.onAppForeground(
            bgTaskScheduler: taskScheduler,
            publicDataRepository: publicDataRepository,
            config: StaticConfig.devConfig
        )
        pendingTasks = await taskScheduler.pendingTaskRequests()
        XCTAssertFalse(pendingTasks.isEmpty)
        XCTAssertTrue(BackgroundMessageSendState.readBackgroundWorkPending()!)

        await BackgroundMessageScheduleService.onEnterBackground(
            bgTaskScheduler: taskScheduler
        )

        pendingTasks = await taskScheduler.pendingTaskRequests()
        XCTAssertFalse(pendingTasks.isEmpty)
        XCTAssertTrue(BackgroundMessageSendState.readBackgroundWorkPending()!)
    }

    func testWhenScheduleCalledWithoutRegisterThenDoesNotCrash() async throws {
        let context = IntegrationTestScenarioContext(scenario: .minimal)
        let publicDataRepository = try context.getPublicDataRepositoryWithVerifiedKeys()
        let taskScheduler = MockTaskScheduler()

        try await BackgroundMessageScheduleService.onAppForeground(
            bgTaskScheduler: taskScheduler,
            publicDataRepository: publicDataRepository,
            config: StaticConfig.devConfig
        )

        let pendingTasks = await taskScheduler.pendingTaskRequests()
        XCTAssertTrue(pendingTasks.isEmpty)
    }

    func testBackgroundTaskDisabledStillCallsCleanupMessageSendingTask() async throws {
        // On app foreground we will only scheduleBackgroundSendJob if backgroundTaskEnabled is true
        // We do write writeBackgroundWorkPending, so the next time the app is foregrounded, we will
        // run the cleanup job, but a background task will never have been scheduled to run.
        // We check this by making sure no background tasks are pending after foregrounding.

        let context = IntegrationTestScenarioContext(scenario: .minimal)
        let publicDataRepository = try context.getPublicDataRepositoryWithVerifiedKeys()
        let taskScheduler = MockTaskScheduler()

        // Note we are using config with background tasks disabled
        let config = ConfigWithoutBackgroundTask()

        try await BackgroundMessageScheduleService.onAppForeground(
            bgTaskScheduler: taskScheduler,
            publicDataRepository: publicDataRepository,
            config: config
        )

        let pendingTasks = await taskScheduler.pendingTaskRequests()
        XCTAssertTrue(pendingTasks.isEmpty)
        XCTAssertTrue(BackgroundMessageSendState.readBackgroundWorkPending()!)
    }

    func testBackgroundTaskEnabledStillCallsCleanupMessageSendingTaskAndScheduleBackgroundJob() async throws {
        // On app foreground we scheduleBackgroundSendJob as backgroundTaskEnabled is true
        // We check that a background job has been registered and submitted by observing the notification.
        // We then verify this by checking the pendingTaskRequests list is not empty.
        // Sadly we cannot trigger the background task run from tests so we cannot check the task runs succesfully

        let context = IntegrationTestScenarioContext(scenario: .minimal)
        let publicDataRepository = try context.getPublicDataRepositoryWithVerifiedKeys()
        let taskScheduler = MockTaskScheduler()

        // Note we are using regular config with background tasks enabled
        let config = StaticConfig.devConfig

        let registrationExpecation = expectation(
            forNotification: BackgroundTaskService.backgroundTaskRegisteredNotification,
            object: nil,
            handler: nil
        )

        BackgroundTaskService.registerBackgroundSendJob(
            config: context.config,
            bgTaskScheduler: taskScheduler
        )

        await fulfillment(of: [registrationExpecation], timeout: 5)

        let submissionExpecation = expectation(
            forNotification: BackgroundTaskService.backgroundTaskScheduledNotification,
            object: nil,
            handler: nil
        )

        await BackgroundMessageScheduleService.onEnterBackground(
            bgTaskScheduler: taskScheduler
        )

        await fulfillment(of: [submissionExpecation], timeout: 5)

        let pendingTasks = await taskScheduler.pendingTaskRequests()
        XCTAssertFalse(pendingTasks.isEmpty)
        XCTAssertTrue(BackgroundMessageSendState.readBackgroundWorkPending()!)

        try await BackgroundMessageScheduleService.onAppForeground(
            bgTaskScheduler: taskScheduler,
            publicDataRepository: publicDataRepository,
            config: config
        )

        XCTAssertFalse(pendingTasks.isEmpty)
        XCTAssertTrue(BackgroundMessageSendState.readBackgroundWorkPending()!)
    }
}
