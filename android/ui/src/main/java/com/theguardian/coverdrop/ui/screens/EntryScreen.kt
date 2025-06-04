package com.theguardian.coverdrop.ui.screens

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.CircularProgressIndicator
import androidx.compose.material.Divider
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.navigation.NavHostController
import androidx.navigation.compose.rememberNavController
import com.theguardian.coverdrop.core.api.models.SystemStatus
import com.theguardian.coverdrop.core.models.StatusEvent
import com.theguardian.coverdrop.ui.R
import com.theguardian.coverdrop.ui.components.ChevronTextDirectlyAfterButton
import com.theguardian.coverdrop.ui.components.CoverDropConfirmationDialog
import com.theguardian.coverdrop.ui.components.CoverDropTopAppBar
import com.theguardian.coverdrop.ui.components.PrimaryButton
import com.theguardian.coverdrop.ui.components.SecondaryButton
import com.theguardian.coverdrop.ui.components.StrapLine
import com.theguardian.coverdrop.ui.components.TopBarNavigationOption
import com.theguardian.coverdrop.ui.navigation.CoverDropDestinations
import com.theguardian.coverdrop.ui.theme.CoverDropSurface
import com.theguardian.coverdrop.ui.theme.Padding
import com.theguardian.coverdrop.ui.utils.findComponentActivity
import com.theguardian.coverdrop.ui.viewmodels.EntryViewModel

@Composable
fun EntryRoute(navController: NavHostController) {
    val viewModel = hiltViewModel<EntryViewModel>()
    val status = viewModel.status.collectAsState()

    EntryScreen(
        navController = navController,
        status = status.value,
    )
}

@Composable
fun EntryScreen(
    navController: NavHostController,
    status: StatusEvent?,
) {
    when {
        status == null -> {
            EntryScreenWaiting()
        }

        status.isAvailable -> {
            EntryScreenHappy(navController = navController)
        }

        else -> {
            EntryScreenUnavailable(navController = navController, status = status)
        }
    }
}

@Composable
private fun EntryScreenTopBar() {
    val context = LocalContext.current
    CoverDropTopAppBar(
        navigationOption = TopBarNavigationOption.Exit,
        onNavigationOptionPressed = {
            context.findComponentActivity()?.finish()
        }
    )
}

@Composable
private fun EntryScreenWaiting() {
    Column(modifier = Modifier.fillMaxHeight(1f)) {
        EntryScreenTopBar()

        Column(
            modifier = Modifier.fillMaxSize(),
            verticalArrangement = Arrangement.Center,
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            CircularProgressIndicator()
        }
    }
}

@Composable
private fun EntryScreenUnavailable(navController: NavHostController, status: StatusEvent) {
    Column(modifier = Modifier.fillMaxHeight(1f)) {
        EntryScreenTopBar()

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
                StrapLine(
                    modifier = Modifier
                )

                ChevronTextDirectlyAfterButton(
                    text = stringResource(R.string.screen_entry_link_about),
                    modifier = Modifier,
                    onClick = { navController.navigate(CoverDropDestinations.ABOUT_ROUTE) }
                )

                Spacer(Modifier.weight(1f))

                Divider(Modifier.padding(vertical = Padding.L))

                Text(text = stringResource(R.string.screen_entry_coverdrop_not_available))

                Text(
                    text = "${status.status}: ${status.description}",
                    fontFamily = FontFamily.Monospace,
                    modifier = Modifier.padding(top = Padding.M)
                )

                Divider(Modifier.padding(vertical = Padding.L))
            }
        }
    }
}

@Composable
private fun EntryScreenHappy(
    navController: NavHostController,
    showGetStartedDialogInitialValue: Boolean = false
) {
    var showGetStartedDialog by rememberSaveable { mutableStateOf(showGetStartedDialogInitialValue) }
    if (showGetStartedDialog) {
        CoverDropConfirmationDialog(
            headingText = stringResource(R.string.dialog_confirm_new_conversation_heading),
            bodyText = stringResource(R.string.dialog_confirm_new_conversation_content),
            confirmText = stringResource(R.string.dialog_confirm_new_conversation_button_yes),
            onConfirmClick = {
                showGetStartedDialog = false
                navController.navigate(CoverDropDestinations.HOW_THIS_WORKS_ROUTE)
            },
            dismissText = stringResource(R.string.dialog_confirm_new_conversation_button_no),
            onDismissClick = { showGetStartedDialog = false },
        )
    }

    Column(modifier = Modifier.fillMaxHeight(1f)) {
        EntryScreenTopBar()

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
                StrapLine(
                    modifier = Modifier
                )

                ChevronTextDirectlyAfterButton(
                    text = stringResource(R.string.screen_entry_link_about),
                    modifier = Modifier,
                    onClick = { navController.navigate(CoverDropDestinations.ABOUT_ROUTE) }
                )
            }

            Column(
                modifier = Modifier.padding(Padding.L)
            ) {
                PrimaryButton(
                    modifier = Modifier.fillMaxWidth(1f),
                    text = stringResource(R.string.screen_entry_button_get_started),
                    onClick = { showGetStartedDialog = true },
                )
                SecondaryButton(
                    modifier = Modifier.fillMaxWidth(1f),
                    text = stringResource(R.string.screen_entry_button_check_your_inbox),
                    onClick = { navController.navigate(CoverDropDestinations.CONTINUE_SESSION_ROUTE) },
                )
            }

        }
    }
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun EntryScreen_available() = CoverDropSurface {
    EntryScreen(
        navController = rememberNavController(),
        status = StatusEvent(SystemStatus.AVAILABLE, true, ""),
    )
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun EntryScreen_waiting() = CoverDropSurface {
    EntryScreen(
        navController = rememberNavController(),
        status = null,
    )
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun EntryScreen_unavailable() = CoverDropSurface {
    EntryScreen(
        navController = rememberNavController(),
        status = StatusEvent(
            status = SystemStatus.SCHEDULED_MAINTENANCE,
            isAvailable = false,
            description = "Busy making tea and finding biscuits..."
        ),
    )
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun EntryScreen_dialogOpen() = CoverDropSurface {
    EntryScreenHappy(
        navController = rememberNavController(),
        showGetStartedDialogInitialValue = true,
    )
}
