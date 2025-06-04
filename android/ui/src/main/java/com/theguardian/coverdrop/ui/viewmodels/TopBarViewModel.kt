package com.theguardian.coverdrop.ui.viewmodels

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.theguardian.coverdrop.core.CoverDropConfiguration
import com.theguardian.coverdrop.ui.usecase.LocalTestModeForceRefreshUseCase
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch
import javax.inject.Inject

/**
 * Using a global state object here. This is perhaps not as clean as using the public data
 * repository, but it avoids the need to change the shared API. This works reliably, because
 * we only ever the visibility of the warning banner from VISIBLE to HIDDEN.
 */
object TopBarGlobalState {
    var isWarningBarSnoozed = false
}

@HiltViewModel
class TopBarViewModel @Inject constructor(
    application: Application,
    internal val configuration: CoverDropConfiguration,
    private val localTestModeForceRefreshUseCase: LocalTestModeForceRefreshUseCase,
) : AndroidViewModel(application) {
    val isLocalTestMode = MutableStateFlow(configuration.localTestMode)
    val showWarningBanner = MutableStateFlow(false)
    val showWarningBannerSnoozeDialog = MutableStateFlow(false)

    init {
        updateShowWarningBannerState()
    }

    fun onForceRefresh() {
        viewModelScope.launch { localTestModeForceRefreshUseCase() }
    }

    fun onWarningBannerClick() {
        showWarningBannerSnoozeDialog.value = true
    }

    fun onSnoozeWarningBannerConfirm() {
        TopBarGlobalState.isWarningBarSnoozed = true
        showWarningBannerSnoozeDialog.value = false
        updateShowWarningBannerState()
    }

    fun onSnoozeWarningBannerDismiss() {
        showWarningBannerSnoozeDialog.value = false
    }

    private fun updateShowWarningBannerState() {
        showWarningBanner.value =
            configuration.showBetaWarningBanner && !TopBarGlobalState.isWarningBarSnoozed
    }
}
