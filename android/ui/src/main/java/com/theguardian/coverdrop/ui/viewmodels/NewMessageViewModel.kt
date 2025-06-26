package com.theguardian.coverdrop.ui.viewmodels

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.theguardian.coverdrop.core.ICoverDropPrivateDataRepository
import com.theguardian.coverdrop.core.ui.models.DraftMessage
import com.theguardian.coverdrop.ui.R
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import javax.inject.Inject

enum class NewMessageUiState {
    SHOWN,

    /** The message has been sent successfully */
    FINISHED,

    /** The user wants to leave the screen; let's ask for confirmation */
    CONFIRM_LEAVING,

    /** The user has aborted the flow */
    EXIT,
}

@HiltViewModel
class NewMessageViewModel @Inject constructor(
    application: Application,
    private val privateDataRepository: ICoverDropPrivateDataRepository,
) : AndroidViewModel(application) {
    private var _uiState = MutableStateFlow(NewMessageUiState.SHOWN)
    val uiState: StateFlow<NewMessageUiState> = _uiState.asStateFlow()

    private val message = MutableStateFlow("")
    private val busy = MutableStateFlow(false)
    private val errorMessage = MutableStateFlow<String?>(null)

    val messageSizeState = message.map { text ->
        DraftMessage(text = text).getFillLevel()
    }.stateIn(viewModelScope, SharingStarted.WhileSubscribed(), 0f)

    fun getErrorMessage(): StateFlow<String?> = errorMessage

    fun getMessage(): StateFlow<String> = message

    fun getBusy(): StateFlow<Boolean> = busy

    fun onMessageChanged(newValue: String) {
        message.value = newValue
    }

    fun onSendMessage(recipient: SelectedRecipientState) {
        if (!validateEntries(recipient) || busy.value) return
        busy.value = true

        viewModelScope.launch(Dispatchers.Default) {
            try {
                privateDataRepository.createNewConversation(
                    id = recipient.getJournalistInfoOrThrow().id,
                    message = DraftMessage(text = message.value),
                )
                _uiState.value = NewMessageUiState.FINISHED
            } catch (e: Exception) {
                val context = getApplication<Application>()
                setErrorMessage(context.getString(R.string.screen_new_message_error_cannot_send_unknown_error))
            } finally {
                busy.value = false
            }
        }
    }

    private fun validateEntries(recipient: SelectedRecipientState): Boolean {
        val context = getApplication<Application>()
        val validationErrors = mutableListOf<String>()

        // a recipient must be chosen
        if (!recipient.isValid()) {
            validationErrors.add(context.getString(R.string.screen_new_message_error_no_recipient_selected))
        }

        // the message must not be empty
        val message = message.value
        if (message.isEmpty()) {
            validationErrors.add(context.getString(R.string.screen_new_message_error_message_empty))
        }

        // the message must fit into the allocated length once compressed
        val draftMessage = DraftMessage(text = message)
        if (draftMessage.getFillLevel() > 1f) {
            validationErrors.add(context.getString(R.string.screen_new_message_error_message_too_long))
        }

        if (validationErrors.isNotEmpty()) {
            setErrorMessage(validationErrors.joinToString("\n"))
        } else {
            setErrorMessage(null)
        }

        return validationErrors.isEmpty()
    }

    private fun setErrorMessage(message: String?) {
        errorMessage.value = message
    }

    fun showExitConfirmationDialog() {
        _uiState.value = NewMessageUiState.CONFIRM_LEAVING
    }

    fun dismissCurrentDialog() {
        _uiState.value = NewMessageUiState.SHOWN
    }

    fun closeSession() {
        viewModelScope.launch {
            withContext(Dispatchers.Default) {
                privateDataRepository.lock()
            }
            _uiState.value = NewMessageUiState.EXIT
        }
    }
}
