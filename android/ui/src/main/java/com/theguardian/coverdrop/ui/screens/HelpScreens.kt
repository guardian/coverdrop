package com.theguardian.coverdrop.ui.screens

import android.content.Context
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.material.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import androidx.navigation.NavHostController
import androidx.navigation.compose.rememberNavController
import com.theguardian.coverdrop.ui.R
import com.theguardian.coverdrop.ui.components.CoverDropTopAppBar
import com.theguardian.coverdrop.ui.components.HelpScreenComponent
import com.theguardian.coverdrop.ui.components.HelpScreenContent
import com.theguardian.coverdrop.ui.components.parseHelpScreenMarkup
import com.theguardian.coverdrop.ui.navigation.CoverDropDestinations
import com.theguardian.coverdrop.ui.theme.CoverDropSurface


@Composable
fun HelpCraftMessageScreen(navController: NavHostController) = HelpScreen(
    navController = navController,
    helpTextResId = R.raw.help_craft_message,
    onClickMapping = mapOf(
        Pair("button_source_protection") { navController.navigate(CoverDropDestinations.HELP_SOURCE_PROTECTION) },
    )
)

@Composable
fun HelpFaqScreen(navController: NavHostController) = HelpScreen(
    navController = navController,
    helpTextResId = R.raw.help_faq,
    onClickMapping = mapOf(
        Pair("button_how_secure_messaging_work") { navController.navigate(CoverDropDestinations.HELP_HOW_SECURE_MESSAGING_WORKS) },
        Pair("button_privacy_policy") { navController.navigate(CoverDropDestinations.HELP_PRIVACY_POLICY) },
        Pair("button_what_to_expect_as_a_reply") { navController.navigate(CoverDropDestinations.HELP_REPLY_EXPECTATIONS) },
    )
)

@Composable
fun HelpHowSecureMessagingWorksScreen(navController: NavHostController) = HelpScreen(
    navController = navController,
    helpTextResId = R.raw.help_how_secure_messaging_works,
    onClickMapping = mapOf(
        Pair("button_faq") { navController.navigate(CoverDropDestinations.HELP_FAQ) },
    )
)

@Composable
fun HelpKeepingPassphraseSafeScreen(navController: NavHostController) = HelpScreen(
    navController = navController,
    helpTextResId = R.raw.help_keeping_passphrase_safe,
)

@Composable
fun HelpPrivacyPolicyScreen(navController: NavHostController) = HelpScreen(
    navController = navController,
    helpTextResId = R.raw.help_privacy_policy,
    onClickMapping = mapOf(
        Pair("button_help_keeping_passphrase_safe") { navController.navigate(CoverDropDestinations.HELP_KEEPING_PASSPHRASES_SAFE_ROUTE) },
    )
)

@Composable
fun HelpReplyExpectationsScreen(navController: NavHostController) = HelpScreen(
    navController = navController,
    helpTextResId = R.raw.help_reply_expectations,
)

@Composable
fun HelpSourceProtectionScreen(navController: NavHostController) = HelpScreen(
    navController = navController,
    helpTextResId = R.raw.help_source_protection,
)

@Composable
fun HelpWhyWeMadeSecureMessagingScreen(navController: NavHostController) = HelpScreen(
    navController = navController,
    helpTextResId = R.raw.help_why_we_made_secure_messaging,
    onClickMapping = mapOf(
        Pair("button_how_secure_messaging_works") { navController.navigate(CoverDropDestinations.HELP_HOW_SECURE_MESSAGING_WORKS) },
    )
)

@Composable
private fun HelpScreen(
    navController: NavHostController,
    helpTextResId: Int,
    onClickMapping: Map<String, () -> Unit> = emptyMap(),
) {
    Column(modifier = Modifier.fillMaxHeight(1f)) {
        CoverDropTopAppBar(onNavigationOptionPressed = { navController.navigateUp() })
        val components = loadHelpScreenComponents(
            context = LocalContext.current,
            resId = helpTextResId,
            highlightColor = MaterialTheme.colors.primary,
        )
        HelpScreenContent(components, onClickMapping = onClickMapping)
    }
}

private fun loadHelpScreenComponents(
    context: Context,
    resId: Int,
    highlightColor: Color,
): List<HelpScreenComponent> {
    val file = context.resources.openRawResource(resId)
    val content = file.bufferedReader().readText()
    return parseHelpScreenMarkup(content, highlightColor)
}

@Preview(device = Devices.PIXEL_6, heightDp = 2000)
@Composable
private fun HelpCraftMessageScreenPreview() = CoverDropSurface {
    HelpCraftMessageScreen(navController = rememberNavController())
}

@Preview(device = Devices.PIXEL_6, heightDp = 2000)
@Composable
private fun HelpFaqScreenPreview() = CoverDropSurface {
    HelpFaqScreen(navController = rememberNavController())
}

@Preview(device = Devices.PIXEL_6, heightDp = 2000)
@Composable
private fun HelpHowSecureMessagingWorksPreview() = CoverDropSurface {
    HelpHowSecureMessagingWorksScreen(navController = rememberNavController())
}

@Preview(device = Devices.PIXEL_6, heightDp = 2000)
@Composable
private fun HelpKeepingPassphraseSafeScreenPreview() = CoverDropSurface {
    HelpKeepingPassphraseSafeScreen(navController = rememberNavController())
}

@Preview(device = Devices.PIXEL_6, heightDp = 2000)
@Composable
private fun HelpReplyExpectationsScreenPreview() = CoverDropSurface {
    HelpReplyExpectationsScreen(navController = rememberNavController())
}

@Preview(device = Devices.PIXEL_6, heightDp = 2000)
@Composable
private fun HelpSourceProtectionScreenPreview() = CoverDropSurface {
    HelpSourceProtectionScreen(navController = rememberNavController())
}

@Preview(device = Devices.PIXEL_6, heightDp = 2000)
@Composable
private fun HelpWhyWeMadeSecureMessagingPreview() = CoverDropSurface {
    HelpWhyWeMadeSecureMessagingScreen(navController = rememberNavController())
}

@Preview(device = Devices.PIXEL_6, heightDp = 2000)
@Composable
private fun HelpPrivacyPolicyScreenPreview() = CoverDropSurface {
    HelpPrivacyPolicyScreen(navController = rememberNavController())
}
