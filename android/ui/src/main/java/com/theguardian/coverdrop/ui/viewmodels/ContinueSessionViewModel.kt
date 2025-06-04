package com.theguardian.coverdrop.ui.viewmodels

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.theguardian.coverdrop.core.ICoverDropLib
import com.theguardian.coverdrop.core.ICoverDropPrivateDataRepository
import com.theguardian.coverdrop.core.crypto.Passphrase
import com.theguardian.coverdrop.core.encryptedstorage.EncryptedStorageAuthenticationFailed
import com.theguardian.coverdrop.core.encryptedstorage.EncryptedStorageBadPassphraseException
import com.theguardian.coverdrop.core.ui.models.UiPassphraseWord
import com.theguardian.coverdrop.ui.R
import com.theguardian.coverdrop.ui.utils.UiErrorMessage
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import javax.inject.Inject


enum class ContinueSessionState {
    ENTERING_PASSPHRASE,
    UNLOCKING_STORAGE,
    FINISHED,
}

@HiltViewModel
class ContinueSessionViewModel @Inject constructor(
    application: Application,
    lib: ICoverDropLib,
    private val privateDataRepository: ICoverDropPrivateDataRepository,
) : AndroidViewModel(application) {
    private val passphraseWordList = lib.getPassphraseWordList()

    /** Current screen state to be shown: see [ContinueSessionState] */
    private val screenState = MutableStateFlow(ContinueSessionState.ENTERING_PASSPHRASE)

    // Using a List<StateFlow> here is fine as the number of words does not change
    private val currentPassphraseWords: List<MutableStateFlow<UiPassphraseWord>> =
        List(privateDataRepository.getPassphraseWordCount()) {
            MutableStateFlow(UiPassphraseWord(content = "", revealed = true))
        }

    private val errorMessage = MutableStateFlow<UiErrorMessage?>(null)

    fun getScreenState() = screenState
    fun getPassphraseWords() = currentPassphraseWords
    fun getErrorMessage() = errorMessage

    init {
        // Prepare the prefixes for the passphrase word list to avoid blocking the UI thread when
        // entering the first word
        passphraseWordList.preparePrefixes()
    }

    fun revealPassphraseWord(position: Int) {
        val newWord = currentPassphraseWords[position].value.copyRevealed()
        currentPassphraseWords[position].value = newWord
    }

    fun hidePassphraseWord(position: Int) {
        val newWord = currentPassphraseWords[position].value.copyHidden()
        currentPassphraseWords[position].value = newWord
    }

    fun revealPassphrase() {
        currentPassphraseWords.forEachIndexed { index, _ -> revealPassphraseWord(index) }
    }

    fun hidePassphrase() {
        currentPassphraseWords.forEachIndexed { index, _ -> hidePassphraseWord(index) }
    }

    fun updatePassphraseWord(position: Int, newContent: String) {
        val isValidPrefix = passphraseWordList.isValidPrefix(newContent)
        val newWord = currentPassphraseWords[position].value.copyTextChanged(
            newContent = newContent,
            isValid = isValidPrefix
        )
        currentPassphraseWords[position].value = newWord
    }

    fun unlockStorage() {
        require(screenState.value == ContinueSessionState.ENTERING_PASSPHRASE)

        viewModelScope.launch {
            // check that the user filled out all words
            val allWordsEntered = currentPassphraseWords.all { it.value.content.isNotEmpty() }
            if (!allWordsEntered) {
                screenState.value = ContinueSessionState.ENTERING_PASSPHRASE
                setErrorMessage(R.string.screen_continue_text_error_not_all_words_entered)
                return@launch
            }

            screenState.value = ContinueSessionState.UNLOCKING_STORAGE
            clearErrorMessage()

            // Fail-safe to ensure that the app is in a lock state.
            privateDataRepository.lock()

            try {
                withContext(Dispatchers.Default) {
                    val passphrase = Passphrase(currentPassphraseWords.map {
                        it.value.content.toCharArray()
                    })
                    try {
                        privateDataRepository.unlock(passphrase)
                    } finally {
                        passphrase.clear()
                    }
                }
                screenState.value = ContinueSessionState.FINISHED
            } catch (e: EncryptedStorageBadPassphraseException) {
                screenState.value = ContinueSessionState.ENTERING_PASSPHRASE
                setErrorMessage(R.string.screen_continue_text_error_bad_passphrase)
            } catch (e: EncryptedStorageAuthenticationFailed) {
                screenState.value = ContinueSessionState.ENTERING_PASSPHRASE
                setErrorMessage(R.string.screen_continue_text_error_authentication_failed)
            } catch (e: Exception) {
                screenState.value = ContinueSessionState.ENTERING_PASSPHRASE
                setErrorMessage(
                    messageResId = R.string.screen_continue_text_error_unknown,
                    isFatal = true,
                )
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
