package com.theguardian.coverdrop.ui.viewmodels

import androidx.annotation.VisibleForTesting
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.theguardian.coverdrop.core.ICoverDropPublicDataRepository
import com.theguardian.coverdrop.core.models.JournalistInfo
import com.theguardian.coverdrop.ui.viewmodels.SelectedRecipientState.EmptySelectionWithChoice
import com.theguardian.coverdrop.ui.viewmodels.SelectedRecipientState.Initializing
import com.theguardian.coverdrop.ui.viewmodels.SelectedRecipientState.SingleRecipientForced
import com.theguardian.coverdrop.ui.viewmodels.SelectedRecipientState.SingleRecipientWithChoice
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import javax.inject.Inject

sealed class SelectedRecipientState {
    /** We are still initializing the recipient selection */
    data object Initializing : SelectedRecipientState()

    /** The backend has returned only one journalist, so we do not offer a choice */
    data class SingleRecipientForced(val journalistInfo: JournalistInfo) : SelectedRecipientState()

    /**
     * The backend has returned multiple journalists, but no default one. Hence, the user is
     * required to choose one.
     */
    data object EmptySelectionWithChoice : SelectedRecipientState()

    /** The user has chosen one journalist and has the option to change it */
    data class SingleRecipientWithChoice(
        val journalistInfo: JournalistInfo
    ) : SelectedRecipientState()

    fun userHasChoice(): Boolean {
        return this is EmptySelectionWithChoice || this is SingleRecipientWithChoice
    }

    fun isValid(): Boolean {
        return this is SingleRecipientForced || this is SingleRecipientWithChoice
    }

    fun getJournalistInfoOrNull(): JournalistInfo? {
        return when (this) {
            is SingleRecipientForced -> journalistInfo
            is SingleRecipientWithChoice -> journalistInfo
            else -> null
        }
    }

    fun getJournalistInfoOrThrow(): JournalistInfo {
        return getJournalistInfoOrNull() ?: throw IllegalStateException("No recipient selected")
    }
}

/**
 * The currently selected recipient within the new message flow; this is shared to allow
 * sharing the selection result with the [NewMessageScreen].
 */
@HiltViewModel
class SelectedRecipientViewModel @Inject constructor(
    private val publicDataRepository: ICoverDropPublicDataRepository
) : ViewModel() {
    private val selectedRecipient = MutableStateFlow<SelectedRecipientState>(Initializing)

    private fun initialize(recipients: List<JournalistInfo>, defaultRecipient: JournalistInfo?) {
        if (recipients.isEmpty()) {
            return
        } else if (recipients.size == 1) {
            selectedRecipient.value = SingleRecipientForced(recipients.single())
        } else {
            if (defaultRecipient != null) {
                selectedRecipient.value = SingleRecipientWithChoice(defaultRecipient)
            } else {
                selectedRecipient.value = EmptySelectionWithChoice
            }
        }
    }

    fun getSelectedRecipient(): StateFlow<SelectedRecipientState> {
        // if we have not yet initialized, do so now in a coroutine
        if (selectedRecipient.value == Initializing) {
            viewModelScope.launch {
                initialize(
                    recipients = publicDataRepository.getAllJournalists(),
                    defaultRecipient = publicDataRepository.getDefaultJournalist()
                )
            }
        }
        return selectedRecipient.asStateFlow()
    }

    fun selectRecipient(recipient: JournalistInfo) {
        if (selectedRecipient.value is SingleRecipientForced) {
            return
        }
        selectedRecipient.value = SingleRecipientWithChoice(recipient)
    }

    fun forceResetToInitializing() {
        selectedRecipient.value = Initializing
    }

    @VisibleForTesting
    fun getInternalStateForTesting(): SelectedRecipientState {
        return selectedRecipient.value
    }
}
