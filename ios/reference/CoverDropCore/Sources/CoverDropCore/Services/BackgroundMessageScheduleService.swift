import BackgroundTasks
import Foundation

enum BackgroundMessageScheduleService {
    /// This is called when the library is being initialized
    ///
    /// If work is pending ( got from the `BackgroundWorkPending` state ) we run immediatly
    /// If there is no work pending (note this is a rare occurrence,
    /// and the app should usually schedule work when backgrounded)
    /// we will schedule a cleanup background task
    public static func onAppForeground(
        bgTaskScheduler: TaskScheduler = BGTaskScheduler.shared,
        publicDataRepository: PublicDataRepositoryProtocol,
        config: CoverDropConfig
    ) async throws {
        // This is the fallback mechanism if the background task didn't run
        if let workPending = BackgroundMessageSendState.readBackgroundWorkPending(),
           workPending {
            _ = await BackgroundMessageSendJob.run(
                publicDataRepository: publicDataRepository,
                now: DateFunction.currentTime(),
                numMessagesPerBackgroundRun: config.numMessagesPerBackgroundRun,
                minDurationBetweenBackgroundRunsInSecs: config.minDurationBetweenBackgroundRunsInSecs
            )
        }
        // This code always runs and schedules a background task optimistically in the future.
        // The task is likey to be overwritten when we background the app, but set it here just in case
        await BackgroundTaskService.scheduleBackgroundSendJob(
            extraDelaySeconds: extraDelaySeconds,
            bgTaskScheduler: bgTaskScheduler
        )
        BackgroundMessageSendState.writeBackgroundWorkPending(true)
    }

    /// This should be called when the app enters the background from `applicationDidEnterBackground` in app delegate
    /// This will overwrite any previously scheduled background tasks with this most recent one
    public static func onEnterBackground(bgTaskScheduler: TaskScheduler = BGTaskScheduler.shared) async {
        await BackgroundTaskService.scheduleBackgroundSendJob(bgTaskScheduler: bgTaskScheduler)
        BackgroundMessageSendState.writeBackgroundWorkPending(true)
    }
}
