package com.theguardian.coverdrop.ui.viewmodels

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.theguardian.coverdrop.core.ICoverDropLib
import com.theguardian.coverdrop.core.api.models.SystemStatus
import com.theguardian.coverdrop.core.models.StatusEvent
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class EntryViewModel @Inject constructor(
    application: Application,
    private val coverDropLib: ICoverDropLib
) : AndroidViewModel(application) {
    private val _statusEvent = MutableStateFlow<StatusEvent?>(null)
    val status = _statusEvent.asStateFlow()

    init {
        getStatus()
    }

    private fun getStatus() {
        viewModelScope.launch {
            _statusEvent.value = try {
                val statusEvent = coverDropLib.getPublicDataRepository().getStatusEvent()

                // Fail-safe to ensure that the app is in a lock state.
                //
                // The regular app flow should ensure this invariant. However, if the user
                // force-closes the CoverDrop activity, but the main app process is still running,
                // the CoverDropLib will still be in an unlocked state.
                ensureLocked()

                statusEvent
            } catch (e: IllegalStateException) {
                StatusEvent(
                    status = SystemStatus.UNAVAILABLE,
                    isAvailable = false,
                    description = "Failed to load the status information"
                )
            }
        }
    }

    private suspend fun ensureLocked() {
        val privateDataRepository = try {
            coverDropLib.getPrivateDataRepository()
        } catch (e: IllegalStateException) {
            // the most likely cause is that the library failed to initialize (e.g. first
            // time start without internet connectivity)
            null
        }
        privateDataRepository?.lock()
    }
}
