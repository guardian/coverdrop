package com.theguardian.coverdrop.ui.tests

import com.theguardian.coverdrop.core.CoverDropConfiguration
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
        TODO("`provideCoverDropConfiguration` for the main source set of :ui-tests, but overridden in the FakeApplicationModule class in :ui-tests:androidTest")
    }
}
