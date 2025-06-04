package com.theguardian.coverdrop.core

import android.app.Application
import com.goterl.lazysodium.SodiumAndroid
import com.theguardian.coverdrop.core.api.IApiCallProvider
import com.theguardian.coverdrop.core.background.BackgroundWorkManager
import com.theguardian.coverdrop.core.crypto.DeadDropParser
import com.theguardian.coverdrop.core.crypto.KeyVerifier
import com.theguardian.coverdrop.core.crypto.PassphraseWordList
import com.theguardian.coverdrop.core.crypto.Protocol
import com.theguardian.coverdrop.core.crypto.VerificationFailureBehaviour
import com.theguardian.coverdrop.core.encryptedstorage.IEncryptedStorage
import com.theguardian.coverdrop.core.persistence.PublicStorage
import com.theguardian.coverdrop.core.utils.DefaultClock
import com.theguardian.coverdrop.core.utils.IClock
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.runBlocking

internal fun createLibSodium(): SodiumAndroid {
    val sodium = SodiumAndroid()
    sodium.sodium_init()
    return sodium
}

/**
 * A fixture for testing the implementations of the CoverDrop components that require other
 * components to be injected using [ICoverDropLibInternal]. Components that are not explicitly
 * overridden in the constructor will throw an exception when accessed.
 */
internal class CoverDropLibInternalFixture(
    private val mContext: Application,
    private val mApiCallProvider: IApiCallProvider? = null,
    private val mConfig: CoverDropConfiguration? = null,
    private val mLibSodium: SodiumAndroid = createLibSodium(),
    private val mEncryptedStorage: IEncryptedStorage? = null,
    private val mPublicStorage: PublicStorage? = null,
    private val mClock: IClock = DefaultClock(),
) : ICoverDropLibInternal {

    private val mPublicDataRepository: CoverDropPublicDataRepository by lazy {
        CoverDropPublicDataRepository(this)
    }

    private val mPrivateSendingQueueProvider: PrivateSendingQueueProvider by lazy {
        PrivateSendingQueueProvider(getPublicStorage())
    }

    fun initialize(): Unit = runBlocking {
        mPublicStorage?.initialize()
        mEncryptedStorage?.onAppStart()
        mPublicDataRepository.initialize()
    }

    private val mLockStateSharedFlow = MutableSharedFlow<LockState>(replay = 0)

    private val mPassphraseWordList = PassphraseWordList.createFromEffWordList(mContext)

    override fun getApiCallProvider(): IApiCallProvider {
        return checkNotNull(mApiCallProvider) { "mApiCallProvider not provided" }
    }

    override fun getApplicationContext(): Application {
        return checkNotNull(mContext) { "mContext not provided" }
    }

    override fun getLibSodium() = mLibSodium

    override fun getConfig(): CoverDropConfiguration {
        return checkNotNull(mConfig) { "mConfig not provided" }
    }

    override fun getEncryptedStorage(): IEncryptedStorage {
        return checkNotNull(mEncryptedStorage) { "mEncryptedStorage not provided" }
    }

    override fun getPublicStorage(): PublicStorage {
        return checkNotNull(mPublicStorage) { "mPublicStorage not provided" }
    }

    override fun getPrivateSendingQueueProvider() = mPrivateSendingQueueProvider

    override fun getProtocol() = Protocol(mLibSodium)

    override fun getKeyVerifier(): KeyVerifier = KeyVerifier(mLibSodium)

    override fun getClock() = mClock

    override fun getDeadDropParser() = DeadDropParser(
        libSodium = mLibSodium,
        verificationFailureBehaviour = VerificationFailureBehaviour.THROW
    )

    // The background work manager and the repositories are our final artifacts, and hence they
    // are the only ones that take in the ICoverDropLibInternal as a constructor parameter.

    override fun getBackgroundWorkManager() = BackgroundWorkManager(this)

    override fun getPublicDataRepository() = mPublicDataRepository

    override fun getPrivateDataRepository() = CoverDropPrivateDataRepository(this)

    override fun getLockFlow(): SharedFlow<LockState> = mLockStateSharedFlow

    override fun publishLockState(lockState: LockState) {
        mLockStateSharedFlow.tryEmit(lockState)
    }

    override fun getInitializationSuccessful() = MutableStateFlow(true)

    override fun getInitializationFailed() = MutableStateFlow(false)

    override suspend fun waitForInitialization(timeoutMs: Long, clock: IClock) {
        // the test fixture is always initialized
    }

    override fun getPassphraseWordList() = mPassphraseWordList

    override suspend fun forceRefreshInLocalTestMode() {
        TODO("Not yet implemented")
    }
}
