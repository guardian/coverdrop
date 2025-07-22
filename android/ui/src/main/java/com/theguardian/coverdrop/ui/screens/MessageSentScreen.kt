package com.theguardian.coverdrop.ui.screens

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.Divider
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.navigation.NavHostController
import androidx.navigation.compose.rememberNavController
import com.theguardian.coverdrop.ui.R
import com.theguardian.coverdrop.ui.components.CoverDropConfirmationDialog
import com.theguardian.coverdrop.ui.components.CoverDropTopAppBar
import com.theguardian.coverdrop.ui.components.PrimaryButton
import com.theguardian.coverdrop.ui.components.SecondaryButton
import com.theguardian.coverdrop.ui.components.TopBarNavigationOption
import com.theguardian.coverdrop.ui.components.TwoLineButton
import com.theguardian.coverdrop.ui.navigation.BackPressHandler
import com.theguardian.coverdrop.ui.navigation.CoverDropDestinations
import com.theguardian.coverdrop.ui.theme.CoverDropSurface
import com.theguardian.coverdrop.ui.theme.Padding
import com.theguardian.coverdrop.ui.utils.ScreenContentWrapper
import com.theguardian.coverdrop.ui.utils.rememberScreenInsets
import com.theguardian.coverdrop.ui.viewmodels.MessageSentUiState
import com.theguardian.coverdrop.ui.viewmodels.MessageSentViewModel

@Composable
fun MessageSentRoute(navController: NavHostController) {
    val viewModel = hiltViewModel<MessageSentViewModel>()

    val uiState = viewModel.uiState.collectAsState()

    BackPressHandler { viewModel.showExitConfirmationDialog() }

    when (uiState.value) {
        MessageSentUiState.SHOWN, MessageSentUiState.CONFIRM_LEAVING -> {
            MessageSentScreen(
                navController = navController,
                showExitConfirmationDialog = uiState.value == MessageSentUiState.CONFIRM_LEAVING,
                onTryToExit = { viewModel.showExitConfirmationDialog() },
                onDismissDialog = { viewModel.dismissCurrentDialog() },
                onExit = { viewModel.closeSession() },
            )
        }

        MessageSentUiState.EXIT -> {
            navController.popBackStack(CoverDropDestinations.ENTRY_ROUTE, inclusive = false)
        }
    }
}

@Composable
fun MessageSentScreen(
    navController: NavHostController,
    showExitConfirmationDialog: Boolean = false,
    onTryToExit: () -> Unit = {},
    onDismissDialog: () -> Unit = {},
    onExit: () -> Unit = {},
) {
    val context = LocalContext.current

    if (showExitConfirmationDialog) {
        CoverDropConfirmationDialog(
            headingText = stringResource(R.string.screen_message_sent_exit_dialog_title),
            bodyText = stringResource(R.string.screen_message_sent_exit_dialog_text),
            confirmText = stringResource(R.string.screen_message_sent_exit_dialog_button_confirm),
            onConfirmClick = onExit,
            dismissText = stringResource(R.string.screen_message_sent_exit_dialog_button_cancel),
            onDismissClick = onDismissDialog,
        )
    }
    ScreenContentWrapper {

    Column(
        modifier = Modifier
            .fillMaxHeight(1f)
            .padding(bottom = rememberScreenInsets().bottom)
    ) {

        CoverDropTopAppBar(
            onNavigationOptionPressed = onTryToExit,
            navigationOption = TopBarNavigationOption.Exit,
        )

        Column(
            modifier = Modifier
                .verticalScroll(rememberScrollState())
                .padding(Padding.L)
                .weight(1f),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Text(
                text = context.getString(R.string.screen_message_sent_header_main),
                style = MaterialTheme.typography.h2,
                textAlign = TextAlign.Center,
                modifier = Modifier.padding(top = Padding.M),
            )
            Text(
                text = stringResource(R.string.screen_message_sent_header_sub),
                style = MaterialTheme.typography.body1,
                modifier = Modifier.padding(top = Padding.M),
            )

            Divider(
                color = MaterialTheme.colors.onBackground,
                thickness = 1.dp,
                modifier = Modifier
                    .padding(Padding.L)
                    .alpha(0.2f)
            )

            Column(
                modifier = Modifier
                    .fillMaxWidth(1f),
                horizontalAlignment = Alignment.Start
            ) {
                Text(
                    text = context.getString(R.string.screen_message_sent_header2_main),
                    style = MaterialTheme.typography.h3,
                )
                Text(
                    text = context.getString(R.string.screen_message_sent_content_main),
                    style = MaterialTheme.typography.body1,
                    modifier = Modifier.padding(top = Padding.M, bottom = Padding.XL),
                )
                TwoLineButton(
                    firstLine = stringResource(R.string.screen_message_sent_help_button_what_to_expect_as_a_reply),
                    secondLine = stringResource(R.string.screen_message_sent_help_button_read_more),
                ) {
                    navController.navigate(CoverDropDestinations.HELP_REPLY_EXPECTATIONS)
                }
            }
        }

        Column(
            modifier = Modifier
                .padding(Padding.L)
        ) {
            PrimaryButton(
                text = stringResource(R.string.screen_message_sent_button_go_to_inbox),
                onClick = { navController.navigate(CoverDropDestinations.INBOX_ROUTE) },
                modifier = Modifier.fillMaxWidth(1f),
            )
            SecondaryButton(
                text = stringResource(R.string.screen_message_sent_button_logout),
                onClick = onTryToExit,
                modifier = Modifier.fillMaxWidth(1f),
            )
        }
    }
    }
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun NewMessageScreenPreview() = CoverDropSurface {
    MessageSentScreen(navController = rememberNavController())
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun NewMessageScreenPreviewWithExitConfirmationDialog() = CoverDropSurface {
    MessageSentScreen(
        navController = rememberNavController(),
        showExitConfirmationDialog = true,
    )
}
