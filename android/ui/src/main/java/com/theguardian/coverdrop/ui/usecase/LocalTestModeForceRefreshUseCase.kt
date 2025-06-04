package com.theguardian.coverdrop.ui.usecase

import com.theguardian.coverdrop.core.ICoverDropLib
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import javax.inject.Inject

class LocalTestModeForceRefreshUseCase @Inject constructor(
    private val lib: ICoverDropLib,
) {
    suspend operator fun invoke() {
        withContext(Dispatchers.Default) {
            lib.forceRefreshInLocalTestMode()
        }
    }
}
