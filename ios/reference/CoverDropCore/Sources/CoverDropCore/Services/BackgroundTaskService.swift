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
    ) async {
        if !hasBeenRegistered {
            Debug.println("Background task not registered yet")
            return
        }

        let request = BGProcessingTaskRequest(identifier: serviceName)
        // We want to make sure we have network connectivity when sending messages
        request.requiresNetworkConnectivity = true
        // but we don't require power
        request.requiresExternalPower = false

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
            // Downcast the parameter to an processing task as this identifier is used for a processing request request.
            Task {
                await BackgroundTaskService.handleAppRefresh(
                    task: task as! BGProcessingTask,
                    config: config
                )
            }
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

    static func handleAppRefresh(
        task: BGProcessingTask,
        config: CoverDropConfig
    ) async {
        Debug.println("Background task run start...")
        // Need to notify the containing app we have started the background process
        NotificationCenter.default
            .post(name: backgroundTaskStartedNotification, object: self)

        do {
            let lib = try await CoverDropService.getLibraryBlocking()

            let result = await BackgroundMessageSendJob.run(
                publicDataRepository: lib.publicDataRepository,
                now: DateFunction.currentTime(),
                numMessagesPerBackgroundRun: config.numMessagesPerBackgroundRun,
                minDurationBetweenBackgroundRunsInSecs: config.minDurationBetweenBackgroundRunsInSecs
            )

            switch result {
            case .success:
                // Wnat to log if we have succeeded or failed
                NotificationCenter.default
                    .post(
                        name: backgroundTaskSuccessNotification,
                        object: self,
                        userInfo: ["message": "Background task completed successfully"]
                    )
                task.setTaskCompleted(success: true)
            case let .failure(reason):
                NotificationCenter.default
                    .post(
                        name: backgroundTaskFailedNotification,
                        object: self,
                        userInfo: ["message": "Background task execution failed: \(reason)"]
                    )
                task.setTaskCompleted(success: false)
            }
        } catch {
            NotificationCenter.default
                .post(name: backgroundTaskFailedNotification, object: self,
                      userInfo: ["message": "Background task failed: \(error)"])
            task.setTaskCompleted(success: false)
        }
        Debug.println("Background task run finished")
    }
}
