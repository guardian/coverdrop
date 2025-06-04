package com.theguardian.coverdrop.core.ui.models

import com.theguardian.coverdrop.core.crypto.Passphrase

typealias UiPassphrase = List<UiPassphraseWord>

/**
 * An immutable representation of a single word within a passphrase for UI modelling.
 *
 * @param content The content of the word.
 * @param revealed Whether the word is revealed or not.
 * @param isValid Whether the word is valid or not. This one is set to true when [content] is not
 * a valid prefix in the word list.
 */
data class UiPassphraseWord(
    val content: String = "",
    val revealed: Boolean = false,
    val isValid: Boolean = true
) {
    fun copyRevealed() = this.copy(revealed = true)
    fun copyHidden() = this.copy(revealed = false)

    fun copyTextChanged(newContent: String, isValid: Boolean = true) = this.copy(
        content = newContent,
        isValid = isValid
    )
}

fun Passphrase.toUiPassphrase(): UiPassphrase {
    return this.getWords().map { UiPassphraseWord(it.concatToString()) }
}
