package com.theguardian.coverdrop.core

import android.app.Application
import android.content.Context
import android.util.Log
import com.goterl.lazysodium.SodiumAndroid
import com.theguardian.coverdrop.core.api.IApiCallProvider
import com.theguardian.coverdrop.core.background.BackgroundWorkManager
import com.theguardian.coverdrop.core.background.CoverDropBackgroundJob
import com.theguardian.coverdrop.core.crypto.DeadDropParser
import com.theguardian.coverdrop.core.crypto.KeyVerifier
import com.theguardian.coverdrop.core.crypto.PassphraseWordList
import com.theguardian.coverdrop.core.crypto.Protocol
import com.theguardian.coverdrop.core.crypto.VerificationFailureBehaviour
import com.theguardian.coverdrop.core.crypto.getHumanReadableDigest
import com.theguardian.coverdrop.core.encryptedstorage.EncryptedStorageConfiguration
import com.theguardian.coverdrop.core.encryptedstorage.EncryptedStorageImpl
import com.theguardian.coverdrop.core.encryptedstorage.IEncryptedStorage
import com.theguardian.coverdrop.core.encryptedstorage.SecureElementAvailabilityCache
import com.theguardian.coverdrop.core.encryptedstorage.selectEncryptedStorageConfiguration
import com.theguardian.coverdrop.core.models.DebugContext
import com.theguardian.coverdrop.core.models.ErrorDuringInitialization
import com.theguardian.coverdrop.core.persistence.CoverDropFileManager
import com.theguardian.coverdrop.core.persistence.PublicStorage
import com.theguardian.coverdrop.core.security.IntegrityGuard
import com.theguardian.coverdrop.core.ui.interfaces.SilenceableLifecycleCallbackProxy
import com.theguardian.coverdrop.core.ui.interfaces.SilenceableUncaughtExceptionHandler
import com.theguardian.coverdrop.core.utils.IClock
import com.theguardian.coverdrop.core.utils.MonotonicClock
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.MainScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import java.time.Duration

interface ICoverDropLib {
    /**
     * Repository to observe and act on private data which is stored encrypted on the disk. This
     * repository is only available after the library is fully initialized.
     */
    fun getPrivateDataRepository(): ICoverDropPrivateDataRepository

    /**
     * Repository to observe data that is not specific to usage of CoverDrop. This data is typically
     * downloaded in the background.
     */
    fun getPublicDataRepository(): ICoverDropPublicDataRepository

    /**
     * Observable that returns whether has been initialized successfully. This typically
     * updates to `true` within a few seconds after the first call to [CoverDropLib.onAppInit].
     *
     * Either this or [getInitializationFailed] will eventually become `true` after the first call
     * to [CoverDropLib.onAppInit].
     */
    fun getInitializationSuccessful(): StateFlow<Boolean>

    /**
     * Observable that returns whether the initialization failed. Similar to
     * [getInitializationSuccessful], this would updates to `true` within a few seconds after
     * the first call to [CoverDropLib.onAppInit] if the initialization failed.
     *
     * Either this or [getInitializationSuccessful] will eventually become `true` after the first
     * call to [CoverDropLib.onAppInit].
     *
     * The UI should check the status event from the [getPublicDataRepository] to get more
     * information about the error.
     */
    fun getInitializationFailed(): StateFlow<Boolean>

    /**
     * Observable that allows its subscribers to react to changes in the lock state. This is useful
     * for UI components that need to react to changes in the lock state. E.g. we want to
     * automatically navigate to the entry screens when the lock state changes to
     * [LockState.LOCKED].
     */
    fun getLockFlow(): SharedFlow<LockState>

    /**
     * This method can be used during local testing to force running of the background jobs
     * including downloading the published keys and dead-drops as well as the [CoverDropBackgroundJob].
     */
    suspend fun forceRefreshInLocalTestMode()

    /**
     * This method returns the passphrase word list. This is useful for UI components that need to
     * validate user input dynamically.
     */
    fun getPassphraseWordList(): PassphraseWordList

    /**
     * Returns the debug context which can be used to diagnose issues with the app. Might not be
     * available before the initialization is complete.
     */
    fun getDebugContext(): DebugContext
}

/**
 * This is the main entry point to the CoverDrop library. Integrating applications must call
 * [CoverDropLib.onAppCreate] at app startup and then [CoverDropLib.onAppInit] as soon as possible.
 * This is usually done during start-up in the main application file.
 */
class CoverDropLib(
    context: Context,
    configuration: CoverDropConfiguration,
    defaultDispatcher: CoroutineDispatcher,
) : ICoverDropLib {

    /**
     * We use composition to hide the internal state of the CoverDrop library from the integrating
     * application. This is necessary to allow using an interface internally when passing around
     * the main state. If this interface would be implemented on this class, it would necessarily
     * expose some internal classes as interface methods must be public.
     */
    private val internal = CoverDropLibInternal(
        mApplicationContext = context.applicationContext,
        mConfiguration = configuration,
        mDefaultDispatcher = defaultDispatcher
    )

    companion object {
        private var sInstance: CoverDropLib? = null
        private var sSilenceableLifecycleCallbackProxy: SilenceableLifecycleCallbackProxy? = null
        private var sSilenceableUncaughtExceptionHandler: SilenceableUncaughtExceptionHandler? =
            null

        /**
         * The calling application must provide a [SilenceableLifecycleCallbackProxy] and a
         * [SilenceableUncaughtExceptionHandler] to allow temporarily allow deactivating common
         * code paths that might leak usage information.
         */
        fun onAppCreate(
            silenceableLifecycleCallbackProxy: SilenceableLifecycleCallbackProxy,
            silenceableUncaughtExceptionHandler: SilenceableUncaughtExceptionHandler
        ) {
            sSilenceableLifecycleCallbackProxy = silenceableLifecycleCallbackProxy
            sSilenceableUncaughtExceptionHandler = silenceableUncaughtExceptionHandler
        }

        /**
         * To be called by the integration as early as possible. All expensive operations are
         * dispatched to [defaultDispatcher] which should make it easy and safe to call from the
         * [Application.onCreate] method. However, it is safe to call this method delayed (e.g.
         * using a worker) to reduce the impact on app startup time. If the user navigates to the
         * [CoverDropActivity] before the initialization finished, a splash screen with a spinner
         * is shown.
         *
         * The method is idempotent and thus safe to be called multiple times. The application can
         * (although rarely needs to) check the current initialization status by observing the
         * live data returned from [getIsFullyInitialized].
         *
         * The calling application must provide a custom [ICoverDropExceptionHandler] to catch
         * exceptions thrown within the [coroutineScope]. See also:
         * [CoverDropThrowingExceptionHandler].
         */
        @Synchronized
        fun onAppInit(
            applicationContext: Application,
            configuration: CoverDropConfiguration,
            coroutineScope: CoroutineScope,
            defaultDispatcher: CoroutineDispatcher,
            exceptionHandler: ICoverDropExceptionHandler,
        ) {
            if (sInstance != null) {
                return
            }

            sInstance = CoverDropLib(
                context = applicationContext,
                configuration = configuration,
                defaultDispatcher = defaultDispatcher,
            )

            // This internally hands-over to the default dispatcher for the long-running methods
            // that do not require running on the main thread. Note that this might finish after
            // the `onCreate` of the first activity.
            coroutineScope.launch {
                try {
                    // guaranteed to be non-null: we only ever set it to a non-null value
                    sInstance!!.internal.internalOnAppInit()
                } catch (e: Exception) {
                    exceptionHandler(e)
                }
            }
        }

        /**
         * To be called by the integration when the app is closed by the user. It is used for any
         * remaining clean-up tasks and scheduling of background work. Since there is no generic
         * app-wide callback, it is suggested to call this from the [android.app.Activity.onPause]
         * callback. It is safe to call multiple times during the app lifecycle.
         */
        fun onAppExit() {
            sInstance!!.internal.internalOnAppExit()
        }

        /**
         * Optional: to be called by the integration when the app is resumed by the user. It is used
         * to potentially download new public data and run the [CoverDropBackgroundJob]. Since there
         * is no generic app-wide callback, it is suggested to call this from the main activity's
         * [android.app.Activity.onResume].
         */
        suspend fun onAppResume() {
            sInstance?.internal?.internalOnAppForegrounded()
        }

        /**
         * Returns a reference to the integrating apps implementation of the
         * [SilenceableLifecycleCallbackProxy]. This will throw an [IllegalStateException] if no
         * such proxy has been set.
         */
        fun getSilenceableLifecycleCallbackProxy(): SilenceableLifecycleCallbackProxy {
            return checkNotNull(sSilenceableLifecycleCallbackProxy)
        }

        /**
         * Returns a reference to the integrating apps implementation of the
         * [SilenceableUncaughtExceptionHandler]. This will throw an [IllegalStateException] if no
         * such proxy has been set.
         */
        fun getSilenceableUncaughtExceptionHandler(): SilenceableUncaughtExceptionHandler {
            return checkNotNull(sSilenceableUncaughtExceptionHandler)
        }

        @Synchronized
        fun getInstance(): CoverDropLib {
            require(sInstance != null) { "CoverDrop must be initialised through `onAppInit` before usage" }
            return sInstance!!
        }
    }

    override fun getInitializationSuccessful() = internal.getInitializationSuccessful()

    override fun getInitializationFailed() = internal.getInitializationFailed()

    override fun getLockFlow() = internal.getLockFlow()

    override fun getPrivateDataRepository() = internal.getPrivateDataRepository()

    override fun getPublicDataRepository(): ICoverDropPublicDataRepository =
        internal.getPublicDataRepository()

    override suspend fun forceRefreshInLocalTestMode() = internal.forceRefreshInLocalTestMode()

    /**
     * Required for access to the internals from [CoverDropBackgroundJob].
     */
    internal fun getInternal(): ICoverDropLibInternal = internal

    override fun getPassphraseWordList(): PassphraseWordList = internal.getPassphraseWordList()

    override fun getDebugContext(): DebugContext {
        val trustedKeys = internal.getConfig().trustedOrgPks
        val hashedTrustedKeys = trustedKeys
            .map { getHumanReadableDigest(internal.getLibSodium(), it) }
            .joinTo(StringBuilder(), separator = "; ", prefix = "[", postfix = "]")
            .toString()

        val publicStorage = internal.getPublicStorage()
        return DebugContext(
            lastUpdatePublicKeys = publicStorage.readPublishedKeysLastUpdate(),
            lastUpdateDeadDrops = publicStorage.readPublishedDeadDropsLastUpdate(),
            lastBackgroundTry = publicStorage.readBackgroundJobLastTriggered(),
            lastBackgroundSend = publicStorage.readBackgroundJobLastRun(),
            hashedOrgKey = hashedTrustedKeys,
        )
    }
}

/**
 * Internal interface for accessing [CoverDropLib] within the individual sub-components and to
 * facilitate mocking in tests.
 */
internal interface ICoverDropLibInternal {
    fun getApiCallProvider(): IApiCallProvider
    fun getApplicationContext(): Context
    fun getLibSodium(): SodiumAndroid
    fun getConfig(): CoverDropConfiguration
    fun getPublicDataRepository(): ICoverDropPublicDataRepositoryInternal
    fun getPrivateDataRepository(): CoverDropPrivateDataRepository
    fun getEncryptedStorage(): IEncryptedStorage
    fun getBackgroundWorkManager(): BackgroundWorkManager
    fun getProtocol(): Protocol
    fun getKeyVerifier(): KeyVerifier
    fun getClock(): IClock
    fun getDeadDropParser(): DeadDropParser
    fun getPublicStorage(): PublicStorage
    fun getPrivateSendingQueueProvider(): PrivateSendingQueueProvider
    fun getInitializationSuccessful(): StateFlow<Boolean>
    fun getInitializationFailed(): StateFlow<Boolean>
    fun publishLockState(lockState: LockState)
    fun getLockFlow(): SharedFlow<LockState>
    fun getPassphraseWordList(): PassphraseWordList

    suspend fun forceRefreshInLocalTestMode()

    suspend fun waitForInitialization(
        timeoutMs: Long = 30_000L,
        clock: IClock = MonotonicClock(),
    )
}

internal class CoverDropLibInternal(
    private val mApplicationContext: Context,
    private val mConfiguration: CoverDropConfiguration,
    private val mDefaultDispatcher: CoroutineDispatcher,
) : ICoverDropLibInternal {

    private val mApiCallProvider: IApiCallProvider by lazy {
        mConfiguration.createApiCallProvider()
    }

    private val mLibSodium: SodiumAndroid by lazy {
        val sodium = SodiumAndroid()
        sodium.sodium_init()
        sodium
    }

    private val mFileManager: CoverDropFileManager by lazy {
        CoverDropFileManager(mApplicationContext) // file-system access
    }

    private val mPassphraseWordList: PassphraseWordList by lazy {
        PassphraseWordList.createFromEffWordList(mApplicationContext)
    }

    private val mPublicStorage: PublicStorage by lazy {
        PublicStorage(mApplicationContext, mFileManager)
    }

    private val mSecureElementAvailabilityCache: SecureElementAvailabilityCache by lazy {
        SecureElementAvailabilityCache(mApplicationContext, mPublicStorage)
    }

    private val mEncryptedStorageConfiguration: EncryptedStorageConfiguration by lazy {
        selectEncryptedStorageConfiguration(
            secureElementAvailabilityCache = mSecureElementAvailabilityCache,
            isForLocalTestMode = mConfiguration.localTestMode
        )
    }

    private val mEncryptedStorage: IEncryptedStorage by lazy {
        EncryptedStorageImpl(
            context = mApplicationContext,
            libSodium = mLibSodium,
            fileManager = mFileManager,
            encryptedStorageConfiguration = mEncryptedStorageConfiguration,
            passphraseWordList = mPassphraseWordList
        )
    }

    private val mProtocol = Protocol(mLibSodium)

    private val mKeyVerifier = KeyVerifier(mLibSodium)

    private val mClock = mConfiguration.clock

    private val mDeadDropParser = DeadDropParser(
        libSodium = mLibSodium,
        verificationFailureBehaviour = VerificationFailureBehaviour.DROP
    )

    // The background work manager and the repositories are our final artifacts, and hence they
    // are the only ones that take in the ICoverDropLibInternal as a constructor parameter.

    private val mBackgroundWorkManager: BackgroundWorkManager by lazy {
        BackgroundWorkManager(this)
    }

    private val mPrivateSendingQueueProvider: PrivateSendingQueueProvider by lazy {
        PrivateSendingQueueProvider(mPublicStorage)
    }

    private val mPublicDataRepository: CoverDropPublicDataRepository by lazy {
        CoverDropPublicDataRepository(this)
    }

    private val mPrivateDataRepository: CoverDropPrivateDataRepository by lazy {
        check(mInitializationSuccessful) { "CoverDropLib did not initialize successfully (yet)" }
        CoverDropPrivateDataRepository(this)
    }

    private var mInitializationSuccessful = false
    private val mStateFlowInitializationSuccessful = MutableStateFlow(mInitializationSuccessful)

    private var mInitializationFailed = false
    private val mStateFlowInitializationFailed = MutableStateFlow(mInitializationFailed)

    private val mLockStateSharedFlow: MutableSharedFlow<LockState> = MutableSharedFlow(replay = 0)
    private val mLockStateStateFlow: StateFlow<LockState> = mLockStateSharedFlow.stateIn(
        scope = MainScope(),
        started = SharingStarted.Eagerly,
        initialValue = LockState.LOCKED
    )

    internal suspend fun internalOnAppInit() {
        try {
            withContext(mDefaultDispatcher) {
                // first access causes initialization (includes .so load and JNI call)
                mLibSodium

                // first access causes initialization (includes I/O write operations)
                mFileManager

                // first access causes initialization (includes I/O read operations)
                mPassphraseWordList

                // first access causes initialization (caching the SE availability)
                mSecureElementAvailabilityCache

                // first access causes initialization (includes slow cryptographic and I/O operations)
                mEncryptedStorage.onAppStart()

                // downloads new public data from the server (if required) and initialized the private
                // sending queue
                mPublicDataRepository.initialize()

                // potentially executes background work that has not been executed yet
                mBackgroundWorkManager.onAppStart()

                // check for potential integrity violations
                IntegrityGuard.INSTANCE.apply {
                    checkForRoot(mApplicationContext)
                    checkForScreenLock(mApplicationContext)
                    checkForDebuggable(mApplicationContext)
                }
            }

            mInitializationSuccessful = true
            mStateFlowInitializationSuccessful.value = true
        } catch (e: Exception) {
            // This log statement is safe because it is independent of usage
            Log.d("CoverDropLib", "CoverDrop failed to initialize: $e")

            // Setting the error state ensures we can advance to the EntryScreen with a slightly
            // more helpful error message.
            mPublicDataRepository.overrideStatusEventForTesting(ErrorDuringInitialization)

            mInitializationFailed = true
            mStateFlowInitializationFailed.value = true
        }
    }

    /**
     * Waits for the library to be fully initialized. This method is blocking and should only be
     * called from a background thread. It will return as soon as the library is fully initialized
     * or the timeout is reached.
     *
     * The library is considered fully initialized when the `onAppInit()` method finished either
     * successfully or with an exception, i.e. either `mInitializationSuccessful` or
     * `mInitializationFailed` is `true`.
     */
    override suspend fun waitForInitialization(timeoutMs: Long, clock: IClock) {
        val deadlineMs = clock.now() + Duration.ofMillis(timeoutMs)
        while (clock.now() < deadlineMs && !mInitializationSuccessful && !mInitializationFailed) {
            delay(timeMillis = 100)
        }
        check(mInitializationSuccessful || mInitializationFailed)
    }

    fun internalOnAppExit() {
        mBackgroundWorkManager.onAppFinished()
    }

    private val mAppForegroundedLock = Mutex()

    suspend fun internalOnAppForegrounded() {
        // if there is another initialization underway, we skip this call
        if (mAppForegroundedLock.isLocked) {
            return
        }

        // ensure we do not perform too much work in parallel
        mAppForegroundedLock.withLock {
            withContext(mDefaultDispatcher) {
                try {
                    // We might race with the general initialization that happens on app start.
                    // Therefore we potentially wait until that has finished
                    waitForInitialization(timeoutMs = 60_000, clock = mClock)
                } catch (notYetInitialized: Exception) {
                    // in case normal initialization failed (or has not yet happened), we skip
                    return@withContext
                }

                // This will update the cached API responses if they are outdated (see the respective
                // value: CoverDropConfiguration#minimumDurationBetweenDownloads)
                mPublicDataRepository.maybeUpdateCachedApiResponses()

                // This will send messages that are pending in the queue in case they have not been
                // sent in the background yet (and we have not done so recently)
                mBackgroundWorkManager.onAppStart()
            }
        }
    }

    override fun getInitializationSuccessful() = mStateFlowInitializationSuccessful

    override fun getInitializationFailed() = mStateFlowInitializationFailed

    override fun getApiCallProvider(): IApiCallProvider = mApiCallProvider

    override fun getApplicationContext(): Context = mApplicationContext

    override fun getLibSodium(): SodiumAndroid = mLibSodium

    override fun getConfig(): CoverDropConfiguration = mConfiguration

    override fun getPublicDataRepository() = mPublicDataRepository

    override fun getPrivateDataRepository() = mPrivateDataRepository

    override fun getEncryptedStorage(): IEncryptedStorage = mEncryptedStorage

    override fun getBackgroundWorkManager(): BackgroundWorkManager = mBackgroundWorkManager

    override fun getProtocol(): Protocol = mProtocol

    override fun getKeyVerifier(): KeyVerifier = mKeyVerifier

    override fun getClock(): IClock = mClock

    override fun getDeadDropParser(): DeadDropParser = mDeadDropParser

    override fun getPrivateSendingQueueProvider(): PrivateSendingQueueProvider =
        mPrivateSendingQueueProvider

    override fun getPublicStorage(): PublicStorage = mPublicStorage

    override fun publishLockState(lockState: LockState) {
        if (lockState == mLockStateStateFlow.value) return
        MainScope().launch {
            mLockStateSharedFlow.emit(lockState)
        }
    }

    override fun getLockFlow() = mLockStateSharedFlow

    override fun getPassphraseWordList(): PassphraseWordList = mPassphraseWordList

    override suspend fun forceRefreshInLocalTestMode() {
        check(mConfiguration.localTestMode) {
            "forceRefreshInLocalTestMode() can only be called in local test mode"
        }

        withContext(mDefaultDispatcher) {
            mPublicDataRepository.forceUpdateCachedApiResponses()
            CoverDropBackgroundJob.run(
                this@CoverDropLibInternal,
                ignoreRateLimit = true
            )
        }
    }
}
