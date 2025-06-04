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
import androidx.compose.material.CircularProgressIndicator
import androidx.compose.material.Divider
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.navigation.NavHostController
import androidx.navigation.compose.rememberNavController
import com.theguardian.coverdrop.core.ui.models.UiPassphrase
import com.theguardian.coverdrop.core.ui.models.toUiPassphrase
import com.theguardian.coverdrop.ui.R
import com.theguardian.coverdrop.ui.components.ChevronTextButton
import com.theguardian.coverdrop.ui.components.CoverDropIcons
import com.theguardian.coverdrop.ui.components.CoverDropTopAppBar
import com.theguardian.coverdrop.ui.components.EditPassphraseColumn
import com.theguardian.coverdrop.ui.components.ErrorMessageWithIcon
import com.theguardian.coverdrop.ui.components.PrimaryButton
import com.theguardian.coverdrop.ui.components.ProgressSpinnerWithText
import com.theguardian.coverdrop.ui.components.TextPassphraseColumn
import com.theguardian.coverdrop.ui.components.TopBarNavigationOption
import com.theguardian.coverdrop.ui.components.TwoLineBanner
import com.theguardian.coverdrop.ui.navigation.CoverDropDestinations
import com.theguardian.coverdrop.ui.theme.CoverDropSurface
import com.theguardian.coverdrop.ui.theme.NeutralMiddle
import com.theguardian.coverdrop.ui.theme.Padding
import com.theguardian.coverdrop.ui.utils.COVERDROP_SAMPLE_DATA
import com.theguardian.coverdrop.ui.utils.UiErrorMessage
import com.theguardian.coverdrop.ui.utils.highlightText
import com.theguardian.coverdrop.ui.utils.popBackStackAndThenNavigate
import com.theguardian.coverdrop.ui.utils.shouldUiBeEnabled
import com.theguardian.coverdrop.ui.viewmodels.NewSessionState
import com.theguardian.coverdrop.ui.viewmodels.NewSessionViewModel

@Composable
fun NewSessionRoute(
    navController: NavHostController,
) {
    val viewModel = hiltViewModel<NewSessionViewModel>()

    val screenState = viewModel.getScreenState().collectAsState()
    val generatedPassphrase = viewModel.getGeneratedPassphrase().collectAsState()
    val enteredPassphraseWords = viewModel.getEnteredPassphraseWords().map { it.collectAsState() }
    val errorMessage = viewModel.getErrorMessage().collectAsState()

    NewSessionScreen(
        // general
        navController = navController,
        screenState = screenState.value,
        revealPassphrase = { viewModel.revealPassphrase() },
        hidePassphrase = { viewModel.hidePassphrase() },
        // for showing the generated passphrase
        generatedPassphrase = generatedPassphrase.value,
        advanceToConfirmation = { viewModel.advanceToConfirmation() },
        // for confirming the passphrase
        enteredPassphraseWords = enteredPassphraseWords.map { it.value },
        errorMessage = errorMessage.value,
        updatePassphraseWord = viewModel::updatePassphraseWord,
        revealPassphraseWord = { viewModel.revealPassphraseWord(it) },
        hidePassphraseWord = { viewModel.hidePassphraseWord(it) },
        confirmPassphraseAndCreateStorage = { viewModel.confirmPassphraseAndCreateStorage() },
    )
}

@Composable
fun NewSessionScreen(
    // general
    navController: NavHostController,
    screenState: NewSessionState,
    revealPassphrase: () -> Unit = {},
    hidePassphrase: () -> Unit = {},
    // for showing the generated passphrase
    generatedPassphrase: UiPassphrase? = null,
    advanceToConfirmation: () -> Unit = {},
    // for confirming the passphrase
    enteredPassphraseWords: UiPassphrase = emptyList(),
    errorMessage: UiErrorMessage? = null,
    updatePassphraseWord: (Int, String) -> Unit = { _, _ -> },
    revealPassphraseWord: (Int) -> Unit = { },
    hidePassphraseWord: (Int) -> Unit = { },
    confirmPassphraseAndCreateStorage: () -> Unit = {},
) {
    Column(modifier = Modifier.fillMaxHeight(1f)) {
        CoverDropTopAppBar(
            navigationOption = if (screenState != NewSessionState.CREATING_STORAGE) {
                TopBarNavigationOption.Back
            } else {
                TopBarNavigationOption.None
            },
            onNavigationOptionPressed = { navController.navigateUp() }
        )

        when (screenState) {
            NewSessionState.SHOWING_PASSPHRASE -> HelpBannerPassphrase(navController)
            NewSessionState.CONFIRMING_PASSPHRASE -> {}
            NewSessionState.CREATING_STORAGE -> {}
            NewSessionState.FINISHED -> {}
        }

        // main content in a scrollable container
        Column(
            modifier = Modifier
                .verticalScroll(rememberScrollState())
                .weight(1f)
        ) {
            Column(
                modifier = Modifier
                    .verticalScroll(rememberScrollState())
                    .padding(top = Padding.L, start = Padding.L, end = Padding.L)
                    .height(IntrinsicSize.Max)
                    .weight(1f)
            ) {
                when (screenState) {
                    NewSessionState.SHOWING_PASSPHRASE -> {
                        ContentShowPassphrase(
                            generatedPassphrase = generatedPassphrase,
                            hidePassphrase = hidePassphrase,
                            revealPassphrase = revealPassphrase,
                        )
                        Spacer(
                            modifier = Modifier
                                .weight(1f)
                                .padding(top = Padding.L)
                        )
                        val isRevealed = generatedPassphrase?.any { it.revealed } ?: false
                        ButtonsForShowingPassphrase(
                            navController = navController,
                            isPassphraseRevealed = isRevealed,
                            revealPassphrase = revealPassphrase,
                            advanceToConfirmation = advanceToConfirmation
                        )
                    }

                    NewSessionState.CONFIRMING_PASSPHRASE -> {
                        val focusRequester = FocusRequester()
                        ContentConfirmPassphrase(
                            passphrase = enteredPassphraseWords,
                            errorMessage = errorMessage,
                            updatePassphraseWord = updatePassphraseWord,
                            revealPassphraseWord = revealPassphraseWord,
                            hidePassphraseWord = hidePassphraseWord,
                            revealPassphrase = revealPassphrase,
                            hidePassphrase = hidePassphrase,
                            focusNextAction = { focusRequester.requestFocus() },
                        )
                        Spacer(
                            modifier = Modifier
                                .weight(1f)
                                .padding(top = Padding.L)
                        )
                        ButtonsForConfirmingPassphrase(
                            confirmPassphraseAndCreateStorage = confirmPassphraseAndCreateStorage,
                            enabled = errorMessage.shouldUiBeEnabled(),
                            focusRequester = focusRequester,
                        )
                    }

                    NewSessionState.CREATING_STORAGE -> {
                        ContentCreatingStorage()
                    }

                    NewSessionState.FINISHED -> {
                        // we pop up to the entry screen to ensure the user cannot accidentally go back
                        // to this screen (which might then only show an indefinite progress spinner)
                        LaunchedEffect(true) {
                            navController.popBackStackAndThenNavigate(
                                popUpTo = CoverDropDestinations.ENTRY_ROUTE,
                                destination = CoverDropDestinations.NEW_MESSAGE_ROUTE,
                            )
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun HelpBannerPassphrase(navController: NavHostController) {
    TwoLineBanner(
        firstLine = stringResource(R.string.screen_new_session_help_banner_keeping_passphrases_safe),
        secondLine = stringResource(R.string.screen_new_session_help_banner_learn_more),
        onClick = { navController.navigate(CoverDropDestinations.HELP_KEEPING_PASSPHRASES_SAFE_ROUTE) }
    )
}

@Composable
private fun ButtonsForShowingPassphrase(
    navController: NavHostController,
    isPassphraseRevealed: Boolean,
    revealPassphrase: () -> Unit,
    advanceToConfirmation: () -> Unit,
) {
    if (isPassphraseRevealed) {
        PrimaryButton(
            text = stringResource(R.string.screen_new_session_button_remembered_my_passphrase),
            onClick = advanceToConfirmation,
            modifier = Modifier
                .padding(start = Padding.M, end = Padding.M)
                .fillMaxWidth(1f),
        )
    } else {
        PrimaryButton(
            text = stringResource(R.string.screen_new_session_button_reveal_passphrase),
            icon = CoverDropIcons.Reveal,
            onClick = revealPassphrase,
            modifier = Modifier
                .padding(start = Padding.M, end = Padding.M)
                .fillMaxWidth(1f)
                .testTag("primary_button_reveal_passphrase"),
        )
    }

    Divider(
        color = NeutralMiddle,
        modifier = Modifier.padding(top = Padding.M)
    )

    ChevronTextButton(
        text = stringResource(R.string.screen_new_session_button_already_have_passphrase),
        onClick = { navController.navigate(CoverDropDestinations.CONTINUE_SESSION_ROUTE) },
        modifier = Modifier.padding(start = Padding.M, end = Padding.M)
    )
}

@Composable
private fun ButtonsForConfirmingPassphrase(
    confirmPassphraseAndCreateStorage: () -> Unit,
    enabled: Boolean,
    focusRequester: FocusRequester,
) {
    val focusManager = LocalFocusManager.current

    PrimaryButton(
        text = stringResource(R.string.screen_new_session_button_confirm_passphrase),
        onClick = {
            focusManager.clearFocus()
            confirmPassphraseAndCreateStorage()
        },
        enabled = enabled,
        modifier = Modifier
            .padding(start = Padding.M, end = Padding.M, bottom = Padding.M)
            .fillMaxWidth(1f)
            .focusRequester(focusRequester)
            .focusable()
    )
}

@Composable
fun ContentConfirmPassphrase(
    passphrase: UiPassphrase,
    errorMessage: UiErrorMessage?,
    updatePassphraseWord: (Int, String) -> Unit = { _, _ -> },
    revealPassphraseWord: (Int) -> Unit = { },
    hidePassphraseWord: (Int) -> Unit = { },
    revealPassphrase: () -> Unit = {},
    hidePassphrase: () -> Unit = {},
    focusNextAction: () -> Unit,
) {
    Column {
        Text(
            text = stringResource(R.string.screen_new_session_header_confirm_passphrase),
            style = MaterialTheme.typography.h1,
        )
        Text(
            text = stringResource(R.string.screen_new_session_text_confirm_passphrase_explanation),
            modifier = Modifier.padding(top = Padding.M),
        )

        errorMessage?.run {
            Spacer(modifier = Modifier.height(Padding.M))
            ErrorMessageWithIcon(
                text = errorMessage.getString(LocalContext.current),
                icon = CoverDropIcons.Warning
            )
        }

        EditPassphraseColumn(
            passphrase = passphrase,
            enabled = errorMessage.shouldUiBeEnabled(),
            onPassphraseWordUpdated = updatePassphraseWord,
            onPassphraseWordRevealed = revealPassphraseWord,
            onPassphraseWordHidden = hidePassphraseWord,
            onPassphraseRevealed = revealPassphrase,
            onPassphraseHidden = hidePassphrase,
            focusNextAction = { focusNextAction(); true },
        )
    }
}

@Composable
private fun ContentShowPassphrase(
    generatedPassphrase: UiPassphrase?,
    hidePassphrase: () -> Unit,
    revealPassphrase: () -> Unit,
) {
    Column {
        Text(
            text = stringResource(R.string.screen_new_session_header_remember_passphrase),
            style = MaterialTheme.typography.h1,
        )
        Text(
            text = highlightText(R.string.screen_new_session_text_remember_passphrase_explanation),
            modifier = Modifier.padding(top = Padding.M),
        )

        if (generatedPassphrase == null) {
            CircularProgressIndicator(
                modifier = Modifier
                    .align(Alignment.CenterHorizontally)
                    .padding(Padding.XL)
            )
        } else {
            TextPassphraseColumn(
                passphrase = generatedPassphrase,
                onPassphraseHidden = hidePassphrase,
                onPassphraseRevealed = revealPassphrase,
                clickOnWordRevealsPassphrase = true,
            )
        }
    }
}

@Composable
private fun ContentCreatingStorage() {
    Column(
        modifier = Modifier
            .fillMaxSize(1f)
            .padding(Padding.L),
        verticalArrangement = Arrangement.Center,
    ) {
        ProgressSpinnerWithText(
            stringResource(R.string.screen_new_session_text_creating_storage_please_wait)
        )
    }
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun NewSessionPreview_withPassphraseHidden() = CoverDropSurface {
    val passphrase = COVERDROP_SAMPLE_DATA.getShortPassphrase()
        .toUiPassphrase()
        .map { it.copyHidden() }
    NewSessionScreen(
        navController = rememberNavController(),
        screenState = NewSessionState.SHOWING_PASSPHRASE,
        generatedPassphrase = passphrase,
    )
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun NewSessionPreview_withLongPassphraseShown() = CoverDropSurface {
    val passphrase = COVERDROP_SAMPLE_DATA.getLongPassphrase()
        .toUiPassphrase()
        .map { it.copyRevealed() }
    NewSessionScreen(
        navController = rememberNavController(),
        screenState = NewSessionState.SHOWING_PASSPHRASE,
        generatedPassphrase = passphrase,
    )
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun NewSessionPreview_withConfirmingLongPassphrase() {
    val passphraseWords = COVERDROP_SAMPLE_DATA.getShortPassphrase()
        .toUiPassphrase()
        .map { it.copyRevealed() }

    CoverDropSurface {
        NewSessionScreen(
            navController = rememberNavController(),
            screenState = NewSessionState.CONFIRMING_PASSPHRASE,
            enteredPassphraseWords = passphraseWords,
        )
    }
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun NewSessionPreview_withConfirmingPassphraseAndErrorMessage() {
    val passphraseWords = COVERDROP_SAMPLE_DATA.getShortPassphrase()
        .toUiPassphrase()
        .map { it.copyRevealed() }

    CoverDropSurface {
        NewSessionScreen(
            navController = rememberNavController(),
            screenState = NewSessionState.CONFIRMING_PASSPHRASE,
            enteredPassphraseWords = passphraseWords,
            errorMessage = COVERDROP_SAMPLE_DATA.getSampleErrorMessage(isFatal = false),
        )
    }
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun NewSessionPreview_withConfirmingPassphraseAndErrorMessageFatal() {
    val passphraseWords = COVERDROP_SAMPLE_DATA.getShortPassphrase()
        .toUiPassphrase()
        .map { it.copyRevealed() }

    CoverDropSurface {
        NewSessionScreen(
            navController = rememberNavController(),
            screenState = NewSessionState.CONFIRMING_PASSPHRASE,
            enteredPassphraseWords = passphraseWords,
            errorMessage = COVERDROP_SAMPLE_DATA.getSampleErrorMessage(isFatal = true),
        )
    }
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun NewSessionPreview_withCreatingStorage() = CoverDropSurface {
    NewSessionScreen(
        navController = rememberNavController(),
        screenState = NewSessionState.CREATING_STORAGE,
    )
}
