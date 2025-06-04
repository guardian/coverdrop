package com.theguardian.coverdrop.ui.viewmodels

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.theguardian.coverdrop.core.ICoverDropLib
import com.theguardian.coverdrop.core.ICoverDropPrivateDataRepository
import com.theguardian.coverdrop.core.crypto.Passphrase
import com.theguardian.coverdrop.core.ui.models.UiPassphrase
import com.theguardian.coverdrop.core.ui.models.UiPassphraseWord
import com.theguardian.coverdrop.core.ui.models.toUiPassphrase
import com.theguardian.coverdrop.ui.R
import com.theguardian.coverdrop.ui.utils.UiErrorMessage
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import javax.inject.Inject

/**
 * The user is moving through these states in linear fashion:
 * - Start state is `SHOWING_PASSPHRASE` (allowing to reveal and hide)
 * - Confirmation then leads to `CONFIRMING_PASSPHRASE`
 * - Entering the correct passphrase and clicking triggers `CREATING_STORAGE`
 * - And finally, the state `FINISHED` triggers the navigation to the new message screen.
 *
 * Errors or dedicated navigation (e.g. clicking "hide") lead to either the same state with an
 * error message or the previous state.
 */
enum class NewSessionState {
    SHOWING_PASSPHRASE,
    CONFIRMING_PASSPHRASE,
    CREATING_STORAGE,
    FINISHED,
}

@HiltViewModel
class NewSessionViewModel @Inject constructor(
    application: Application,
    lib: ICoverDropLib,
    private val privateDataRepository: ICoverDropPrivateDataRepository,
) : AndroidViewModel(application) {
    private val passphraseWordList = lib.getPassphraseWordList()

    /** Current screen state to be shown: see [NewSessionState] */
    private val screenState = MutableStateFlow(NewSessionState.SHOWING_PASSPHRASE)

    /** The generated passphrase (computed not more than once). Might be initially null. */
    private var generatedPassphrase: Passphrase? = null
    private val generatedUiPassphrase = MutableStateFlow<UiPassphrase?>(null)

    /** The passphrase as entered by the user during the confirmation step */
    private val enteredPassphraseWords: List<MutableStateFlow<UiPassphraseWord>> =
        List(privateDataRepository.getPassphraseWordCount()) {
            MutableStateFlow(UiPassphraseWord(content = "", revealed = true))
        }

    /** Potential error message to be shown when the user confirms the passphrase */
    private val errorMessage = MutableStateFlow<UiErrorMessage?>(null)

    fun getScreenState() = screenState
    fun getEnteredPassphraseWords() = enteredPassphraseWords
    fun getErrorMessage() = errorMessage
    fun getGeneratedPassphrase() = generatedUiPassphrase

    init {
        generateNewPassphrase()
    }

    //
    // Operations
    //

    /** Reveals the entire passphrase (either the generated one and the entered one) */
    fun revealPassphrase() {
        if (screenState.value == NewSessionState.SHOWING_PASSPHRASE) {
            generatedUiPassphrase.value?.let {
                generatedUiPassphrase.value = it.map { word -> word.copyRevealed() }
            }
        } else if (screenState.value == NewSessionState.CONFIRMING_PASSPHRASE) {
            enteredPassphraseWords.forEachIndexed { index, _ -> revealPassphraseWord(index) }
        }
    }

    /** Hides the entire passphrase (both the generated one and the entered one) */
    fun hidePassphrase() {
        if (screenState.value == NewSessionState.SHOWING_PASSPHRASE) {
            generatedUiPassphrase.value?.let {
                generatedUiPassphrase.value = it.map { word -> word.copyHidden() }
            }
        } else if (screenState.value == NewSessionState.CONFIRMING_PASSPHRASE) {
            enteredPassphraseWords.forEachIndexed { index, _ -> hidePassphraseWord(index) }
        }
    }

    /** Reveals a single word of the passphrase on the confirmation screen */
    fun revealPassphraseWord(position: Int) {
        val newWord = enteredPassphraseWords[position].value.copyRevealed()
        enteredPassphraseWords[position].value = newWord
    }

    /** Hides a single word of the passphrase on the confirmation screen */
    fun hidePassphraseWord(position: Int) {
        val newWord = enteredPassphraseWords[position].value.copyHidden()
        enteredPassphraseWords[position].value = newWord
    }

    fun advanceToConfirmation() {
        require(screenState.value == NewSessionState.SHOWING_PASSPHRASE)
        require(generatedUiPassphrase.value?.all { it.revealed } ?: false)
        screenState.value = NewSessionState.CONFIRMING_PASSPHRASE
    }

    fun updatePassphraseWord(position: Int, newContent: String) {
        val isValidPrefix = passphraseWordList.isValidPrefix(newContent)
        val newWord = enteredPassphraseWords[position].value.copyTextChanged(
            newContent = newContent,
            isValid = isValidPrefix
        )
        enteredPassphraseWords[position].value = newWord
    }

    fun confirmPassphraseAndCreateStorage() {
        require(screenState.value == NewSessionState.CONFIRMING_PASSPHRASE)

        screenState.value = NewSessionState.CREATING_STORAGE
        clearErrorMessage()

        viewModelScope.launch {
            // check that the user filled out all words
            val allWordsEntered = enteredPassphraseWords.all {
                it.value.content.isNotEmpty()
            }
            if (!allWordsEntered) {
                screenState.value = NewSessionState.CONFIRMING_PASSPHRASE
                setErrorMessage(R.string.screen_new_session_text_error_not_all_words_entered)
                return@launch
            }

            // check that the user passphrase is identical with the one generated
            val enteredPassphrase = Passphrase(enteredPassphraseWords.map {
                it.value.content.toCharArray()
            })
            try {
                val enteredPassphraseWords = enteredPassphraseWords.map { it.value.content }
                val generatedPassphraseWords =
                    generatedPassphrase?.getWords()?.map { it.concatToString() }

                if (enteredPassphraseWords != generatedPassphraseWords) {
                    screenState.value = NewSessionState.CONFIRMING_PASSPHRASE
                    setErrorMessage(R.string.screen_new_session_error_entered_passphrase_does_not_match)
                    return@launch
                }

                // then create the storage
                try {
                    withContext(Dispatchers.Default) {
                        privateDataRepository.createOrResetStorage(generatedPassphrase!!)
                    }
                } catch (e: Exception) {
                    screenState.value = NewSessionState.CONFIRMING_PASSPHRASE
                    setErrorMessage(
                        messageResId = R.string.screen_new_session_text_error_unknown,
                        isFatal = true,
                    )
                    return@launch
                }
                screenState.value = NewSessionState.FINISHED
            } finally {
                enteredPassphrase.clear()
            }

        }
    }

    private fun generateNewPassphrase() {
        viewModelScope.launch {
            withContext(Dispatchers.Default) {
                // Prepare the prefixes for the passphrase word list to avoid blocking the UI thread when
                // entering the first word
                passphraseWordList.preparePrefixes()

                generatedPassphrase = privateDataRepository.generatePassphrase()
                generatedUiPassphrase.value = generatedPassphrase!!.toUiPassphrase()
            }
        }
    }

    private fun clearErrorMessage() {
        errorMessage.value = null
    }

    private fun setErrorMessage(messageResId: Int, isFatal: Boolean = false) {
        errorMessage.value = UiErrorMessage(messageResId = messageResId, isFatal = isFatal)
    }
}
