package com.theguardian.coverdrop.ui.viewmodels

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.theguardian.coverdrop.core.CoverDropConfiguration
import com.theguardian.coverdrop.core.ICoverDropLib
import com.theguardian.coverdrop.core.models.DebugContext
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import javax.inject.Inject

@HiltViewModel
class AboutScreenViewModel @Inject constructor(
    application: Application,
    configuration: CoverDropConfiguration,
    lib: ICoverDropLib,
) : AndroidViewModel(application) {
    val debugContext = MutableStateFlow<DebugContext?>(null)

    init {
        if (configuration.showDebugInformation) {
            fetchDebugContext(lib)
        }
    }

    private fun fetchDebugContext(lib: ICoverDropLib) {
        viewModelScope.launch {
            withContext(Dispatchers.Default) {
                debugContext.value = lib.getDebugContext()
            }
        }
    }
}
