package com.theguardian.coverdrop.ui.screens

import androidx.compose.foundation.focusable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.Divider
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.navigation.NavHostController
import androidx.navigation.compose.rememberNavController
import com.theguardian.coverdrop.core.ui.models.UiPassphrase
import com.theguardian.coverdrop.core.ui.models.UiPassphraseWord
import com.theguardian.coverdrop.core.ui.models.toUiPassphrase
import com.theguardian.coverdrop.ui.R
import com.theguardian.coverdrop.ui.components.ChevronTextButton
import com.theguardian.coverdrop.ui.components.CoverDropConfirmationDialog
import com.theguardian.coverdrop.ui.components.CoverDropIcons
import com.theguardian.coverdrop.ui.components.CoverDropTopAppBar
import com.theguardian.coverdrop.ui.components.EditPassphraseColumn
import com.theguardian.coverdrop.ui.components.ErrorMessageWithIcon
import com.theguardian.coverdrop.ui.components.PrimaryButton
import com.theguardian.coverdrop.ui.components.ProgressSpinnerWithText
import com.theguardian.coverdrop.ui.components.TopBarNavigationOption
import com.theguardian.coverdrop.ui.navigation.CoverDropDestinations
import com.theguardian.coverdrop.ui.theme.CoverDropSurface
import com.theguardian.coverdrop.ui.theme.NeutralMiddle
import com.theguardian.coverdrop.ui.theme.Padding
import com.theguardian.coverdrop.ui.utils.COVERDROP_SAMPLE_DATA
import com.theguardian.coverdrop.ui.utils.ScreenContentWrapper
import com.theguardian.coverdrop.ui.utils.UiErrorMessage
import com.theguardian.coverdrop.ui.utils.popBackStackAndThenNavigate
import com.theguardian.coverdrop.ui.utils.rememberScreenInsets
import com.theguardian.coverdrop.ui.utils.shouldUiBeEnabled
import com.theguardian.coverdrop.ui.viewmodels.ContinueSessionState
import com.theguardian.coverdrop.ui.viewmodels.ContinueSessionViewModel

@Composable
fun ContinueSessionRoute(
    navController: NavHostController,
) {
    val viewModel = hiltViewModel<ContinueSessionViewModel>()

    val passphraseWords = viewModel.getPassphraseWords().map { it.collectAsState() }
    val screenState = viewModel.getScreenState().collectAsState()
    val errorMessage = viewModel.getErrorMessage().collectAsState()

    ContinueSessionScreen(
        navController = navController,
        screenState = screenState.value,
        passphraseWords = passphraseWords.map { it.value },
        errorMessage = errorMessage.value,
        updatePassphraseWord = { pos, s -> viewModel.updatePassphraseWord(pos, s) },
        revealPassphraseWord = { pos -> viewModel.revealPassphraseWord(pos) },
        hidePassphraseWord = { pos -> viewModel.hidePassphraseWord(pos) },
        revealPassphrase = { viewModel.revealPassphrase() },
        hidePassphrase = { viewModel.hidePassphrase() },
        unlock = { viewModel.unlockStorage() },
    )
}


@Composable
fun ContinueSessionScreen(
    navController: NavHostController,
    screenState: ContinueSessionState,
    passphraseWords: UiPassphrase,
    errorMessage: UiErrorMessage? = null,
    updatePassphraseWord: (Int, String) -> Unit = { _, _ -> },
    revealPassphraseWord: (Int) -> Unit = { },
    hidePassphraseWord: (Int) -> Unit = { },
    revealPassphrase: () -> Unit = {},
    hidePassphrase: () -> Unit = {},
    unlock: () -> Unit = {},
    showGetStartedDialogInitialValue: Boolean = false,
) {
    var showGetStartedDialog by rememberSaveable { mutableStateOf(showGetStartedDialogInitialValue) }
    if (showGetStartedDialog) {
        CoverDropConfirmationDialog(
            headingText = stringResource(R.string.dialog_confirm_new_conversation_heading),
            bodyText = stringResource(R.string.dialog_confirm_new_conversation_content),
            confirmText = stringResource(R.string.dialog_confirm_new_conversation_button_yes),
            onConfirmClick = {
                showGetStartedDialog = false

                // navigate up to the entry screen before changing to the "new session" flow
                // which prevents cycles in the navigation stack if the user would later
                // click "I already have a passphrase" button.
                navController.popBackStackAndThenNavigate(
                    popUpTo = CoverDropDestinations.ENTRY_ROUTE,
                    destination = CoverDropDestinations.HOW_THIS_WORKS_ROUTE,
                )
            },
            dismissText = stringResource(R.string.dialog_confirm_new_conversation_button_no),
            onDismissClick = { showGetStartedDialog = false },
        )
    }

    ScreenContentWrapper {
        Column(modifier = Modifier.fillMaxHeight(1f)) {
            CoverDropTopAppBar(
                navigationOption = if (screenState != ContinueSessionState.UNLOCKING_STORAGE) {
                    TopBarNavigationOption.Back
                } else {
                    TopBarNavigationOption.None
                },
                onNavigationOptionPressed = { navController.navigateUp() }
            )

            Column(
                modifier = Modifier
                    .verticalScroll(rememberScrollState())
                    .weight(1f)
                    .padding(bottom = rememberScreenInsets().bottom)
            ) {
                Column(
                    modifier = Modifier
                        .verticalScroll(rememberScrollState())
                        .padding(top = Padding.L, start = Padding.L, end = Padding.L)
                        .height(IntrinsicSize.Max)
                        .weight(1f)
                ) {
                    when (screenState) {
                        ContinueSessionState.ENTERING_PASSPHRASE -> {
                            ContentConfirmingPassphrase(
                                passphraseWords = passphraseWords,
                                errorMessage = errorMessage,
                                updatePassphraseWord = updatePassphraseWord,
                                revealPassphraseWord = revealPassphraseWord,
                                hidePassphraseWord = hidePassphraseWord,
                                revealPassphrase = revealPassphrase,
                                hidePassphrase = hidePassphrase,
                                unlock = unlock,
                                onStartNewSession = { showGetStartedDialog = true }
                            )
                        }

                        ContinueSessionState.UNLOCKING_STORAGE -> {
                            ContentUnlockingStorage()
                        }

                        ContinueSessionState.FINISHED -> {
                            // we pop up to the entry screen to ensure the user cannot accidentally go back
                            // to this screen (which might then only show an indefinite progress spinner)
                            LaunchedEffect(true) {
                                navController.popBackStackAndThenNavigate(
                                    popUpTo = CoverDropDestinations.ENTRY_ROUTE,
                                    destination = CoverDropDestinations.INBOX_ROUTE,
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun ContentConfirmingPassphrase(
    passphraseWords: UiPassphrase,
    errorMessage: UiErrorMessage? = null,
    updatePassphraseWord: (Int, String) -> Unit = { _, _ -> },
    revealPassphraseWord: (Int) -> Unit = { },
    hidePassphraseWord: (Int) -> Unit = { },
    revealPassphrase: () -> Unit = {},
    hidePassphrase: () -> Unit = {},
    unlock: () -> Unit = {},
    onStartNewSession: () -> Unit = {},
) {
    val allWordsEntered = passphraseWords.all { it.content.isNotEmpty() }
    val focusManager = LocalFocusManager.current
    val focusRequester = FocusRequester()

    Column {
        Text(
            text = stringResource(R.string.screen_continue_header_enter_passphrase),
            style = MaterialTheme.typography.h1,
        )
        Text(
            text = stringResource(R.string.screen_continue_text_explanation_remember_passphrase),
            modifier = Modifier.padding(top = Padding.M),
        )

        errorMessage?.run {
            Spacer(modifier = Modifier.height(Padding.M))
            ErrorMessageWithIcon(
                text = getString(LocalContext.current),
                icon = CoverDropIcons.Warning
            )
        }

        EditPassphraseColumn(
            passphrase = passphraseWords,
            enabled = errorMessage.shouldUiBeEnabled(),
            onPassphraseWordUpdated = updatePassphraseWord,
            onPassphraseWordRevealed = revealPassphraseWord,
            onPassphraseWordHidden = hidePassphraseWord,
            onPassphraseRevealed = revealPassphrase,
            onPassphraseHidden = hidePassphrase,
            focusNextAction = { focusRequester.requestFocus(); true }
        )

        Spacer(
            modifier = Modifier
                .weight(1f)
                .padding(top = Padding.L)
        )

        PrimaryButton(
            text = stringResource(R.string.screen_continue_button_confirm_passphrase),
            enabled = allWordsEntered && errorMessage.shouldUiBeEnabled(),
            onClick = {
                focusManager.clearFocus()
                unlock()
            },
            modifier = Modifier
                .fillMaxWidth(1f)
                .focusRequester(focusRequester)
                .focusable()
        )

        Divider(
            color = NeutralMiddle,
            modifier = Modifier.padding(top = Padding.M)
        )

        ChevronTextButton(
            text = stringResource(R.string.screen_continue_button_i_do_not_have_a_passphrase_yet),
            onClick = onStartNewSession,
            modifier = Modifier.padding(start = Padding.M, end = Padding.M)
        )
    }
}

@Composable
private fun ContentUnlockingStorage() {
    Column(
        modifier = Modifier
            .fillMaxSize(1f)
            .padding(Padding.L),
        verticalArrangement = Arrangement.Center,
    ) {
        ProgressSpinnerWithText(
            stringResource(R.string.screen_continue_text_unlocking_storage_please_wait)
        )
    }
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun ContinueSessionPreview_withError() {
    CoverDropSurface {
        ContinueSessionScreen(
            navController = rememberNavController(),
            screenState = ContinueSessionState.ENTERING_PASSPHRASE,
            passphraseWords = COVERDROP_SAMPLE_DATA.getShortPassphrase().toUiPassphrase(),
            errorMessage = COVERDROP_SAMPLE_DATA.getSampleErrorMessage(isFatal = false),
        )
    }
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun ContinueSessionPreview_withErrorFatal() {
    CoverDropSurface {
        ContinueSessionScreen(
            navController = rememberNavController(),
            screenState = ContinueSessionState.ENTERING_PASSPHRASE,
            passphraseWords = COVERDROP_SAMPLE_DATA.getShortPassphrase().toUiPassphrase(),
            errorMessage = COVERDROP_SAMPLE_DATA.getSampleErrorMessage(isFatal = true),
        )
    }
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun ContinueSessionPreview_withSomeWordsEntered() {
    val passphraseWords = COVERDROP_SAMPLE_DATA.getShortPassphrase().toUiPassphrase()

    CoverDropSurface {
        ContinueSessionScreen(
            navController = rememberNavController(),
            screenState = ContinueSessionState.ENTERING_PASSPHRASE,
            passphraseWords = listOf(
                passphraseWords[0].copyRevealed().copy(isValid = false),
                passphraseWords[1].copy(isValid = false),
                passphraseWords[2],
                passphraseWords[3].copyTextChanged(""),
            ),
        )
    }
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun ContinueSessionPreview_withLongPassphrase() {
    val passphraseWords = COVERDROP_SAMPLE_DATA.getLongPassphrase()
        .toUiPassphrase()
        .toMutableList()

    CoverDropSurface {
        ContinueSessionScreen(
            navController = rememberNavController(),
            screenState = ContinueSessionState.ENTERING_PASSPHRASE,
            passphraseWords = passphraseWords,
        )
    }
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun ContinueSessionPreview_showDialog() {
    CoverDropSurface {
        ContinueSessionScreen(
            navController = rememberNavController(),
            screenState = ContinueSessionState.ENTERING_PASSPHRASE,
            passphraseWords = COVERDROP_SAMPLE_DATA.getShortPassphrase().toUiPassphrase(),
            showGetStartedDialogInitialValue = true,
        )
    }
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun ContinueSessionPreview_whileUnlocking() {
    val passphraseWords = List(4) { UiPassphraseWord("") }

    CoverDropSurface {
        ContinueSessionScreen(
            navController = rememberNavController(),
            screenState = ContinueSessionState.UNLOCKING_STORAGE,
            passphraseWords = passphraseWords,
        )
    }
}
