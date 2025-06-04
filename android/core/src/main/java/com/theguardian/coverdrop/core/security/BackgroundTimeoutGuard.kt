package com.theguardian.coverdrop.core.security

import com.theguardian.coverdrop.core.CoverDropConfiguration
import com.theguardian.coverdrop.core.ICoverDropLib
import kotlinx.coroutines.runBlocking

private const val NS_IN_MS = 1_000_000

interface IBackgroundTimeoutGuard {
    fun onPause()
    fun onResume()
}

class BackgroundTimeoutGuard(
    private val configuration: CoverDropConfiguration,
    private val lib: ICoverDropLib
) : IBackgroundTimeoutGuard {

    // We use a simple `long` value instead of `Instant` here to use the system's monotonic clock.
    // This avoids issues with the real-time clock where time might jump forward/backward resulting
    // in locking too early or locking too late. The value will be `null` while the activity has not
    // been paused yet.
    private var lastOnPauseEventTimestampMs: Long? = null

    override fun onPause() {
        lastOnPauseEventTimestampMs = monotonicTimeMs()
    }

    override fun onResume() {
        if (lastOnPauseEventTimestampMs == null) return

        val timePassedMs = monotonicTimeMs() - lastOnPauseEventTimestampMs!!
        if (timePassedMs > configuration.backgroundTimeoutForAutomaticLogout.toMillis()) {
            runBlocking {
                val privateDataRepository = lib.getPrivateDataRepository()
                privateDataRepository.lock()
            }
        }
    }

    private fun monotonicTimeMs() = System.nanoTime() / NS_IN_MS
}
