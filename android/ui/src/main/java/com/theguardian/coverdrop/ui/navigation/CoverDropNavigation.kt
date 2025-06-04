package com.theguardian.coverdrop.ui.navigation

import androidx.compose.runtime.Composable
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation.NavHostController
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import com.theguardian.coverdrop.ui.screens.AboutRoute
import com.theguardian.coverdrop.ui.screens.ContinueSessionRoute
import com.theguardian.coverdrop.ui.screens.ConversationRoute
import com.theguardian.coverdrop.ui.screens.EntryRoute
import com.theguardian.coverdrop.ui.screens.HelpCraftMessageScreen
import com.theguardian.coverdrop.ui.screens.HelpFaqScreen
import com.theguardian.coverdrop.ui.screens.HelpHowSecureMessagingWorksScreen
import com.theguardian.coverdrop.ui.screens.HelpKeepingPassphraseSafeScreen
import com.theguardian.coverdrop.ui.screens.HelpPrivacyPolicyScreen
import com.theguardian.coverdrop.ui.screens.HelpReplyExpectationsScreen
import com.theguardian.coverdrop.ui.screens.HelpSourceProtectionScreen
import com.theguardian.coverdrop.ui.screens.HelpWhyWeMadeSecureMessagingScreen
import com.theguardian.coverdrop.ui.screens.HowThisWorksRoute
import com.theguardian.coverdrop.ui.screens.InboxRoute
import com.theguardian.coverdrop.ui.screens.MessageSentRoute
import com.theguardian.coverdrop.ui.screens.NewMessageRoute
import com.theguardian.coverdrop.ui.screens.NewSessionRoute
import com.theguardian.coverdrop.ui.screens.RecipientSelectionRoute
import com.theguardian.coverdrop.ui.screens.SplashRoute
import com.theguardian.coverdrop.ui.viewmodels.SelectedRecipientViewModel

object CoverDropDestinations {
    const val SPLASH_ROUTE = "splash"
    const val ABOUT_ROUTE = "about"
    const val ENTRY_ROUTE = "entry"
    const val CONTINUE_SESSION_ROUTE = "continue_session"
    const val NEW_MESSAGE_ROUTE = "new_message"
    const val MESSAGE_SENT_ROUTE = "message_sent"
    const val NEW_SESSION_ROUTE = "new_session"
    const val HOW_THIS_WORKS_ROUTE = "how_this_works"
    const val RECIPIENT_SELECTION_ROUTE = "recipient_selection"

    const val INBOX_ROUTE = "inbox"
    const val CONVERSATION_ROUTE = "conversation/{id}"

    const val HELP_CRAFT_MESSAGE_ROUTE = "help/craft_message"
    const val HELP_KEEPING_PASSPHRASES_SAFE_ROUTE = "help/passphrase"
    const val HELP_REPLY_EXPECTATIONS = "help/reply_expectations"
    const val HELP_SOURCE_PROTECTION = "help/source_protection"
    const val HELP_PRIVACY_POLICY = "help/privacy_policy"
    const val HELP_WHY_WE_MADE_SECURE_MESSAGING = "help/why_we_made_secure_messaging"
    const val HELP_HOW_SECURE_MESSAGING_WORKS = "help/how_secure_messaging_words"
    const val HELP_FAQ = "help/faq"
}

@Composable
fun CoverDropNavGraph(
    navController: NavHostController,
    startDestination: String = CoverDropDestinations.SPLASH_ROUTE,
    sharedSelectedRecipientViewModel: SelectedRecipientViewModel = viewModel(),
) {
    NavHost(
        navController = navController,
        startDestination = startDestination,
    ) {
        composable(CoverDropDestinations.SPLASH_ROUTE) {
            SplashRoute(navController)
        }
        composable(CoverDropDestinations.ABOUT_ROUTE) {
            AboutRoute(navController)
        }
        composable(CoverDropDestinations.CONTINUE_SESSION_ROUTE) {
            ContinueSessionRoute(navController)
        }
        composable(CoverDropDestinations.ENTRY_ROUTE) {
            EntryRoute(navController)
        }
        composable(CoverDropDestinations.HOW_THIS_WORKS_ROUTE) {
            HowThisWorksRoute(navController)
        }

        composable(CoverDropDestinations.NEW_MESSAGE_ROUTE) {
            NewMessageRoute(sharedSelectedRecipientViewModel, navController)
        }
        composable(CoverDropDestinations.MESSAGE_SENT_ROUTE) {
            MessageSentRoute(navController)
        }
        composable(CoverDropDestinations.NEW_SESSION_ROUTE) {
            NewSessionRoute(navController)
        }
        composable(CoverDropDestinations.RECIPIENT_SELECTION_ROUTE) {
            RecipientSelectionRoute(sharedSelectedRecipientViewModel, navController)
        }

        composable(CoverDropDestinations.INBOX_ROUTE) {
            InboxRoute(navController)
        }
        composable(CoverDropDestinations.CONVERSATION_ROUTE) {
            ConversationRoute(navController)
        }

        composable(CoverDropDestinations.HELP_CRAFT_MESSAGE_ROUTE) {
            HelpCraftMessageScreen(navController)
        }
        composable(CoverDropDestinations.HELP_FAQ) {
            HelpFaqScreen(navController)
        }
        composable(CoverDropDestinations.HELP_HOW_SECURE_MESSAGING_WORKS) {
            HelpHowSecureMessagingWorksScreen(navController)
        }
        composable(CoverDropDestinations.HELP_KEEPING_PASSPHRASES_SAFE_ROUTE) {
            HelpKeepingPassphraseSafeScreen(navController)
        }
        composable(CoverDropDestinations.HELP_PRIVACY_POLICY) {
            HelpPrivacyPolicyScreen(navController)
        }
        composable(CoverDropDestinations.HELP_REPLY_EXPECTATIONS) {
            HelpReplyExpectationsScreen(navController)
        }
        composable(CoverDropDestinations.HELP_SOURCE_PROTECTION) {
            HelpSourceProtectionScreen(navController)
        }
        composable(CoverDropDestinations.HELP_WHY_WE_MADE_SECURE_MESSAGING) {
            HelpWhyWeMadeSecureMessagingScreen(navController)
        }
    }
}
