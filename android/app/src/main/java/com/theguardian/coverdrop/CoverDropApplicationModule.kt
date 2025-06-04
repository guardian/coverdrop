package com.theguardian.coverdrop

import com.theguardian.coverdrop.core.CoverDropConfiguration
import com.theguardian.coverdrop.core.api.ApiConfiguration
import com.theguardian.coverdrop.core.api.IApiCallProvider
import com.theguardian.coverdrop.core.crypto.PublicSigningKey
import com.theguardian.coverdrop.tracking.SampleLifecycleListener
import com.theguardian.coverdrop.ui.di.DiNamed
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import javax.inject.Named
import javax.inject.Singleton
import kotlin.coroutines.CoroutineContext
import kotlin.coroutines.EmptyCoroutineContext

@Module
@InstallIn(SingletonComponent::class)
class CoverDropApplicationModule {
    @Provides
    @Singleton
    fun providesGlobalScope(): CoroutineScope {
        return CoroutineScope(SupervisorJob() + Dispatchers.Default)
    }

    @Named(DiNamed.CoroutineDispatchers.DEFAULT)
    @Provides
    fun providesDefaultDispatcher(): CoroutineDispatcher {
        return Dispatchers.Default
    }

    @Named(DiNamed.CoroutineDispatchers.MAIN)
    @Provides
    fun providesMainDispatcher(): CoroutineDispatcher {
        return Dispatchers.Main
    }

    @Named(DiNamed.CoroutineDispatchers.IO)
    @Provides
    fun providesIoDispatcher(): CoroutineDispatcher {
        return Dispatchers.IO
    }

    @Named(DiNamed.CoroutineDispatchers.EMPTY)
    @Provides
    fun providesEmptyCoroutineContext(): CoroutineContext = EmptyCoroutineContext

    @Provides
    @Singleton
    fun provideCoverDropConfiguration(): CoverDropConfiguration {
        return CoverDropConfiguration(
            apiConfiguration = getApiConfiguration(),
            createApiCallProvider = ::apiCallProviderFactory,
            trustedOrgPks = getTrustedOrgPks(),
            localTestMode = BuildConfig.LOCAL_TEST_MODE_ENABLED,
            disableScreenCaptureProtection = BuildConfig.SCREEN_CAPTURE_PROTECTION_DISABLED,
        )
    }

    @Provides
    @Singleton
    fun provideSampleLifecycleListener(): SampleLifecycleListener {
        return SampleLifecycleListener()
    }

    private fun apiCallProviderFactory(): IApiCallProvider {
        return OkHttpApiCallProvider()
    }

    private fun getApiConfiguration() = ApiConfiguration(
        apiBaseUrl = BuildConfig.API_BASE_URL,
        messagingBaseUrl = BuildConfig.MESSAGING_BASE_URL,
    )

    private fun getTrustedOrgPks(): List<PublicSigningKey> {
        val orgPks = BuildConfig.TRUSTED_ORG_PKS.split(',')
        return orgPks.map { PublicSigningKey.fromHexEncodedString(it) }
    }
}
