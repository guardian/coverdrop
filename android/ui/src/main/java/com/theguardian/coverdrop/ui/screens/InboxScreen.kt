package com.theguardian.coverdrop.ui.screens

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.CircularProgressIndicator
import androidx.compose.material.Divider
import androidx.compose.material.Text
import androidx.compose.material.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.navigation.NavHostController
import androidx.navigation.compose.rememberNavController
import com.theguardian.coverdrop.core.models.JournalistInfo
import com.theguardian.coverdrop.core.models.JournalistVisibility
import com.theguardian.coverdrop.core.models.Message
import com.theguardian.coverdrop.core.models.MessageThread
import com.theguardian.coverdrop.ui.R
import com.theguardian.coverdrop.ui.components.CoverDropConfirmationDialog
import com.theguardian.coverdrop.ui.components.CoverDropErrorDialog
import com.theguardian.coverdrop.ui.components.CoverDropIcons
import com.theguardian.coverdrop.ui.components.CoverDropProgressDialog
import com.theguardian.coverdrop.ui.components.CoverDropTopAppBar
import com.theguardian.coverdrop.ui.components.ErrorMessageWithIcon
import com.theguardian.coverdrop.ui.components.FlatTextButton
import com.theguardian.coverdrop.ui.components.MessageThread
import com.theguardian.coverdrop.ui.components.MessageThreadViewData
import com.theguardian.coverdrop.ui.components.PrimaryButton
import com.theguardian.coverdrop.ui.components.SecondaryButton
import com.theguardian.coverdrop.ui.components.TopBarNavigationOption
import com.theguardian.coverdrop.ui.navigation.BackPressHandler
import com.theguardian.coverdrop.ui.navigation.CoverDropDestinations
import com.theguardian.coverdrop.ui.theme.CoverDropSurface
import com.theguardian.coverdrop.ui.theme.InfoPastelBlue
import com.theguardian.coverdrop.ui.theme.NeutralMiddle
import com.theguardian.coverdrop.ui.theme.Padding
import com.theguardian.coverdrop.ui.utils.humanFriendlyMessageTimeString
import com.theguardian.coverdrop.ui.viewmodels.InboxDialogState
import com.theguardian.coverdrop.ui.viewmodels.InboxUiState
import com.theguardian.coverdrop.ui.viewmodels.InboxViewModel
import java.time.Duration
import java.time.Instant

@Composable
fun InboxRoute(
    navController: NavHostController,
) {
    val viewModel = hiltViewModel<InboxViewModel>()

    val uiState = viewModel.uiState.collectAsState()
    val dialogState = viewModel.dialogState.collectAsState()

    BackPressHandler { viewModel.showExitConfirmationDialog() }

    InboxScreen(
        navController = navController,
        uiState = uiState.value,
        dialogState = dialogState.value,
        messageExpiryDuration = viewModel.messageExpiryDuration,
        onTryToExit = { viewModel.showExitConfirmationDialog() },
        onShowDeleteAllDialog = { viewModel.showDeleteConfirmationDialog() },
        onDismissDialog = { viewModel.dismissCurrentDialog() },
        onShowAbout = { navController.navigate(CoverDropDestinations.ABOUT_ROUTE) },
        onDeleteVault = { viewModel.deleteVault() },
        onLeaveInbox = { viewModel.closeSession() },
    )
}

@Composable
private fun InboxScreen(
    navController: NavHostController,
    uiState: InboxUiState,
    dialogState: InboxDialogState = InboxDialogState.None,
    messageExpiryDuration: Duration = Duration.ofDays(14),
    onTryToExit: () -> Unit = {},
    onShowDeleteAllDialog: () -> Unit = {},
    onDismissDialog: () -> Unit = {},
    onShowAbout: () -> Unit = {},
    onDeleteVault: () -> Unit = {},
    onLeaveInbox: () -> Unit = {},
) {
    MainContent(
        screenState = uiState,
        messageExpiryDuration = messageExpiryDuration,
        navController = navController,
        onShowDeleteAllDialog = onShowDeleteAllDialog,
        onShowAbout = onShowAbout,
        onTryToExit = onTryToExit,
    )

    when (dialogState) {
        is InboxDialogState.None -> {}

        is InboxDialogState.ShowExitConfirmationDialog -> {
            CoverDropConfirmationDialog(
                headingText = stringResource(R.string.screen_inbox_exit_dialog_title),
                bodyText = stringResource(R.string.screen_inbox_exit_dialog_text),
                confirmText = stringResource(R.string.screen_inbox_exit_dialog_button_confirm),
                onConfirmClick = onLeaveInbox,
                dismissText = stringResource(R.string.screen_inbox_exit_dialog_button_cancel),
                onDismissClick = onDismissDialog,
            )
        }

        is InboxDialogState.ShowDeleteConfirmationDialog -> {
            CoverDropConfirmationDialog(
                headingText = stringResource(R.string.screen_inbox_delete_dialog_confirmation_heading),
                bodyText = stringResource(R.string.screen_inbox_delete_dialog_confirmation_content),
                confirmText = stringResource(R.string.screen_inbox_delete_dialog_confirmation_button_delete),
                onConfirmClick = onDeleteVault,
                dismissText = stringResource(R.string.screen_inbox_delete_dialog_confirmation_button_keep),
                onDismissClick = onDismissDialog,
            )
        }

        is InboxDialogState.ShowDeletingProgressDialog -> {
            CoverDropProgressDialog(
                headingText = stringResource(R.string.screen_inbox_delete_dialog_progress_heading),
                onDismissClick = { /* no-op */ },
            )
        }

        is InboxDialogState.ShowDeletionErrorDialog -> {
            CoverDropErrorDialog(
                headingText = stringResource(R.string.screen_inbox_delete_dialog_error_heading),
                bodyText = stringResource(R.string.screen_inbox_delete_dialog_error_content),
                dismissText = stringResource(R.string.screen_inbox_delete_dialog_error_button_dismiss),
                onDismissClick = onDismissDialog,
            )
        }
    }
}

@Composable
private fun MainContent(
    screenState: InboxUiState,
    messageExpiryDuration: Duration,
    navController: NavHostController,
    onShowDeleteAllDialog: () -> Unit,
    onShowAbout: () -> Unit,
    onTryToExit: () -> Unit,
    now: Instant = Instant.now(),
) {
    Column(modifier = Modifier.fillMaxHeight(1f)) {
        CoverDropTopAppBar(
            navigationOption = TopBarNavigationOption.Exit,
            onNavigationOptionPressed = onTryToExit
        )

        when (screenState) {
            is InboxUiState.Loading -> {
                Column(
                    modifier = Modifier
                        .weight(1f)
                        .fillMaxWidth(),
                    verticalArrangement = Arrangement.Center,
                    horizontalAlignment = Alignment.CenterHorizontally
                ) {
                    CircularProgressIndicator(modifier = Modifier.padding(20.dp))
                }
            }

            is InboxUiState.ShowMessages -> {
                Column(
                    modifier = Modifier
                        .verticalScroll(rememberScrollState())
                        .weight(1f)
                ) {
                    ThreadsList(screenState, messageExpiryDuration, navController, now)
                }
            }

            is InboxUiState.Exit -> {
                navController.popBackStack(CoverDropDestinations.ENTRY_ROUTE, inclusive = false)
            }
        }

        // If there are no current conversations, show the send new message button
        if (screenState is InboxUiState.ShowMessages && screenState.activeConversation == null) {
            Column(
                modifier = Modifier.padding(Padding.M)
            ) {
                SecondaryButton(
                    text = stringResource(R.string.screen_conversation_button_send_new),
                    onClick = { navController.navigate(CoverDropDestinations.NEW_MESSAGE_ROUTE) },
                )
            }
        }

        // If there is a current message, show the delete button
        if (screenState is InboxUiState.ShowMessages && screenState.activeConversation != null) {
            FlatTextButton(
                text = stringResource(R.string.screen_inbox_delete_your_messages),
                icon = CoverDropIcons.Delete,
                modifier = Modifier
                    .align(Alignment.Start)
                    .padding(start = Padding.M, end = Padding.M),
                onClick = onShowDeleteAllDialog,
            )
        }

        Divider(color = NeutralMiddle, modifier = Modifier.padding(top = Padding.M))

        Row(
            horizontalArrangement = Arrangement.SpaceBetween,
            modifier = Modifier
                .fillMaxWidth()
                .padding(all = Padding.M)
        ) {
            TextButton(
                onClick = onShowAbout, modifier = Modifier.align(Alignment.CenterVertically)
            ) {
                Text(
                    text = stringResource(R.string.screen_inbox_about_secure_messaging),
                    color = Color.White,
                    fontWeight = FontWeight.Bold,
                )
            }

            PrimaryButton(
                onClick = onTryToExit,
                text = stringResource(R.string.screen_inbox_leave_inbox),
                modifier = Modifier.wrapContentWidth(align = Alignment.End)
            )
        }
    }
}

@Composable
private fun ThreadsList(
    screenState: InboxUiState.ShowMessages,
    messageExpiryDuration: Duration,
    navController: NavHostController,
    now: Instant,
) {
    if (screenState.activeConversation == null) {
        Row(
            Modifier
                .fillMaxWidth()
                .padding(12.dp)
        ) {
            ErrorMessageWithIcon(
                text = stringResource(
                    R.string.screen_inbox_text_no_messages_last_x_days,
                    messageExpiryDuration.toDays()
                ),
                icon = CoverDropIcons.Info,
                colorBorder = InfoPastelBlue,
                colorText = Color.White
            )
        }
    }

    // active message threads
    screenState.activeConversation?.let { active ->
        val mostRecentUpdate = active.mostRecentUpdate()
        Row(
            Modifier.fillMaxWidth()
        ) {
            MessageThread(
                viewData = MessageThreadViewData.Active(
                    name = active.recipient.displayName,
                    time = mostRecentUpdate?.let {
                        humanFriendlyMessageTimeString(
                            timestamp = it,
                            now = now,
                            forceAbsoluteTime = true
                        )
                    }
                ),
                onClick = {
                    navController.navigate(
                        route = CoverDropDestinations.CONVERSATION_ROUTE.replace(
                            oldValue = "{id}", newValue = active.recipient.id
                        )
                    )
                },
            )
        }
    }

    // expired message threads
    screenState.inactiveConversation.forEach { thread ->
        val mostRecentUpdate = thread.mostRecentUpdate()
        Row(
            Modifier.fillMaxWidth()
        ) {
            MessageThread(
                viewData = MessageThreadViewData.Inactive(
                    name = thread.recipient.displayName,
                    time = mostRecentUpdate?.let {
                        humanFriendlyMessageTimeString(
                            timestamp = it,
                            now = now,
                            forceAbsoluteTime = true
                        )
                    }
                ),
                onClick = {
                    navController.navigate(
                        route = CoverDropDestinations.CONVERSATION_ROUTE.replace(
                            oldValue = "{id}", newValue = thread.recipient.id
                        )
                    )
                },
            )
        }
    }
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun InboxScreenPreview_loading() {
    CoverDropSurface {
        InboxScreen(
            navController = rememberNavController(),
            uiState = InboxUiState.Loading,
        )
    }
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun InboxScreenPreview_withMessages() {
    val uiState = getSampleThreadsUiStateForPreview()
    CoverDropSurface {
        InboxScreen(
            navController = rememberNavController(),
            uiState = uiState,
        )
    }
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun InboxScreenPreview_empty() {
    CoverDropSurface {
        InboxScreen(
            navController = rememberNavController(),
            uiState = InboxUiState.ShowMessages(null, emptyList()),
            messageExpiryDuration = Duration.ofDays(42),
        )
    }
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun InboxScreenPreview_showingExitConformationDialog() {
    val uiState = getSampleThreadsUiStateForPreview()
    CoverDropSurface {
        InboxScreen(
            navController = rememberNavController(),
            uiState = uiState,
            dialogState = InboxDialogState.ShowExitConfirmationDialog,
        )
    }
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun InboxScreenPreview_showingDeletionConformationDialog() {
    val uiState = getSampleThreadsUiStateForPreview()
    CoverDropSurface {
        InboxScreen(
            navController = rememberNavController(),
            uiState = uiState,
            dialogState = InboxDialogState.ShowDeleteConfirmationDialog,
        )
    }
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun InboxScreenPreview_showingDeletionProgressDialog() {
    val uiState = getSampleThreadsUiStateForPreview()
    CoverDropSurface {
        InboxScreen(
            navController = rememberNavController(),
            uiState = uiState,
            dialogState = InboxDialogState.ShowDeletingProgressDialog,
        )
    }
}

private fun getSampleThreadsUiStateForPreview(): InboxUiState.ShowMessages {
    val sampleMessages = listOf(Message.Sent("message 1", Instant.now()))

    val journalist1 = JournalistInfo(
        id = "1",
        displayName = "Tom",
        description = "description",
        isTeam = false,
        tag = "a0b1c2d3",
        visibility = JournalistVisibility.VISIBLE,
    )

    val journalistNames = listOf(
        "Sheila",
        "Tony",
        "Peter",
        "Angela",
        "Penny",
        "Dave",
    )

    val expiredThreads: List<MessageThread> = journalistNames.map {
        MessageThread(
            JournalistInfo(
                id = "1",
                displayName = it,
                description = "description",
                isTeam = false,
                tag = "a0b1c2d3",
                visibility = JournalistVisibility.VISIBLE,
            ), sampleMessages
        )
    }

    return InboxUiState.ShowMessages(
        MessageThread(journalist1, sampleMessages), expiredThreads
    )
}
