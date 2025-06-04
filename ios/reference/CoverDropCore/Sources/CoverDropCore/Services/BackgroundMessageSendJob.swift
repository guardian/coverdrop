import BackgroundTasks
import Foundation

/// The background message send service is responsible for dequeing and sending messages to the messaging api
/// As this is triggered by `didEnterForeground` events we don't want to send messages too frequently in the
/// case of high usage.
/// This is done by supporting the  following scenarios:
/// If less than `minDurationBetweenBackgroundRunsInSecs` had passed since the last run,
/// we will not send messages If there were any previous failures of the last run or more
/// than `minDurationBetweenBackgroundRunsInSecs` has passed we will attempt to send messages
/// If any of the message sends fail, we will store this fact, and retry on the next run regardless of the time passed
/// since last run.
/// Note that this is also triggered by the `BackgroundTaskService`

enum BackgroundMessageSendServiceError: Error {
    case failedToSendAllMessages
    case skippedRun
    case failedToGetCoverMessage
}

/// Note that testing background tasks isn't very straightforward on iOS
/// This is why we have the `skipBackgroundTaskChecks` to facilitate automated testing
/// To verify this works with background tasks enabled we need to
/// 1. Run the project on a device via xcode
/// 2. Put a breakpoint in `AppInitialView` after `.sheet(isPresented: $showCoverDropView) `
/// This will allow you to enter the debugger when you press the `Open Coverdrop` button
/// 3. Background the app to trigger the job scheduling
/// 4. Foreground the app and the app and press the `Open Coverdrop` button
/// 5. Enter the following code in the debugger. This will start a new task (note that this starts the previously
/// scheduled task from step 3:
/// `e -l objc -- (void)[[BGTaskScheduler sharedScheduler] \`
/// `_simulateLaunchForTaskWithIdentifier:@"com.theguardian.coverdrop.reference.refresh"]`
/// 6. Check the background task logs, you should see an succesful run
/// https://developer.apple.com/documentation/backgroundtasks/starting-and-terminating-tasks-during-development

enum BackgroundMessageSendJob {
    public static func run(
        publicDataRepository: any PublicDataRepositoryProtocol,
        now: Date,
        numMessagesPerBackgroundRun: Int,
        minDurationBetweenBackgroundRunsInSecs: Int
    ) async -> Result<Void, BackgroundMessageSendServiceError> {
        BackgroundMessageSendState.writeBackgroundWorkLastTrigger(instant: now)

        // Rate limiting
        if let lastRun = BackgroundMessageSendState.readBackgroundWorkLastSuccessfulRun() {
            let message = """
            Assessing background tasks with now: \(now) last run: \(lastRun)
            min duration: \(minDurationBetweenBackgroundRunsInSecs)
            """
            Debug.println(message)
            let shouldRun = shouldExecute(
                now: now,
                lastRun: lastRun,
                minimumDurationBetweenRuns: TimeInterval(minDurationBetweenBackgroundRunsInSecs)
            )

            if !shouldRun {
                let message = "Skipped running background tasks"
                Debug.println(message)
                return .failure(.skippedRun)
            }
        }

        // Get our cover message data
        let coverMessageFactoryOpt = try? publicDataRepository.getCoverMessageFactory()
        guard let coverMessageFactory = coverMessageFactoryOpt else {
            return .failure(.failedToGetCoverMessage)
        }

        // We try to send as many messages as we can.
        // Any message that fail to send remain in the queue for reprocessing later
        for _ in 0 ..< numMessagesPerBackgroundRun {
            _ = await publicDataRepository.trySendMessageAndDequeue(coverMessageFactory)
        }

        BackgroundMessageSendState.writeBackgroundWorkLastSuccessfulRun(instant: now)
        BackgroundMessageSendState.writeBackgroundWorkPending(false)
        return .success(())
    }

    public static func shouldExecute(
        now: Date,
        lastRun: Date,
        minimumDurationBetweenRuns: TimeInterval
    ) -> Bool {
        // If the last run appears to be in the future, the device clock has jumped backwards;
        // in this case, we should run (which then updates our timestamp)
        if lastRun > now {
            Debug.println("Run due to lastRun > now ")
            return true
        }

        // If at least the minimum duration has passed, we should run
        if lastRun.addingTimeInterval(minimumDurationBetweenRuns) <= now {
            Debug.println("Run due to lastRun.addingTimeInterval(minimumDurationBetweenRuns) <= now")
            return true
        }

        // Otherwise, we skip
        return false
    }
}
