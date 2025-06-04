package com.theguardian.coverdrop.ui.components

import android.view.KeyEvent
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.IconButton
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Text
import androidx.compose.material.TextField
import androidx.compose.material.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.focus.FocusDirection
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.key.onKeyEvent
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.SemanticsPropertyKey
import androidx.compose.ui.semantics.SemanticsPropertyReceiver
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import com.theguardian.coverdrop.core.ui.models.UiPassphrase
import com.theguardian.coverdrop.core.ui.models.UiPassphraseWord
import com.theguardian.coverdrop.ui.R
import com.theguardian.coverdrop.ui.theme.CoverDropPreviewSurface
import com.theguardian.coverdrop.ui.theme.NeutralBrighter
import com.theguardian.coverdrop.ui.theme.Padding
import com.theguardian.coverdrop.ui.theme.RoundedCornerShape
import com.theguardian.coverdrop.ui.theme.RoundedCorners
import com.theguardian.coverdrop.ui.theme.WarningPastelRed

/**
 * Password mask to be used for hidden password fields that are not user editable. For anything
 * that is user editable use [PasswordVisualTransformation].
 */
const val PASSPHRASE_MASK = "•••••••••"


/** A custom semantic key to allow for UI tests to assert the current hidden/revealed state */
val PassphraseWordHiddenKey = SemanticsPropertyKey<Boolean>("PasswordHidden")
var SemanticsPropertyReceiver.passphraseWordHidden by PassphraseWordHiddenKey

/** A custom semantic key to allow for UI tests to assert whether an entered word is invalid */
val PassphraseWordInvalidKey = SemanticsPropertyKey<Boolean>("PasswordInvalid")
var SemanticsPropertyReceiver.passphraseWordInvalid by PassphraseWordInvalidKey


@Composable
fun TextPassphraseColumn(
    passphrase: UiPassphrase,
    onPassphraseHidden: () -> Unit = {},
    onPassphraseRevealed: () -> Unit = {},
    showHideRevealButton: Boolean = true,
    clickOnWordRevealsPassphrase: Boolean = false,
) {
    val onClickAction = if (clickOnWordRevealsPassphrase) {
        { onPassphraseRevealed() }
    } else {
        null
    }

    Column {
        passphrase.forEach { word ->
            TextPassphraseWord(
                text = word.content,
                revealed = word.revealed,
                onClick = onClickAction,
                modifier = Modifier.padding(top = Padding.L),
            )
        }

        if (showHideRevealButton) {
            PassphraseHideRevealButton(
                isPassphraseRevealed = passphrase.any { it.revealed },
                hidePassphrase = onPassphraseHidden,
                revealPassphrase = onPassphraseRevealed,
                modifier = Modifier
                    .padding(top = Padding.M)
                    .align(Alignment.End)
            )
        }
    }
}

/**
 * A column of editable passphrase words with the ability to hide/reveal the entire passphrase.
 *
 * @param passphrase The passphrase to display and edit
 * @param enabled Whether the passphrase words should be editable
 * @param onPassphraseWordUpdated Callback for when a passphrase word is updated
 * @param onPassphraseWordHidden Callback for when a passphrase word should be hidden
 * @param onPassphraseWordRevealed Callback for when a passphrase word should be revealed
 * @param onPassphraseHidden Callback for when the entire passphrase is hidden
 * @param onPassphraseRevealed Callback for when the entire passphrase is revealed
 * @param focusNextAction Callback for when the next action should be focused
 */
@Composable
fun EditPassphraseColumn(
    passphrase: UiPassphrase,
    enabled: Boolean = true,
    onPassphraseWordUpdated: (Int, String) -> Unit = { _, _ -> },
    onPassphraseWordHidden: (Int) -> Unit = {},
    onPassphraseWordRevealed: (Int) -> Unit = {},
    onPassphraseHidden: () -> Unit = {},
    onPassphraseRevealed: () -> Unit = {},
    focusNextAction: () -> Boolean = { false },
) {
    val focusManager = LocalFocusManager.current
    Column {
        passphrase.forEachIndexed { index, word ->
            PassphraseWordEditField(
                passphraseWord = word,
                updatePassphraseWord = { onPassphraseWordUpdated(index, it) },
                showHideRevealIcons = true,
                hidePassphraseWord = { onPassphraseWordHidden(index) },
                revealPassphraseWord = { onPassphraseWordRevealed(index) },
                enabled = enabled,
                testTag = "passphrase_edit_$index",
                onDone = {
                    if (index < passphrase.size - 1) {
                        focusManager.moveFocus(FocusDirection.Down)
                    } else {
                        focusManager.clearFocus()
                        focusNextAction()
                    }
                },
                modifier = Modifier.padding(top = Padding.L),
            )
        }

        PassphraseHideRevealButton(
            isPassphraseRevealed = passphrase.any { it.revealed },
            hidePassphrase = onPassphraseHidden,
            revealPassphrase = onPassphraseRevealed,
            modifier = Modifier
                .padding(top = Padding.M)
                .align(Alignment.End)
        )
    }
}

@Composable
fun TextPassphraseWord(
    text: String,
    revealed: Boolean,
    modifier: Modifier = Modifier,
    onClick: (() -> Unit)? = null,
) {
    val dashedStroke = Stroke(
        width = 1f,
        pathEffect = PathEffect.dashPathEffect(floatArrayOf(10f, 10f), 0f)
    )

    val textModifier = if (onClick != null) {
        Modifier.clickable(onClick = onClick)
    } else {
        Modifier
    }

    Box(
        modifier = modifier
            .fillMaxWidth()
            .drawBehind {
                drawRoundRect(
                    color = NeutralBrighter,
                    style = dashedStroke,
                    cornerRadius = CornerRadius(
                        x = RoundedCorners.XS.toPx(),
                        y = RoundedCorners.XS.toPx()
                    )
                )
            },
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = if (revealed) {
                text
            } else {
                PASSPHRASE_MASK
            },
            modifier = textModifier
                .fillMaxWidth()
                .padding(Padding.M)
                .testTag("passphrase_box"),
            textAlign = TextAlign.Center,
            style = TextStyle(fontWeight = FontWeight.Bold, fontFamily = FontFamily.Monospace)
        )
    }
}

/**
 * A single passphrase word that can be edited.
 *
 * @param passphraseWord The passphrase word to display and edit
 * @param updatePassphraseWord Callback for when the passphrase word is updated
 * @param showHideRevealIcons Whether to show the hide/reveal icons
 * @param hidePassphraseWord Callback for when the passphrase word should be hidden
 * @param revealPassphraseWord Callback for when the passphrase word should be revealed
 * @param enabled Whether the passphrase word should be editable
 * @param onDone Callback for when the ENTER key is pressed
 */
@Composable
fun PassphraseWordEditField(
    passphraseWord: UiPassphraseWord,
    modifier: Modifier = Modifier,
    updatePassphraseWord: (String) -> Unit,
    showHideRevealIcons: Boolean = false,
    hidePassphraseWord: () -> Unit = {},
    revealPassphraseWord: () -> Unit = {},
    enabled: Boolean = true,
    onDone: () -> Boolean = { false },
    testTag: String = "passphrase_edit",
) {
    var text by rememberSaveable(stateSaver = TextFieldValue.Saver) {
        mutableStateOf(TextFieldValue(passphraseWord.content))
    }

    val foregroundColor = if (enabled) {
        MaterialTheme.colors.onBackground
    } else {
        MaterialTheme.colors.onSurface.copy(alpha = 0.5f)
    }

    TextField(
        value = text,
        onValueChange = { text = it; updatePassphraseWord(it.text) },
        singleLine = true,
        visualTransformation = if (passphraseWord.revealed) VisualTransformation.None else PasswordVisualTransformation(),
        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
        enabled = enabled,
        trailingIcon = {
            if (showHideRevealIcons) {
                if (passphraseWord.revealed) {
                    IconButton(
                        onClick = { hidePassphraseWord() },
                        modifier = Modifier.testTag(testTag + "_hide")
                    ) {
                        CoverDropIcons.Hide.AsComposable(
                            modifier = Modifier.size(24.dp),
                        )
                    }
                } else {
                    IconButton(
                        onClick = { revealPassphraseWord() },
                        modifier = Modifier.testTag(testTag + "_reveal")
                    ) {
                        CoverDropIcons.Reveal.AsComposable(
                            modifier = Modifier.size(24.dp),
                        )
                    }
                }
            }
        },
        colors = TextFieldDefaults.textFieldColors(
            backgroundColor = Color.Transparent,
            unfocusedIndicatorColor = Color.Transparent,
            focusedIndicatorColor = Color.Transparent,
        ),
        textStyle = TextStyle(
            fontWeight = FontWeight.Bold,
            fontFamily = FontFamily.Monospace,
            color = foregroundColor
        ),
        keyboardActions = KeyboardActions(onDone = { onDone() }),
        modifier = modifier
            .fillMaxWidth()
            .background(MaterialTheme.colors.surface)
            .border(
                width = 1.dp,
                color = if (passphraseWord.isValid) foregroundColor else WarningPastelRed,
                shape = RoundedCornerShape.XS,
            )
            .onKeyEvent { it.nativeKeyEvent.keyCode == KeyEvent.KEYCODE_ENTER && onDone() }
            .testTag(testTag)
            .semantics {
                passphraseWordHidden = !passphraseWord.revealed
                passphraseWordInvalid = !passphraseWord.isValid
            },
    )
}

@Composable
private fun PassphraseHideRevealButton(
    isPassphraseRevealed: Boolean,
    hidePassphrase: () -> Unit,
    revealPassphrase: () -> Unit,
    modifier: Modifier = Modifier
) {
    if (isPassphraseRevealed) {
        FlatTextButton(
            text = stringResource(R.string.component_passphrases_button_hide_passphrase),
            icon = CoverDropIcons.Hide,
            modifier = modifier,
            onClick = hidePassphrase,
        )
    } else {
        FlatTextButton(
            text = stringResource(R.string.component_passphrases_button_reveal_passphrase),
            icon = CoverDropIcons.Reveal,
            modifier = modifier,
            onClick = revealPassphrase,
        )
    }
}

@Preview
@Composable
private fun TextPassphraseWordPreview_revealed() = CoverDropPreviewSurface {
    TextPassphraseWord(text = "foobar", revealed = true)
}

@Preview
@Composable
private fun TextPassphraseWordPreview_hidden() = CoverDropPreviewSurface {
    TextPassphraseWord(text = "foobar", revealed = false)
}


@Preview
@Composable
fun PassphraseWordEditFieldPreview_hidden() = CoverDropPreviewSurface {
    PassphraseWordEditField(
        passphraseWord = UiPassphraseWord("word", false),
        updatePassphraseWord = {},
        showHideRevealIcons = true,
    )
}

@Preview
@Composable
fun PassphraseWordEditFieldPreview_revealed() = CoverDropPreviewSurface {
    PassphraseWordEditField(
        passphraseWord = UiPassphraseWord("word", true),
        updatePassphraseWord = {},
        showHideRevealIcons = true,
    )
}

@Preview
@Composable
fun PassphraseWordEditFieldPreview_invalidWord() = CoverDropPreviewSurface {
    PassphraseWordEditField(
        passphraseWord = UiPassphraseWord(content = "badword", revealed = true, isValid = false),
        updatePassphraseWord = {},
        showHideRevealIcons = true,
    )
}

@Preview
@Composable
fun PassphraseWordEditFieldPreview_noActionIcon() = CoverDropPreviewSurface {
    PassphraseWordEditField(
        passphraseWord = UiPassphraseWord("word", true),
        updatePassphraseWord = {},
        showHideRevealIcons = false,
    )
}

@Preview
@Composable
fun PassphraseHideRevealButtonPreview_revealed() = CoverDropPreviewSurface {
    PassphraseHideRevealButton(
        isPassphraseRevealed = true,
        hidePassphrase = {},
        revealPassphrase = {},
    )
}

@Preview
@Composable
fun PassphraseHideRevealButtonPreview_hidden() = CoverDropPreviewSurface {
    PassphraseHideRevealButton(
        isPassphraseRevealed = false,
        hidePassphrase = {},
        revealPassphrase = {},
    )
}

@Preview
@Composable
fun TextPassphraseColumnPreview() = CoverDropPreviewSurface {
    TextPassphraseColumn(
        passphrase = listOf(
            UiPassphraseWord(content = "word1", revealed = true),
            UiPassphraseWord(content = "word2", revealed = false),
            UiPassphraseWord(content = "word3", revealed = true),
        )
    )
}

@Preview
@Composable
fun EditPassphraseColumnPreview() = CoverDropPreviewSurface {
    EditPassphraseColumn(
        passphrase = listOf(
            UiPassphraseWord(content = "word1", revealed = true),
            UiPassphraseWord(content = "word2", revealed = false),
            UiPassphraseWord(content = "word3", revealed = true, isValid = false),
        )
    )
}
