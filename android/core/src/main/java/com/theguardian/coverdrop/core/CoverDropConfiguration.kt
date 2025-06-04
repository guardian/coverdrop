package com.theguardian.coverdrop.core

import com.theguardian.coverdrop.core.api.ApiConfiguration
import com.theguardian.coverdrop.core.api.IApiCallProvider
import com.theguardian.coverdrop.core.crypto.PublicSigningKey
import com.theguardian.coverdrop.core.generated.CLIENT_DEAD_DROP_CACHE_TTL_SECONDS
import com.theguardian.coverdrop.core.generated.CLIENT_STATUS_DOWNLOAD_RATE_SECONDS
import com.theguardian.coverdrop.core.utils.DefaultClock
import com.theguardian.coverdrop.core.utils.IClock
import java.time.Duration

data class CoverDropConfiguration(

    /**
     * The configuration of the API including the backend URL.
     */
    internal val apiConfiguration: ApiConfiguration,

    /**
     * The clock to use for all operations that require the current date and time (e.g. certificate
     * validation).
     */
    internal val clock: IClock = DefaultClock(),

    /**
     * Factory method for creating the application's [IApiCallProvider] which is generally a
     * wrapper around its own HTTP client.
     *
     * We use a factory method to allow the application to delay the HTTP client initialization
     * until we need it. Also, if there are cached resources, there might be no need to initialize
     * one at all.
     */
    internal val createApiCallProvider: () -> IApiCallProvider,

    /**
     * A list of [PublicSigningKey] that are trusted as the organization public keys.
     * These should be read from a trustworthy signed source that is not accessible to an active
     * network adversary. For instance, they might be bundled with the signed APK file.
     */
    internal val trustedOrgPks: List<PublicSigningKey>,

    /**
     * The maximum of most-recent dead-drops to keep cached on disk. New dead-drops are downloaded
     * when the library is initialized on app start. This is also when old dead-drops are evicted.
     * It is recommended to keep at least 2 weeks worth of dead-drops in cache. Eviction is
     * performed based on the timestamp of the newest dead-drop, hence ensuring that offsets in the
     * client clock do not cause wrong evictions.
     */
    internal val deadDropCacheTTL: Duration = Duration.ofSeconds(
        CLIENT_DEAD_DROP_CACHE_TTL_SECONDS.toLong()
    ),

    /**
     * The minimum time that needs to pass before new keys or dead-drops are downloaded.
     * Even when setting a small value, the download will only ever execute during library
     * initialization on app start.
     */
    internal val minimumDurationBetweenDefaultDownloads: Duration = Duration.ofSeconds(
        CLIENT_STATUS_DOWNLOAD_RATE_SECONDS.toLong()
    ),

    /**
     * The minimum time that needs to pass before new status updates are downloaded.
     */
    internal val minimumDurationBetweenStatusUpdateDownloads: Duration = Duration.ofSeconds(
        CLIENT_STATUS_DOWNLOAD_RATE_SECONDS.toLong()
    ),

    /**
     * Whether the library should be initialized in test mode. In test mode, the library will
     * use extra short passphrases and use the local test server instead of the production server.
     *
     * This value should always be set to `false` in production.
     */
    val localTestMode: Boolean = false,

    /**
     * Whether to show a warning banner underneath the top app bar that explains that the app is a
     * testing version. Clicking on the banner allows to temporarily disable the warning.
     */
    val showBetaWarningBanner: Boolean = true,

    /**
     * Whether to show debug information on the about screen.
     */
    val showDebugInformation: Boolean = true,

    /**
     * Whether the UI should be disabled. This is helpful for creating screencast and screenshots
     * during development and UX testing.
     *
     * This value should always be set to `false` in production.
     */
    val disableScreenCaptureProtection: Boolean = false,

    /**
     * Minimum time between two successive runs of the background job. If the background job
     * is scheduled more often than this (e.g. the user is opening and closing the app regularly),
     * executions are skipped.
     */
    val minimumDurationBetweenBackgroundRuns: Duration = Duration.ofMinutes(60),

    /**
     * Number of messages to send for each execution of the background job. The number of expected
     * background jobs per day times this number denotes the expected message throughout per day.
     */
    val numMessagesPerBackgroundRun: Int = 2,

    /**
     * The amount of time that may pass between the activity being paused and resumed, before the
     * session is automatically closed.
     */
    val backgroundTimeoutForAutomaticLogout: Duration = Duration.ofMinutes(5),

    /**
     * Existing messages (either from the journalist or the user) are removed after this duration.
     * The cut-off is enforced whenever the mailbox is saved. Note: the mailbox is automatically
     * saved when unlocking a session and merging in new messages.
     */
    val messageExpiryDuration: Duration = Duration.ofDays(14),
) {
    init {
        if (localTestMode) {
            check(BuildConfig.DEBUG) { "Local test mode MUST NOT be enabled in release builds!" }
        }
    }
}
