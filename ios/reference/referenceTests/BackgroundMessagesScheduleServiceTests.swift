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
}
