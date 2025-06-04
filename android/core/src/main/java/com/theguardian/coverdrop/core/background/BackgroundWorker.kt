package com.theguardian.coverdrop.core.background

import android.content.Context
import androidx.annotation.VisibleForTesting
import androidx.work.CoroutineWorker
import androidx.work.ListenableWorker.Result
import androidx.work.WorkerParameters
import com.theguardian.coverdrop.core.CoverDropLib
import com.theguardian.coverdrop.core.ICoverDropLibInternal
import com.theguardian.coverdrop.core.utils.IClock
import java.time.Duration
import java.time.Instant

internal const val COVERDROP_BACKGROUND_WORKER_NAME = "coverdrop-background-worker"

internal class CoverDropBackgroundWorker(
    appContext: Context,
    workerParams: WorkerParameters,
) : CoroutineWorker(appContext, workerParams) {

    /**
     * Executes the [CoverDropBackgroundWorker]. It is the [CoverDropBackgroundJob]'s job to skip
     * actual execution if it is being called too often.
     */
    override suspend fun doWork(): Result {
        return try {
            val lib = getCoverDropLibInternal()
            return CoverDropBackgroundJob.run(lib)
        } catch (ignore: Exception) {
            Result.failure()
        }
    }

    private suspend fun getCoverDropLibInternal(): ICoverDropLibInternal {
        if (testingInternalLibOverride != null) {
            return testingInternalLibOverride!!
        }

        // inject CoverDropLib; this one should be automatically being initialised by the
        // integrating apps [Application] class. However, we need to wait for it.
        val lib = CoverDropLib.getInstance().getInternal()
        lib.waitForInitialization()
        return lib
    }

    companion object {
        // Since we cannot easily pass the CoverDropLib into the worker, we need to provide a way to
        // override it for testing purposes. This is a global override and tests should make sure
        // to reset it after the test has finished.
        private var testingInternalLibOverride: ICoverDropLibInternal? = null

        @VisibleForTesting
        fun overrideInternalLibForTesting(lib: ICoverDropLibInternal?) {
            testingInternalLibOverride = lib
        }
    }
}

internal object CoverDropBackgroundJob {

    suspend fun run(
        lib: ICoverDropLibInternal,
        clock: IClock = lib.getClock(),
        ignoreRateLimit: Boolean = false
    ): Result {
        val configuration = lib.getConfig()
        val publicStorage = lib.getPublicStorage()

        publicStorage.writeBackgroundJobLastTriggered()

        // rate limiting
        val lastRun = publicStorage.readBackgroundJobLastRun()
        if (lastRun != null) {
            val skipRun = !shouldExecute(
                now = clock.now(),
                lastRun = lastRun,
                minimumDurationBetweenRuns = configuration.minimumDurationBetweenBackgroundRuns,
            )
            if (skipRun && !ignoreRateLimit) {
                return Result.retry()
            }
        }

        // send up to `NUM_OF_MESSAGES_PER_BACKGROUND_RUN` messages from the front of the queue; if
        // there is any error in this part, we bail and retry later according to our retry policy
        try {
            val publicDataRepository = lib.getPublicDataRepository()
            repeat(configuration.numMessagesPerBackgroundRun) {
                publicDataRepository.sendNextMessageFromQueue()
            }

            // one successful completion, we can mark this work as done so that it is not retried
            // until we schedule it again
            publicStorage.writeBackgroundWorkPending(false)
            publicStorage.writeBackgroundJobLastRun(clock.now())

            return Result.success()
        } catch (e: Exception) {
            return Result.retry()
        }
    }

    private fun shouldExecute(
        now: Instant,
        lastRun: Instant,
        minimumDurationBetweenRuns: Duration,
    ): Boolean {
        // if the last run appears to be in the future, the device clock has jumped backwards;
        // in this case we should run (which then updates our timestamp)
        if (lastRun > now) return true

        // if at least the minimum duration has passed, we should run
        if (lastRun + minimumDurationBetweenRuns <= now) return true

        // otherwise, we skip
        return false
    }
}
