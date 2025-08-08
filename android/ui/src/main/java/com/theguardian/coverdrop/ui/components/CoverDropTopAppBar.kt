package com.theguardian.coverdrop.ui.components

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.GenericShape
import androidx.compose.material.Icon
import androidx.compose.material.IconButton
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Text
import androidx.compose.material.TopAppBar
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.platform.LocalInspectionMode
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.theguardian.coverdrop.ui.R
import com.theguardian.coverdrop.ui.theme.BackgroundWarningPastelRed
import com.theguardian.coverdrop.ui.theme.CoverDropPreviewSurface
import com.theguardian.coverdrop.ui.theme.WarningPastelRed
import com.theguardian.coverdrop.ui.viewmodels.TopBarViewModel


enum class TopBarNavigationOption {
    None,
    Back,
    Exit,
}

data class CoverDropTopAppBarWarningBannerInfo(
    val onWarningBannerClick: () -> Unit = {},
    val showWarningBannerSnoozeDialog: Boolean = false,
    val snoozeDialogConfirm: () -> Unit = {},
    val snoozeDialogDismiss: () -> Unit = {},
)

@Composable
fun CoverDropTopAppBar(
    onNavigationOptionPressed: () -> Unit = {},
    navigationOption: TopBarNavigationOption = TopBarNavigationOption.Back
) {
    // short-circuiting the view model in preview mode (the Android UI preview options are not
    // able to instantiate a view model)
    if (LocalInspectionMode.current) {
        CoverDropTopAppBarUi(onNavigationOptionPressed, navigationOption)
        return
    }

    val viewModel = hiltViewModel<TopBarViewModel>()

    val isLocalTestMode = viewModel.isLocalTestMode.collectAsState()
    val showWarningBanner = viewModel.showWarningBanner.collectAsState()
    val showWarningBannerSnoozeDialog = viewModel.showWarningBannerSnoozeDialog.collectAsState()

    CoverDropTopAppBarUi(
        onNavigationOptionPressed = onNavigationOptionPressed,
        navigationOption = navigationOption,
        isLocalTestMode = isLocalTestMode.value,
        onForceRefresh = { viewModel.onForceRefresh() },
        warningBannerInfo = if (showWarningBanner.value) {
            CoverDropTopAppBarWarningBannerInfo(
                onWarningBannerClick = { viewModel.onWarningBannerClick() },
                showWarningBannerSnoozeDialog = showWarningBannerSnoozeDialog.value,
                snoozeDialogConfirm = { viewModel.onSnoozeWarningBannerConfirm() },
                snoozeDialogDismiss = { viewModel.onSnoozeWarningBannerDismiss() },
            )
        } else null
    )
}

/**
 * The top app bar for the CoverDrop app.
 *
 * @param onNavigationOptionPressed The callback to be called when the navigation option is pressed.
 *
 * @param navigationOption The navigation option to display in the app bar.
 *
 * @param isLocalTestMode Whether the app is in local test mode. This displays a TEST MODE warning
 * inside the app bar and a force refresh button.
 *
 * @param warningBannerInfo Whether to show a warning banner that explains that the app is in a
 * testing version. Clicking on the banner will allow to temporary disable the warning.
 *
 * @param onForceRefresh The callback to be called when the force refresh button is pressed (only
 * shown in local test mode).
 */
@Composable
private fun CoverDropTopAppBarUi(
    onNavigationOptionPressed: () -> Unit = {},
    navigationOption: TopBarNavigationOption,
    isLocalTestMode: Boolean = false,
    warningBannerInfo: CoverDropTopAppBarWarningBannerInfo? = null,
    onForceRefresh: () -> Unit = {},
) {
    if (warningBannerInfo?.showWarningBannerSnoozeDialog == true) {
        CoverDropTopBarWarningBannerSnoozeDialog(
            warningBannerInfo.snoozeDialogConfirm,
            warningBannerInfo.snoozeDialogDismiss,
        )
    }
    Column(
        modifier = Modifier
            // we clip the shadow so that it does not appear on top of the top bar and thus making
            // it merge with the status bar which has the same colour
            .clip(GenericShape { size, _ ->
                lineTo(size.width, 0f)
                lineTo(size.width, size.height + 32)
                lineTo(0f, size.height + 32)
            })
            .shadow(4.dp)
    ) {
        TopAppBar(
            elevation = 0.dp, // set to 0 to avoid elevation colour overlaying
            backgroundColor = MaterialTheme.colors.surface,
            navigationIcon = {
                CoverDropTopBarNavigationIcon(navigationOption, onNavigationOptionPressed)
            },
            title = {
                if (isLocalTestMode) Text("TEST MODE", color = WarningPastelRed)
            },
            actions = {
                Image(
                    painter = painterResource(R.drawable.top_bar_logo_shield_and_text),
                    contentDescription = "The CoverDrop logo",
                    Modifier
                        .height(30.dp)
                        .padding(end = 12.dp),
                )
                if (isLocalTestMode) {
                    IconButton(onClick = onForceRefresh) {
                        CoverDropIcons.Refresh.AsComposable()
                    }
                }
            }
        )
        if (warningBannerInfo != null)
            TwoLineBanner(
                firstLine = stringResource(R.string.top_bar_warning_banner_first_line),
                secondLine = stringResource(R.string.top_bar_warning_banner_second_line),
                icon = CoverDropIcons.Warning,
                backgroundColor = BackgroundWarningPastelRed,
                onClick = warningBannerInfo.onWarningBannerClick,
            )
    }
}

@Composable
private fun CoverDropTopBarNavigationIcon(
    navigationOption: TopBarNavigationOption,
    onNavigationOptionPressed: () -> Unit
) = when (navigationOption) {
    TopBarNavigationOption.Back -> IconButton(
        modifier = Modifier.testTag("top_bar_navigation_action"),
        onClick = onNavigationOptionPressed
    ) {
        Icon(Icons.AutoMirrored.Default.ArrowBack, "Back")
    }

    TopBarNavigationOption.Exit -> IconButton(
        modifier = Modifier.testTag("top_bar_navigation_action"),
        onClick = onNavigationOptionPressed
    ) {
        CoverDropIcons.Close.AsComposable()
    }

    else -> {
    }
}

@Composable
private fun CoverDropTopBarWarningBannerSnoozeDialog(
    onWarningBannerSnoozeDialogConfirm: () -> Unit = {},
    onWarningBannerSnoozeDialogDismiss: () -> Unit = {},
) {
    CoverDropConfirmationDialog(
        headingText = stringResource(R.string.top_bar_warning_banner_snooze_dialog_header),
        bodyText = stringResource(R.string.top_bar_warning_banner_snooze_dialog_body),
        onConfirmClick = onWarningBannerSnoozeDialogConfirm,
        confirmText = stringResource(R.string.top_bar_warning_banner_snooze_dialog_button_snooze),
        onDismissClick = onWarningBannerSnoozeDialogDismiss,
        dismissText = stringResource(R.string.top_bar_warning_banner_snooze_dialog_button_dismiss),
    )
}

@Preview
@Composable
fun CoverDropTopBarPreview() = CoverDropPreviewSurface {
    CoverDropTopAppBarUi(navigationOption = TopBarNavigationOption.Back)
}

@Preview
@Composable
fun CoverDropTopBarPreview_alternativeActionExit() = CoverDropPreviewSurface {
    CoverDropTopAppBarUi(navigationOption = TopBarNavigationOption.Exit)
}

@Preview
@Composable
fun CoverDropTopBarPreview_inLocalTestMode() = CoverDropPreviewSurface {
    CoverDropTopAppBarUi(navigationOption = TopBarNavigationOption.Back, isLocalTestMode = true)
}

@Preview
@Composable
fun CoverDropTopBarPreview_withWarningBanner() = CoverDropPreviewSurface {
    CoverDropTopAppBarUi(
        navigationOption = TopBarNavigationOption.Back,
        warningBannerInfo = CoverDropTopAppBarWarningBannerInfo()
    )
}
