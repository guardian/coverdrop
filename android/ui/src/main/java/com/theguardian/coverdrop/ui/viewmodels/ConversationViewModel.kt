package com.theguardian.coverdrop.ui.viewmodels

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.theguardian.coverdrop.core.ICoverDropLib
import com.theguardian.coverdrop.core.models.MessageThread
import com.theguardian.coverdrop.core.ui.models.DraftMessage
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class ConversationViewModel @Inject constructor(
    application: Application,
    savedStateHandle: SavedStateHandle,
    coverDrop: ICoverDropLib,
) : AndroidViewModel(application) {

    private val privateDataRepository = coverDrop.getPrivateDataRepository()
    private val journalistId: String = checkNotNull(savedStateHandle["id"])

    private val mutableActiveConversation = MutableStateFlow<MessageThread?>(null)
    val activeConversation = mutableActiveConversation.asStateFlow()

    private val message = MutableStateFlow("")
    val messageSizeState = message.map { DraftMessage(text = message.value).getFillLimit() }

    init {
        getActive()
    }

    private fun getActive() {
        viewModelScope.launch {
            mutableActiveConversation.value =
                privateDataRepository.getConversationForId(journalistId)
        }
    }

    fun onMessageChanged(newValue: String) {
        message.value = newValue
    }

    fun onSendMessage() {
        viewModelScope.launch {
            val draftMessage = DraftMessage(text = message.value)
            privateDataRepository.replyToConversation(journalistId, draftMessage)
            getActive() // refresh conversation
        }
    }
}
