package com.theguardian.coverdrop.ui.viewmodels

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.theguardian.coverdrop.core.ICoverDropLib
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import javax.inject.Inject

@HiltViewModel
class SplashViewModel @Inject constructor(
    application: Application,
    lib: ICoverDropLib,
) : AndroidViewModel(application) {
    val canProceedState = combine(
        lib.getInitializationSuccessful(),
        lib.getInitializationFailed()
    ) { success, failed ->
        success || failed
    }.stateIn(viewModelScope, SharingStarted.Lazily, false)
}
