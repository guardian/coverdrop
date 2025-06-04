package com.theguardian.coverdrop.ui.tests.di

import androidx.test.platform.app.InstrumentationRegistry
import com.theguardian.coverdrop.core.CoverDropConfiguration
import com.theguardian.coverdrop.testutils.TestScenario
import com.theguardian.coverdrop.testutils.createCoverDropConfigurationForTest
import com.theguardian.coverdrop.ui.BuildConfig
import com.theguardian.coverdrop.ui.di.DiNamed
import com.theguardian.coverdrop.ui.tests.CoverDropApplicationModule
import dagger.Module
import dagger.Provides
import dagger.hilt.components.SingletonComponent
import dagger.hilt.testing.TestInstallIn
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import javax.inject.Named
import javax.inject.Singleton
import kotlin.coroutines.CoroutineContext
import kotlin.coroutines.EmptyCoroutineContext

@Module
@TestInstallIn(
    components = [SingletonComponent::class],
    replaces = [CoverDropApplicationModule::class]
)
class FakeApplicationModule {
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
        return createCoverDropConfigurationForTest(
            // this needs to be the instrumentation context as otherwise it will not find the
            // `assets` folder imported from `:test-utils`
            context = InstrumentationRegistry.getInstrumentation().context,
            apiBaseUrl = BuildConfig.API_BASE_URL,
            messagingBaseUrl = BuildConfig.MESSAGING_BASE_URL,
            scenario = TestScenario.Minimal,
        )
    }

}
