import BackgroundTasks
import Foundation

let expectedMeanDelaySeconds = 10 * 60
let minDelaySeconds = 5 * 60
let maxDelaySeconds = 120 * 60
let extraDelaySeconds = 10 * 60

public enum BackgroundTaskService {
    static var serviceName = "com.theguardian.coverdrop.reference.refresh"
    static var hasBeenRegistered = false
    public static let backgroundTaskStartedNotification = Notification.Name("secureMessaging:backgroundTaskStarted")
    public static let backgroundTaskScheduledNotification = Notification.Name("secureMessaging:backgroundTaskScheduled")
    public static let backgroundTaskRegisteredNotification = Notification
        .Name("secureMessaging:backgroundTaskRegistered")
    public static let backgroundTaskSuccessNotification = Notification.Name("secureMessaging:backgroundTaskSuccess")
    public static let backgroundTaskFailedNotification = Notification.Name("secureMessaging:backgroundTaskFailed")

    static func scheduleBackgroundSendJob(
        extraDelaySeconds: Int = 0,
        bgTaskScheduler: TaskScheduler = BGTaskScheduler.shared
    ) {
        if !hasBeenRegistered {
            Debug.println("Background task not registered yet")
            return
        }

        let request = BGAppRefreshTaskRequest(identifier: serviceName)

        let delay = try? Int(SecureRandomUtils.nextDurationFromExponentialDistribution(
            expectedMeanDuration: Duration.seconds(expectedMeanDelaySeconds),
            atLeastDuration: Duration.seconds(minDelaySeconds),
            atMostDuration: Duration.seconds(maxDelaySeconds)
        ).components.seconds) + extraDelaySeconds
        let timeDelay = TimeInterval(delay ?? expectedMeanDelaySeconds)
        request.earliestBeginDate = Date(timeIntervalSinceNow: timeDelay)

        do {
            try bgTaskScheduler.submit(request)
            NotificationCenter.default
                .post(
                    name: backgroundTaskScheduledNotification,
                    object: self,
                    userInfo: ["message": "Background task scheduled success"]
                )
            Debug.println("Background task scheduled")
        } catch {
            NotificationCenter.default
                .post(
                    name: backgroundTaskScheduledNotification,
                    object: self,
                    userInfo:
                    ["message": "Background task scheduled failed \(error)"]
                )
        }
    }

    static func registerBackgroundSendJob(
        config: CoverDropConfig,
        bgTaskScheduler: TaskScheduler = BGTaskScheduler.shared
    ) {
        _ = bgTaskScheduler.register(forTaskWithIdentifier: serviceName, using: nil) { task in
            guard let task = task as? BGAppRefreshTask else {
                NotificationCenter.default
                    .post(
                        name: backgroundTaskFailedNotification,
                        object: self,
                        userInfo: ["message": "Background task registration failed"]
                    )
                return
            }

            BackgroundTaskService.handleAppRefresh(
                task: task,
                config: config
            )
        }
        hasBeenRegistered = true

        NotificationCenter.default
            .post(
                name: backgroundTaskRegisteredNotification,
                object: self,
                userInfo:
                ["message": "Background task registered"]
            )
        Debug.println("Registered Background task")
    }

    public static func handleAppRefresh(
        task: BGAppRefreshTask,
        config: CoverDropConfig
    ) {
        scheduleBackgroundSendJob()
        Task {
            let result = await handleBackgroundMessasgeSendTask(config: config)
            backgroundTaskCompletionHandler(result: result, task: task)
        }
    }

    public static func backgroundTaskNotificationHandler(result: Result<Void, BackgroundMessageSendServiceError>) {
        switch result {
        case .success:
            NotificationCenter.default
                .post(
                    name: backgroundTaskSuccessNotification,
                    object: self,
                    userInfo: ["message": "Background task completed successfully"]
                )
        case let .failure(reason):
            NotificationCenter.default
                .post(
                    name: backgroundTaskFailedNotification,
                    object: self,
                    userInfo: ["message": "Background task execution failed: \(reason)"]
                )
        }
    }

    public static func manuallyTriggerBackgroundMessageSendTask(config: CoverDropConfig) async
    -> Result<Void, BackgroundMessageSendServiceError> {
        await handleBackgroundMessasgeSendTask(config: config)
    }

    private static func handleBackgroundMessasgeSendTask(config: CoverDropConfig) async
        -> Result<Void, BackgroundMessageSendServiceError> {
        Debug.println("Background task run start...")
        // Need to notify the containing app we have started the background process
        NotificationCenter.default
            .post(name: backgroundTaskStartedNotification, object: self)

        var result: Result<Void, BackgroundMessageSendServiceError> = .failure(
            .failedToGetCoverDropService
        )

        guard let lib = try? await CoverDropService.getLibraryBlocking(config: config) else {
            return result
        }

        result = await BackgroundMessageSendJob.run(
            publicDataRepository: lib.publicDataRepository,
            now: DateFunction.currentTime(),
            numMessagesPerBackgroundRun: config.numMessagesPerBackgroundRun,
            minDurationBetweenBackgroundRunsInSecs: config.minDurationBetweenBackgroundRunsInSecs
        )

        backgroundTaskNotificationHandler(result: result)

        Debug.println("Background task run finished")
        return result
    }

    public static func backgroundTaskCompletionHandler(
        result: Result<Void, BackgroundMessageSendServiceError>,
        task: BGAppRefreshTask
    ) {
        switch result {
        case .success:
            task.setTaskCompleted(success: true)
        case .failure:
            task.setTaskCompleted(success: false)
        }
    }
}
