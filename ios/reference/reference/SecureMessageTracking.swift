import CoverDropCore
import Foundation

class SecureMessagingTracking {
    // CoverDropCore library sends notifications when background tasks are
    // scheduled, started, successful or have failed.
    // We observe these changes, and log these events with firebase.

    deinit {
        NotificationCenter.default.removeObserver(self)
    }

    init() {
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(logTrackingEvent(_:)),
            name: BackgroundTaskService.backgroundTaskStartedNotification,
            object: nil
        )
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(logTrackingEvent(_:)),
            name: BackgroundTaskService.backgroundTaskSuccessNotification,
            object: nil
        )
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(logTrackingEvent(_:)),
            name: BackgroundTaskService.backgroundTaskFailedNotification,
            object: nil
        )
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(logTrackingEvent(_:)),
            name: BackgroundTaskService.backgroundTaskScheduledNotification,
            object: nil
        )
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(logTrackingEvent(_:)),
            name: BackgroundTaskService.backgroundTaskRegisteredNotification,
            object: nil
        )
    }

    @objc
    private func logTrackingEvent(_ notification: Notification) {
        let notificationName = notification.name.rawValue
        let userInfoDescription = notification.userInfo?.description ?? "No additional info"
        Debug.println(
            "CoverDrop notification: name=\(notificationName), userInfo=\(userInfoDescription)"
        )
    }
}
