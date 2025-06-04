package com.theguardian.coverdrop.ui.tests.di

import com.theguardian.coverdrop.core.CoverDropConfiguration
import com.theguardian.coverdrop.core.ICoverDropLib
import com.theguardian.coverdrop.core.ICoverDropPrivateDataRepository
import com.theguardian.coverdrop.core.ICoverDropPublicDataRepository
import com.theguardian.coverdrop.core.security.BackgroundTimeoutGuard
import com.theguardian.coverdrop.core.security.IBackgroundTimeoutGuard
import com.theguardian.coverdrop.ui.di.CoverDropModule
import com.theguardian.coverdrop.ui.tests.utils.CoverDropLibMock
import com.theguardian.coverdrop.ui.tests.utils.CoverDropPrivateDataRepositoryMock
import dagger.Binds
import dagger.Module
import dagger.Provides
import dagger.hilt.components.SingletonComponent
import dagger.hilt.testing.TestInstallIn
import javax.inject.Singleton

@Module
@TestInstallIn(
    components = [SingletonComponent::class],
    replaces = [CoverDropModule::class]
)
abstract class FakeCoverDropModule {
    companion object {

        @Provides
        @Singleton
        fun provideCoverDropLibMock(
        ): CoverDropLibMock {
            return CoverDropLibMock()
        }

        @Provides
        @Singleton
        fun providePrivateRepository(lib: CoverDropLibMock): CoverDropPrivateDataRepositoryMock {
            return lib.getPrivateDataRepository()
        }

        @Provides
        @Singleton
        fun providePublicRepository(lib: CoverDropLibMock): ICoverDropPublicDataRepository {
            return lib.getPublicDataRepository()
        }

        @Provides
        @Singleton
        fun provideBackgroundTimeoutGuard(
            configuration: CoverDropConfiguration,
            lib: CoverDropLibMock
        ): IBackgroundTimeoutGuard {
            return BackgroundTimeoutGuard(configuration = configuration, lib = lib)
        }
    }

    @Binds
    @Singleton
    abstract fun bindCoverdropLib(
        coverdrop: CoverDropLibMock,
    ): ICoverDropLib

    @Binds
    @Singleton
    abstract fun bindPrivateDataRepository(
        coverDropPrivateDataRepository: CoverDropPrivateDataRepositoryMock,
    ): ICoverDropPrivateDataRepository
}
