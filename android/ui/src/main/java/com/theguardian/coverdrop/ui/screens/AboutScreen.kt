package com.theguardian.coverdrop.ui.screens

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.navigation.NavHostController
import androidx.navigation.compose.rememberNavController
import com.theguardian.coverdrop.core.models.DebugContext
import com.theguardian.coverdrop.ui.R
import com.theguardian.coverdrop.ui.components.ChevronTextButtonGroup
import com.theguardian.coverdrop.ui.components.ChevronTextButtonGroupRowInformation
import com.theguardian.coverdrop.ui.components.CoverDropTopAppBar
import com.theguardian.coverdrop.ui.navigation.CoverDropDestinations
import com.theguardian.coverdrop.ui.theme.CoverDropSurface
import com.theguardian.coverdrop.ui.theme.Padding
import com.theguardian.coverdrop.ui.viewmodels.AboutScreenViewModel
import java.time.Instant


@Composable
fun AboutRoute(navController: NavHostController) {
    val viewModel = hiltViewModel<AboutScreenViewModel>()

    val debugContextState = viewModel.debugContext.collectAsState()
    AboutScreen(navController = navController, debugContext = debugContextState.value)
}

@Composable
private fun AboutScreen(navController: NavHostController, debugContext: DebugContext?) {
    Column(modifier = Modifier.fillMaxHeight(1f)) {
        CoverDropTopAppBar(
            onNavigationOptionPressed = { navController.navigateUp() }
        )
        Column(
            modifier = Modifier
                .fillMaxHeight()
                .verticalScroll(rememberScrollState())
        ) {
            Column(
                modifier = Modifier.padding(Padding.L)
            ) {
                MainContent(navController, debugContext)
            }
        }
    }
}

@Composable
private fun MainContent(navController: NavHostController, debugContext: DebugContext?) {
    Text(
        text = stringResource(id = R.string.screen_about_header),
        style = MaterialTheme.typography.h1,
    )

    Text(
        text = stringResource(R.string.screen_about_headline_what_this_is_for),
        style = MaterialTheme.typography.h2,
        modifier = Modifier.padding(top = Padding.XL, bottom = Padding.L),
    )
    ChevronTextButtonGroup(
        buttons = listOf(
            ChevronTextButtonGroupRowInformation(
                text = stringResource(R.string.screen_about_button_why_we_made_secure_messaging),
                onClick = { navController.navigate(CoverDropDestinations.HELP_WHY_WE_MADE_SECURE_MESSAGING) }
            ),
            ChevronTextButtonGroupRowInformation(
                text = stringResource(R.string.screen_about_button_how_secure_messaging_works),
                onClick = { navController.navigate(CoverDropDestinations.HELP_HOW_SECURE_MESSAGING_WORKS) }
            ),
            ChevronTextButtonGroupRowInformation(
                text = stringResource(R.string.screen_about_button_faqs),
                onClick = { navController.navigate(CoverDropDestinations.HELP_FAQ) }
            ),
            ChevronTextButtonGroupRowInformation(
                text = stringResource(R.string.screen_about_button_privacy_policy),
                onClick = { navController.navigate(CoverDropDestinations.HELP_PRIVACY_POLICY) }
            ),
        ),
    )

    Text(
        text = stringResource(R.string.screen_about_headline_getting_started),
        style = MaterialTheme.typography.h2,
        modifier = Modifier.padding(top = Padding.XL, bottom = Padding.L),
    )
    ChevronTextButtonGroup(
        buttons = listOf(
            ChevronTextButtonGroupRowInformation(
                text = stringResource(R.string.screen_about_button_craft_your_first_message),
                onClick = { navController.navigate(CoverDropDestinations.HELP_CRAFT_MESSAGE_ROUTE) }
            ),
            ChevronTextButtonGroupRowInformation(
                text = stringResource(R.string.screen_about_button_keeping_passphrases_safe),
                onClick = { navController.navigate(CoverDropDestinations.HELP_KEEPING_PASSPHRASES_SAFE_ROUTE) }
            ),
        ),
    )

    Text(
        text = stringResource(R.string.screen_about_headline_as_the_conversation_progresses),
        style = MaterialTheme.typography.h2,
        modifier = Modifier.padding(top = Padding.XL, bottom = Padding.L),
    )
    ChevronTextButtonGroup(
        buttons = listOf(
            ChevronTextButtonGroupRowInformation(
                text = stringResource(R.string.screen_about_button_what_to_expect_as_a_reply),
                onClick = { navController.navigate(CoverDropDestinations.HELP_REPLY_EXPECTATIONS) }
            ),
            ChevronTextButtonGroupRowInformation(
                text = stringResource(R.string.screen_about_button_source_protection),
                onClick = { navController.navigate(CoverDropDestinations.HELP_SOURCE_PROTECTION) }
            ),
        ),
    )

    if (debugContext != null) {
        Text(
            text = stringResource(R.string.screen_about_headline_technical_information),
            style = MaterialTheme.typography.h2,
            modifier = Modifier.padding(top = Padding.XL, bottom = Padding.L),
        )
        Text(
            text = stringResource(R.string.screen_about_technical_information),
            style = MaterialTheme.typography.body1,
            modifier = Modifier.padding(bottom = Padding.L),
        )
        Text(
            text = debugContext.toString(),
            style = MaterialTheme.typography.body1,
            fontFamily = FontFamily.Monospace,
            fontSize = MaterialTheme.typography.body1.fontSize,
            modifier = Modifier.padding(bottom = Padding.L),
        )
    }
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun AboutScreenPreview() = CoverDropSurface {
    AboutScreen(navController = rememberNavController(), debugContext = null)
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun AboutScreenPreview_withDebugContext() = CoverDropSurface {
    AboutScreen(
        navController = rememberNavController(),
        debugContext = DebugContext(
            lastUpdatePublicKeys = Instant.now(),
            lastUpdateDeadDrops = Instant.now(),
            lastBackgroundTry = Instant.now(),
            lastBackgroundSend = Instant.now(),
            hashedOrgKey = "[abcdef ghijkl mnoqpr stu]"
        )
    )
}
