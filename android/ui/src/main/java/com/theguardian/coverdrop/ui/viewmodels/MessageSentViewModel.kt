package com.theguardian.coverdrop.ui.viewmodels

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.theguardian.coverdrop.core.ICoverDropPrivateDataRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import javax.inject.Inject

enum class MessageSentUiState {
    SHOWN,
    CONFIRM_LEAVING,
    EXIT,
}

@HiltViewModel
class MessageSentViewModel @Inject constructor(
    application: Application,
    private val privateDataRepository: ICoverDropPrivateDataRepository,
) : AndroidViewModel(application) {
    private var _uiState = MutableStateFlow(MessageSentUiState.SHOWN)
    val uiState: StateFlow<MessageSentUiState> = _uiState.asStateFlow()

    fun showExitConfirmationDialog() {
        _uiState.value = MessageSentUiState.CONFIRM_LEAVING
    }

    fun dismissCurrentDialog() {
        _uiState.value = MessageSentUiState.SHOWN
    }

    fun closeSession() {
        viewModelScope.launch {
            withContext(Dispatchers.Default) {
                privateDataRepository.lock()
            }
            _uiState.value = MessageSentUiState.EXIT
        }
    }
}
