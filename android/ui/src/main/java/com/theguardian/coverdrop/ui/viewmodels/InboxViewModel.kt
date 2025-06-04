package com.theguardian.coverdrop.ui.viewmodels

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.theguardian.coverdrop.core.CoverDropConfiguration
import com.theguardian.coverdrop.core.ICoverDropLib
import com.theguardian.coverdrop.core.models.MessageThread
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import javax.inject.Inject

sealed class InboxUiState {
    /** The inbox is still loading */
    data object Loading : InboxUiState()

    /** The inbox data is ready and being displayed */
    class ShowMessages(
        val activeConversation: MessageThread?,
        val inactiveConversation: List<MessageThread>,
    ) : InboxUiState()

    /** The user leaves the message vault */
    data object Exit : InboxUiState()
}

sealed class InboxDialogState {
    data object None : InboxDialogState()
    data object ShowDeleteConfirmationDialog : InboxDialogState()
    data object ShowDeletingProgressDialog : InboxDialogState()
    data object ShowDeletionErrorDialog : InboxDialogState()
    data object ShowExitConfirmationDialog : InboxDialogState()
}

@HiltViewModel
class InboxViewModel @Inject constructor(
    application: Application,
    configuration: CoverDropConfiguration,
    lib: ICoverDropLib,
) : AndroidViewModel(application) {
    private val privateDataRepository = lib.getPrivateDataRepository()

    private var _uiState = MutableStateFlow<InboxUiState>(InboxUiState.Loading)
    private var _dialogState = MutableStateFlow<InboxDialogState>(InboxDialogState.None)

    val uiState: StateFlow<InboxUiState> = _uiState.asStateFlow()
    val dialogState: StateFlow<InboxDialogState> = _dialogState.asStateFlow()

    val messageExpiryDuration = configuration.messageExpiryDuration

    init {
        loadMessages()
    }

    private fun loadMessages() {
        _uiState.value = InboxUiState.Loading
        viewModelScope.launch {
            _uiState.value = InboxUiState.ShowMessages(
                activeConversation = privateDataRepository.getActiveConversation(),
                inactiveConversation = privateDataRepository.getInactiveConversations(),
            )
        }
    }

    fun deleteVault() {
        _dialogState.value = InboxDialogState.ShowDeletingProgressDialog
        viewModelScope.launch {
            withContext(Dispatchers.Default) {
                privateDataRepository.deleteVault()
            }
            _uiState.value = InboxUiState.Exit
        }
    }

    fun closeSession() {
        viewModelScope.launch {
            withContext(Dispatchers.Default) {
                privateDataRepository.lock()
            }
            _uiState.value = InboxUiState.Exit
        }
    }

    fun showExitConfirmationDialog() {
        _dialogState.value = InboxDialogState.ShowExitConfirmationDialog
    }

    fun showDeleteConfirmationDialog() {
        _dialogState.value = InboxDialogState.ShowDeleteConfirmationDialog
    }

    fun dismissCurrentDialog() {
        _dialogState.value = InboxDialogState.None
    }
}
