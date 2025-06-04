package com.theguardian.coverdrop.ui.di

import android.app.Application
import com.theguardian.coverdrop.core.CoverDropConfiguration
import com.theguardian.coverdrop.core.CoverDropLib
import com.theguardian.coverdrop.core.CoverDropPrivateDataRepository
import com.theguardian.coverdrop.core.CoverDropThrowingExceptionHandler
import com.theguardian.coverdrop.core.ICoverDropLib
import com.theguardian.coverdrop.core.ICoverDropPrivateDataRepository
import com.theguardian.coverdrop.core.ICoverDropPublicDataRepository
import com.theguardian.coverdrop.core.security.BackgroundTimeoutGuard
import com.theguardian.coverdrop.core.security.IBackgroundTimeoutGuard
import com.theguardian.coverdrop.core.ui.interfaces.SilenceableLifecycleCallbackProxy
import com.theguardian.coverdrop.core.ui.interfaces.SilenceableUncaughtExceptionHandler
import com.theguardian.coverdrop.ui.tracking.SilenceableIndirectLifecycleCallbackProxyImpl
import com.theguardian.coverdrop.ui.tracking.SilenceableUncaughtExceptionHandlerProxyImpl
import dagger.Binds
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.MainScope
import javax.inject.Named
import javax.inject.Singleton

object DiNamed {
    object CoroutineDispatchers {
        const val DEFAULT = "DefaultDispatcher"
        const val MAIN = "MainDispatcher"
        const val IO = "IoDispatcher"
        const val EMPTY = "EmptyContext"
    }
}

@Module
@InstallIn(SingletonComponent::class)
abstract class CoverDropModule {
    companion object {

        @Provides
        @Singleton
        fun provideCoverDropLib(
            application: Application,
            configuration: CoverDropConfiguration,
            @Named(DiNamed.CoroutineDispatchers.DEFAULT) defaultDispatcher: CoroutineDispatcher,
            silenceableLifecycleCallbackProxy: SilenceableLifecycleCallbackProxy,
            silenceableUncaughtExceptionHandler: SilenceableUncaughtExceptionHandler,
        ): CoverDropLib {
            CoverDropLib.onAppCreate(
                silenceableLifecycleCallbackProxy = silenceableLifecycleCallbackProxy,
                silenceableUncaughtExceptionHandler = silenceableUncaughtExceptionHandler
            )
            CoverDropLib.onAppInit(
                applicationContext = application,
                configuration = configuration,
                coroutineScope = MainScope(),
                defaultDispatcher = defaultDispatcher,
                exceptionHandler = CoverDropThrowingExceptionHandler(),
            )
            return CoverDropLib.getInstance()
        }

        @Provides
        @Singleton
        fun providePrivateRepository(lib: CoverDropLib): CoverDropPrivateDataRepository {
            return lib.getPrivateDataRepository()
        }

        @Provides
        @Singleton
        fun providePublicRepository(lib: CoverDropLib): ICoverDropPublicDataRepository {
            return lib.getPublicDataRepository()
        }

        @Provides
        @Singleton
        fun provideBackgroundTimeoutGuard(
            configuration: CoverDropConfiguration,
            lib: CoverDropLib
        ): IBackgroundTimeoutGuard {
            return BackgroundTimeoutGuard(configuration = configuration, lib = lib)
        }

        @Provides
        @Singleton
        fun provideSilenceableLifecycleCallbackProxy(): SilenceableLifecycleCallbackProxy {
            return SilenceableIndirectLifecycleCallbackProxyImpl.INSTANCE
        }

        @Provides
        @Singleton
        fun provideSilenceableUncaughtExceptionHandler(): SilenceableUncaughtExceptionHandler {
            return SilenceableUncaughtExceptionHandlerProxyImpl.INSTANCE
        }
    }

    @Binds
    abstract fun bindCoverdropLib(
        coverdrop: CoverDropLib,
    ): ICoverDropLib

    @Binds
    abstract fun bindPrivateDataRepository(
        coverDropPrivateDataRepository: CoverDropPrivateDataRepository,
    ): ICoverDropPrivateDataRepository
}
