package com.theguardian.coverdrop.core.background

import android.content.Context
import androidx.work.BackoffPolicy
import androidx.work.Constraints
import androidx.work.ExistingWorkPolicy
import androidx.work.NetworkType
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.WorkManager
import com.theguardian.coverdrop.core.ICoverDropLibInternal
import com.theguardian.coverdrop.core.persistence.CoverDropFileManager
import com.theguardian.coverdrop.core.persistence.PublicStorage
import com.theguardian.coverdrop.core.utils.nextDurationFromExponentialDistribution
import java.security.SecureRandom
import java.time.Duration

const val EXPECTED_MEAN_DELAY_MINUTES: Long = 10
const val MIN_DELAY_MINUTES: Long = 5
const val MAX_DELAY_MINUTES: Long = 120
const val EXTRA_DELAY_MINUTES: Long = 10

internal class BackgroundWorkManager(
    private val lib: ICoverDropLibInternal,
    private val context: Context = lib.getApplicationContext(),
    private val workManager: WorkManager = WorkManager.getInstance(context),
) {
    private val clock = lib.getClock()
    private val fileManager = CoverDropFileManager(context, clock)
    private val publicStorage = PublicStorage(context, clock, fileManager)

    /**
     * Called when the app has exited (or is about to exit). This method schedules the
     * [CoverDropBackgroundWorker] for execution for a randomly chosen time within the next 5-120
     * minutes (expected mean: 10 minutes). The method must not perform any expensive I/O
     * operations.
     */
    fun onAppFinished() {
        // enqueue work as a new one-time work request that should run within the next ~10 minutes
        scheduleWork()
        publicStorage.writeBackgroundWorkPending(true)
    }

    /**
     * Called when the app starts. If the [BackgroundWorkManager] detects that a previously
     * scheduled [CoverDropBackgroundWorker] has not been yet executed, it is executed now. It also
     * schedules a "fail-safe" execution in case the [onAppFinished] method will not be called.
     */
    suspend fun onAppStart() {
        val backgroundWorkPending = publicStorage.readBackgroundWorkPending()
        if (backgroundWorkPending) {
            // and cancel anything that might be scheduled
            WorkManager.getInstance(context).cancelUniqueWork(COVERDROP_BACKGROUND_WORKER_NAME)

            // there is pending work which means that a [BackgroundWorker] was scheduled but not yet
            // executed; execute it now
            CoverDropBackgroundJob.run(lib)
        } else {
            // there is no pending work which means that (a) nothing was scheduled or (b) it
            // was already successfully executed
        }

        // we always schedule background work now to increase our chance that at least some is run
        // in case that the `onAppFinished` callback fails; for this we schedule it further
        // into the future to account for normal app usage
        scheduleWork(extraDelay = Duration.ofMinutes(EXTRA_DELAY_MINUTES))
        publicStorage.writeBackgroundWorkPending(true)
    }

    private fun scheduleWork(extraDelay: Duration = Duration.ZERO) {
        val secureRandom = SecureRandom()

        // delay the work by a random amount; we use an exponential distribution to be able to
        // choose a low mean delay while individual long delays are still plausible
        val delay = secureRandom.nextDurationFromExponentialDistribution(
            expectedMeanDuration = Duration.ofMinutes(EXPECTED_MEAN_DELAY_MINUTES),
            atLeastDuration = Duration.ofMinutes(MIN_DELAY_MINUTES),
            atMostDuration = Duration.ofMinutes(MAX_DELAY_MINUTES),
        ) + extraDelay

        // we require some sort of working network connection for our operations; we also do
        // not want to run when the battery is low to avoid inconveniencing the user
        val constraints = Constraints.Builder()
            .setRequiredNetworkType(NetworkType.CONNECTED)
            .setRequiresBatteryNotLow(true)
            .build()

        val workRequest = OneTimeWorkRequestBuilder<CoverDropBackgroundWorker>()
            .setInitialDelay(delay)
            .setBackoffCriteria(BackoffPolicy.EXPONENTIAL, Duration.ofMinutes(10))
            .setConstraints(constraints)
            .addTag(COVERDROP_BACKGROUND_WORKER_NAME)
            .build()

        // if there is already any work pending under our tag, we replace it with the new one
        workManager.beginUniqueWork(
            COVERDROP_BACKGROUND_WORKER_NAME,
            ExistingWorkPolicy.REPLACE,
            workRequest
        ).enqueue()
    }
}
