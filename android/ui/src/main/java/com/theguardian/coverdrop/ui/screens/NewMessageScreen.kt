package com.theguardian.coverdrop.ui.screens

import android.widget.Toast
import android.widget.Toast.LENGTH_SHORT
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Text
import androidx.compose.material.TextField
import androidx.compose.material.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation.NavHostController
import androidx.navigation.compose.rememberNavController
import com.theguardian.coverdrop.ui.R
import com.theguardian.coverdrop.ui.components.CoverDropConfirmationDialog
import com.theguardian.coverdrop.ui.components.CoverDropIcons
import com.theguardian.coverdrop.ui.components.CoverDropTopAppBar
import com.theguardian.coverdrop.ui.components.ErrorMessageWithIcon
import com.theguardian.coverdrop.ui.components.MessageLimitIndicator
import com.theguardian.coverdrop.ui.components.PrimaryButton
import com.theguardian.coverdrop.ui.components.TopBarNavigationOption
import com.theguardian.coverdrop.ui.components.TwoLineBanner
import com.theguardian.coverdrop.ui.navigation.BackPressHandler
import com.theguardian.coverdrop.ui.navigation.CoverDropDestinations
import com.theguardian.coverdrop.ui.theme.CoverDropSurface
import com.theguardian.coverdrop.ui.theme.Padding
import com.theguardian.coverdrop.ui.theme.RoundedCorners
import com.theguardian.coverdrop.ui.utils.COVERDROP_SAMPLE_DATA
import com.theguardian.coverdrop.ui.utils.ScreenContentWrapper
import com.theguardian.coverdrop.ui.utils.rememberScreenInsets
import com.theguardian.coverdrop.ui.viewmodels.NewMessageUiState
import com.theguardian.coverdrop.ui.viewmodels.NewMessageViewModel
import com.theguardian.coverdrop.ui.viewmodels.SelectedRecipientState
import com.theguardian.coverdrop.ui.viewmodels.SelectedRecipientViewModel


@Composable
fun NewMessageRoute(
    sharedSelectedRecipientViewModel: SelectedRecipientViewModel,
    navController: NavHostController,
) {
    val viewModel = hiltViewModel<NewMessageViewModel>()

    val uiState = viewModel.uiState.collectAsState()
    val busy = viewModel.getBusy().collectAsStateWithLifecycle()
    val text = viewModel.getMessage().collectAsStateWithLifecycle()
    val errorMessage = viewModel.getErrorMessage().collectAsStateWithLifecycle()
    val messagePercentSize = viewModel.messageSizeState.collectAsStateWithLifecycle()

    val selectedRecipient = sharedSelectedRecipientViewModel
        .getSelectedRecipient()
        .collectAsStateWithLifecycle()

    BackPressHandler { viewModel.showExitConfirmationDialog() }

    when (uiState.value) {
        NewMessageUiState.SHOWN, NewMessageUiState.CONFIRM_LEAVING -> {
            NewMessageScreen(
                navController = navController,
                selectedRecipient = selectedRecipient.value,
                busy = busy.value,
                text = text.value,
                errorMessage = errorMessage.value,
                showExitConfirmationDialog = uiState.value == NewMessageUiState.CONFIRM_LEAVING,
                onSelectRecipient = { navController.navigate(CoverDropDestinations.RECIPIENT_SELECTION_ROUTE) },
                onMessageChanged = { newValue -> viewModel.onMessageChanged(newValue) },
                totalMessageSizePercent = messagePercentSize.value,
                onSendMessage = { viewModel.onSendMessage(recipient = selectedRecipient.value) },
                onTryToExit = { viewModel.showExitConfirmationDialog() },
                onDismissDialog = { viewModel.dismissCurrentDialog() },
                onExit = { viewModel.closeSession() },
            )
        }

        NewMessageUiState.FINISHED -> {
            LaunchedEffect(true) {
                navController.navigate(CoverDropDestinations.MESSAGE_SENT_ROUTE)
                sharedSelectedRecipientViewModel.forceResetToInitializing()
            }
        }

        NewMessageUiState.EXIT -> {
            LaunchedEffect(true) {
                navController.popBackStack(CoverDropDestinations.ENTRY_ROUTE, inclusive = false)
                sharedSelectedRecipientViewModel.forceResetToInitializing()
            }
        }
    }
}

@Composable
private fun NewMessageScreen(
    navController: NavHostController,
    selectedRecipient: SelectedRecipientState,
    busy: Boolean,
    text: String,
    errorMessage: String? = null,
    showExitConfirmationDialog: Boolean = false,
    onSelectRecipient: () -> Unit = {},
    onMessageChanged: (String) -> Unit = {},
    totalMessageSizePercent: Float,
    onSendMessage: () -> Unit = {},
    onTryToExit: () -> Unit = {},
    onDismissDialog: () -> Unit = {},
    onExit: () -> Unit = {},
) {
    val focusManager = LocalFocusManager.current
    val foregroundColor = MaterialTheme.colors.onBackground

    if (showExitConfirmationDialog) {
        CoverDropConfirmationDialog(
            headingText = stringResource(R.string.screen_new_message_exit_dialog_title),
            bodyText = stringResource(R.string.screen_new_message_exit_dialog_text),
            confirmText = stringResource(R.string.screen_new_message_exit_dialog_button_confirm),
            onConfirmClick = onExit,
            dismissText = stringResource(R.string.screen_new_message_exit_dialog_button_cancel),
            onDismissClick = onDismissDialog,
        )
    }
    ScreenContentWrapper {
        Column(modifier = Modifier
            .fillMaxHeight(1f)
            .padding(bottom = rememberScreenInsets().bottom)
        ) {
            CoverDropTopAppBar(
                onNavigationOptionPressed = onTryToExit,
                navigationOption = TopBarNavigationOption.Exit,
            )

            TwoLineBanner(
                firstLine = stringResource(R.string.screen_new_message_help_banner_craft_your_first_message),
                secondLine = stringResource(R.string.screen_new_message_help_banner_learn_more),
                onClick = { navController.navigate(CoverDropDestinations.HELP_CRAFT_MESSAGE_ROUTE) }
            )

            Column(
                modifier = Modifier
                    .verticalScroll(rememberScrollState())
                    .weight(1f)
            ) {
                Column(
                    modifier = Modifier
                        .verticalScroll(rememberScrollState())
                        .padding(Padding.L)
                        .weight(1f)
                ) {
                    Text(
                        text = stringResource(R.string.screen_new_message_header_new_message),
                        style = MaterialTheme.typography.h1,
                    )

                    errorMessage?.run {
                        Spacer(modifier = Modifier.height(Padding.M))
                        ErrorMessageWithIcon(text = this, icon = CoverDropIcons.Warning)
                    }

                    // Recipient
                    val context = LocalContext.current
                    val recipientForcedError =
                        stringResource(R.string.screen_new_message_error_forced_recipient)
                    InputFieldHeaderAndDescription(
                        headerText = stringResource(R.string.screen_new_message_text_who_would_you_like_to_contact),
                        descriptionText = stringResource(R.string.screen_new_message_text_desc_who_would_you_like_to_contact)
                    )

                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(top = Padding.S)
                            .background(MaterialTheme.colors.surface)
                            .clickable {
                                when (selectedRecipient) {
                                    is SelectedRecipientState.Initializing -> {}
                                    is SelectedRecipientState.SingleRecipientWithChoice -> onSelectRecipient()
                                    is SelectedRecipientState.EmptySelectionWithChoice -> onSelectRecipient()
                                    is SelectedRecipientState.SingleRecipientForced -> {
                                        Toast
                                            .makeText(context, recipientForcedError, LENGTH_SHORT)
                                            .show()
                                    }
                                }
                            }
                            .drawBehind {
                                drawRoundRect(
                                    color = foregroundColor,
                                    style = Stroke(width = 2f),
                                    cornerRadius = CornerRadius(
                                        x = RoundedCorners.XS.toPx(),
                                        y = RoundedCorners.XS.toPx()
                                    )
                                )
                            }
                            .testTag("edit_recipient"),
                    ) {
                        Row(modifier = Modifier.padding(Padding.L)) {
                            Text(
                                when (selectedRecipient) {
                                    is SelectedRecipientState.Initializing -> stringResource(R.string.screen_new_message_text_recipient_loading)
                                    is SelectedRecipientState.EmptySelectionWithChoice -> stringResource(
                                        R.string.screen_new_message_text_no_recipient_selected
                                    )

                                    else -> selectedRecipient.getJournalistInfoOrNull()!!.displayName
                                },
                                style = MaterialTheme.typography.body1,
                            )
                            Spacer(modifier = Modifier.weight(1f))
                            if (selectedRecipient.userHasChoice()) {
                                CoverDropIcons.Edit.AsComposable()
                                Text(
                                    stringResource(R.string.screen_new_message_text_recipient_change),
                                    style = MaterialTheme.typography.body1.copy(fontWeight = FontWeight.W700),
                                    modifier = Modifier.padding(start = Padding.S)
                                )
                            }
                        }
                    }

                    var messageText by remember { mutableStateOf(TextFieldValue(text)) }

                    InputFieldHeaderAndDescription(
                        headerText = stringResource(R.string.screen_new_message_text_your_message),
                        descriptionText = stringResource(R.string.screen_new_message_text_your_message_description),
                    )

                    MessageLimitIndicator(
                        percentFull = totalMessageSizePercent
                    )

                    TextField(
                        value = messageText,
                        onValueChange = { messageText = it; onMessageChanged(it.text) },
                        singleLine = false,
                        maxLines = 5,
                        colors = TextFieldDefaults.textFieldColors(
                            backgroundColor = Color.Transparent,
                            unfocusedIndicatorColor = Color.Transparent,
                            focusedIndicatorColor = Color.Transparent,
                        ),
                        modifier = Modifier
                            .padding(top = Padding.S)
                            .defaultMinSize(minHeight = 100.dp)
                            .fillMaxWidth()
                            .background(MaterialTheme.colors.surface)
                            .drawBehind {
                                drawRoundRect(
                                    color = foregroundColor,
                                    style = Stroke(width = 2f),
                                    cornerRadius = CornerRadius(
                                        x = RoundedCorners.XS.toPx(),
                                        y = RoundedCorners.XS.toPx()
                                    )
                                )
                            }
                            .testTag("edit_message")
                    )

                    PrimaryButton(
                        text = stringResource(R.string.screen_new_message_button_send_message),
                        enabled = !busy && totalMessageSizePercent < 1.0f,
                        onClick = {
                            focusManager.clearFocus()
                            onMessageChanged(messageText.text)
                            onSendMessage()
                        },
                        modifier = Modifier
                            .padding(vertical = Padding.L)
                            .fillMaxWidth(1f)
                    )
                }
            }
        }
    }
}

@Composable
private fun InputFieldHeaderAndDescription(headerText: String, descriptionText: String?) {
    Column(modifier = Modifier.padding(top = Padding.L, bottom = Padding.S)) {
        Text(
            text = headerText,
            fontWeight = FontWeight.Bold,
        )
        if (descriptionText != null) {
            Text(text = descriptionText, style = MaterialTheme.typography.body2)
        }
    }
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun NewMessageScreenLengthOkPreview() = CoverDropSurface {
    NewMessageScreen(
        navController = rememberNavController(),
        busy = false,
        text = COVERDROP_SAMPLE_DATA.getSampleMessage(),
        totalMessageSizePercent = 0.9f,
        selectedRecipient = SelectedRecipientState.SingleRecipientWithChoice(
            COVERDROP_SAMPLE_DATA.getTeams().first()
        ),
    )
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun NewMessageScreenConfirmExitPreview() = CoverDropSurface {
    NewMessageScreen(
        navController = rememberNavController(),
        busy = false,
        text = COVERDROP_SAMPLE_DATA.getSampleMessage(),
        totalMessageSizePercent = 0.9f,
        selectedRecipient = SelectedRecipientState.SingleRecipientWithChoice(
            COVERDROP_SAMPLE_DATA.getTeams().first()
        ),
        showExitConfirmationDialog = true,
    )
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun NewMessageScreenInitializingRecipientPreview() = CoverDropSurface {
    NewMessageScreen(
        navController = rememberNavController(),
        busy = false,
        text = COVERDROP_SAMPLE_DATA.getSampleMessage(),
        totalMessageSizePercent = 0.9f,
        selectedRecipient = SelectedRecipientState.Initializing,
    )
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun NewMessageScreenForceWithNoChoiceRecipientPreview() = CoverDropSurface {
    NewMessageScreen(
        navController = rememberNavController(),
        busy = false,
        text = COVERDROP_SAMPLE_DATA.getSampleMessage(),
        totalMessageSizePercent = 0.9f,
        selectedRecipient = SelectedRecipientState.SingleRecipientForced(
            COVERDROP_SAMPLE_DATA.getTeams().first()
        ),
    )
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun NewMessageScreenWithErrorMessage() = CoverDropSurface {
    NewMessageScreen(
        navController = rememberNavController(),
        busy = false,
        text = "",
        totalMessageSizePercent = 0.9f,
        selectedRecipient = SelectedRecipientState.SingleRecipientWithChoice(
            COVERDROP_SAMPLE_DATA.getTeams().first()
        ),
        errorMessage = "Something went wrong. And this message is long.",
    )
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun NewMessageScreenMessageTooLongPreview() = CoverDropSurface {
    NewMessageScreen(
        navController = rememberNavController(),
        busy = false,
        text = COVERDROP_SAMPLE_DATA.getSampleMessage(100),
        totalMessageSizePercent = 1.1f,
        selectedRecipient = SelectedRecipientState.SingleRecipientWithChoice(
            COVERDROP_SAMPLE_DATA.getTeams().first()
        ),
    )
}
