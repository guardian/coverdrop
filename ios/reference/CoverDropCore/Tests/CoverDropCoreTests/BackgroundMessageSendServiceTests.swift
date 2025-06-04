@testable import CoverDropCore
import XCTest

final class BackgroundMessageSendServiceTests: XCTestCase {
    override func setUp() {
        // remove UserDefaults keys so they do not intefer with future test runs
        BackgroundMessageSendState.clearAllState()
    }

    func testShouldExecute() async throws {
        func runTest(
            lastRun: Date,
            minimumDurationBetweenRuns: TimeInterval,
            resultExpected: Bool
        ) {
            let result = BackgroundMessageSendJob.shouldExecute(
                now: now,
                lastRun: lastRun,
                minimumDurationBetweenRuns: minimumDurationBetweenRuns
            )
            XCTAssert(result == resultExpected)
        }

        // Test run where last run was 40 seconds ago with minimum run of 30 seconds
        let now = DateFunction.currentTime()
        var lastRun = try now.minusSeconds(40)
        var minimumDurationBetweenRuns = TimeInterval(30)

        runTest(
            lastRun: lastRun,
            minimumDurationBetweenRuns: minimumDurationBetweenRuns,
            resultExpected: true
        )

        // Test run where last run was 20 seconds ago with minimum run of 30 seconds,
        // should be false due to duration having not passed yet

        lastRun = try now.minusSeconds(20)
        minimumDurationBetweenRuns = TimeInterval(30)

        runTest(
            lastRun: lastRun,
            minimumDurationBetweenRuns: minimumDurationBetweenRuns,
            resultExpected: false
        )

        // Test run where last run was 40 seconds ago with minimum run of 30 seconds,
        // shouldRetryDueToPreviousFailure is true should be true due to shouldRetryDueToPreviousFailure being true

        lastRun = try now.minusSeconds(40)
        minimumDurationBetweenRuns = TimeInterval(30)
        runTest(
            lastRun: lastRun,
            minimumDurationBetweenRuns: minimumDurationBetweenRuns,
            resultExpected: true
        )

        // Test run where last run was 10 seconds in the future with minimum run of 30 seconds
        // should be true due to shouldRetryDueToPreviousFailure being true

        lastRun = DateFunction.currentTime().addingTimeInterval(TimeInterval(10))
        minimumDurationBetweenRuns = TimeInterval(30)
        runTest(
            lastRun: lastRun,
            minimumDurationBetweenRuns: minimumDurationBetweenRuns,
            resultExpected: true
        )
    }

    func testRunFunction() async throws {
        let context = IntegrationTestScenarioContext(scenario: .minimal)
        let publicDataRepository = try context.getPublicDataRepositoryWithVerifiedKeys()

        let coverMessageFactory = try publicDataRepository.getCoverMessageFactory()
        _ = try await PrivateSendingQueueRepository.shared.loadOrInitialiseQueue(coverMessageFactory)

        let lastRun = Date(timeIntervalSince1970: TimeInterval(0))
        BackgroundMessageSendState.writeBackgroundWorkLastSuccessfulRun(instant: lastRun)

        let numMessagesPerBackgroundRun = 2
        let minDurationBetweenBackgroundRunsInSecs = 60 * 60

        let now = Date(timeIntervalSince1970: TimeInterval(60 * 62))
        // Our first run of the message send service should always be successful as the UserDefaults state is removed
        _ = await BackgroundMessageSendJob.run(
            publicDataRepository: publicDataRepository,
            now: now,
            numMessagesPerBackgroundRun: numMessagesPerBackgroundRun,
            minDurationBetweenBackgroundRunsInSecs: minDurationBetweenBackgroundRunsInSecs
        )

        var result = BackgroundMessageSendState.readBackgroundWorkLastSuccessfulRun()
        XCTAssertEqual(result, now)

        // second run should not alter the last run timestamp as it is within the minimumDurationBetweenRuns
        _ = await BackgroundMessageSendJob.run(
            publicDataRepository: publicDataRepository,
            now: now,
            numMessagesPerBackgroundRun: numMessagesPerBackgroundRun,
            minDurationBetweenBackgroundRunsInSecs: minDurationBetweenBackgroundRunsInSecs
        )

        result = BackgroundMessageSendState.readBackgroundWorkLastSuccessfulRun()
        XCTAssertEqual(result, now)

        let future = now.addingTimeInterval(TimeInterval(61 * 60))

        // third run should  alter the last run timestamp its in the future, even though background tasks are pending
        _ = await BackgroundMessageSendJob.run(
            publicDataRepository: publicDataRepository,
            now: future,
            numMessagesPerBackgroundRun: numMessagesPerBackgroundRun,
            minDurationBetweenBackgroundRunsInSecs: minDurationBetweenBackgroundRunsInSecs
        )
        result = BackgroundMessageSendState.readBackgroundWorkLastSuccessfulRun()

        XCTAssertEqual(result, future)
    }
}
