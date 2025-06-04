package com.theguardian.coverdrop.ui.viewmodels

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.theguardian.coverdrop.core.ICoverDropLib
import com.theguardian.coverdrop.core.models.JournalistInfo
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

internal enum class RecipientSelectionState {
    SHOWING_SELECTION,
    CONFIRM_TEAM,
}

internal data class TeamCardInfo(
    val id: String,
    val displayName: String,
    val description: String = "",
)

internal data class JournalistCardInfo(
    val id: String,
    val displayName: String,
    val tagLine: String = "",
)

internal fun JournalistInfo.toTeamsCardInfo() = TeamCardInfo(id, displayName, description)
internal fun JournalistInfo.toJournalistCardInfo() =
    JournalistCardInfo(id, displayName, description)

@HiltViewModel
internal class RecipientSelectionViewModel @Inject constructor(
    application: Application,
    private val coverDrop: ICoverDropLib,
) : AndroidViewModel(application) {

    private val liveRecipients = MutableStateFlow<List<JournalistInfo>>(emptyList())

    init {
        getAllRecipients()
    }

    private fun getAllRecipients() {
        viewModelScope.launch(Dispatchers.Default) {
            liveRecipients.value = coverDrop.getPublicDataRepository()
                .getAllJournalists()
                .sortedBy { it.sortName }
        }
    }

    private val liveTeams = liveRecipients.map { recipients ->
        recipients.filter { it.isTeam }.map { ri -> ri.toTeamsCardInfo() }
    }

    private val liveJournalists = liveRecipients.map { recipients ->
        recipients.filter { !it.isTeam }.map { it.toJournalistCardInfo() }
    }

    val teams = liveTeams
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(),
            initialValue = emptyList()
        )

    val journalists = liveJournalists
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(),
            initialValue = emptyList()
        )

    private val mutableScreenState = MutableStateFlow(RecipientSelectionState.SHOWING_SELECTION)
    val screenState = mutableScreenState.asStateFlow()

    private val mutableSelectedTeam = MutableStateFlow<TeamCardInfo?>(null)
    val selectedTeam = mutableSelectedTeam.asStateFlow()


    fun selectTeam(id: String) {
        val selectedRecipient = getRecipientInfo(id) ?: return
        mutableSelectedTeam.value = selectedRecipient.toTeamsCardInfo()
        mutableScreenState.value = RecipientSelectionState.CONFIRM_TEAM
    }

    fun confirmRecipient(
        id: String,
        outputViewModel: SelectedRecipientViewModel,
        onFinished: () -> Unit = {},
    ) {
        val confirmedRecipient = getRecipientInfo(id) ?: return
        outputViewModel.selectRecipient(confirmedRecipient)
        onFinished()
    }

    fun backToList() {
        mutableScreenState.value = RecipientSelectionState.SHOWING_SELECTION
    }

    private fun getRecipientInfo(id: String): JournalistInfo? {
        return liveRecipients.value.firstOrNull { it.id == id }
    }
}
