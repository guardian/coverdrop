package com.theguardian.coverdrop.ui.tests

import androidx.activity.compose.setContent
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.navigation.compose.ComposeNavigator
import androidx.navigation.testing.TestNavHostController
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.theguardian.coverdrop.ui.navigation.CoverDropDestinations
import com.theguardian.coverdrop.ui.navigation.CoverDropNavGraph
import com.theguardian.coverdrop.ui.tests.utils.performScrollToAndClick
import com.theguardian.coverdrop.ui.tests.utils.waitForNavigationTo
import com.theguardian.coverdrop.ui.theme.CoverDropSurface
import dagger.hilt.android.testing.HiltAndroidRule
import dagger.hilt.android.testing.HiltAndroidTest
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TestName
import org.junit.runner.RunWith

@HiltAndroidTest
@RunWith(AndroidJUnit4::class)
class AboutScreenTest {

    @get:Rule
    var hiltRule = HiltAndroidRule(this)

    @get:Rule
    val composeTestRule = createAndroidComposeRule<TestActivity>()

    @get:Rule
    val testName = TestName()

    private lateinit var navController: TestNavHostController

    @Before
    fun setupAppNavHost() {
        hiltRule.inject()

        composeTestRule.activity.setContent {
            CoverDropSurface {
                navController = TestNavHostController(LocalContext.current)
                navController.navigatorProvider.addNavigator(ComposeNavigator())
                CoverDropNavGraph(
                    navController = navController,
                    startDestination = CoverDropDestinations.ABOUT_ROUTE
                )
            }
        }
    }

    @Test
    fun whenLaunched_thenAboutScreenShown() {
        composeTestRule.onNodeWithText("About Secure Messaging").assertIsDisplayed()
    }

    /**
     * GIVEN the user is on the About screen
     * WHEN clicking why we made secure messaging
     * THEN then the corresponding help screen is shown
     */
    @Test
    fun whenClickingWhyWeMadeSecureMessaging_thenHelpScreenIsShown() {
        composeTestRule.onNodeWithText("Why we made Secure Messaging").performScrollToAndClick()
        composeTestRule.waitForNavigationTo(
            navController = navController,
            targetRoute = CoverDropDestinations.HELP_WHY_WE_MADE_SECURE_MESSAGING
        )
    }

    /**
     * GIVEN the user is on the About screen
     * WHEN clicking how secure messaging works
     * THEN then the corresponding help screen is shown
     */
    @Test
    fun whenClickingHowSecureMessagingWorks_thenHelpScreenIsShown() {
        composeTestRule.onNodeWithText("How Secure Messaging works").performScrollToAndClick()
        composeTestRule.waitForNavigationTo(
            navController = navController,
            targetRoute = CoverDropDestinations.HELP_HOW_SECURE_MESSAGING_WORKS
        )
    }

    /**
     * GIVEN the user is on the About screen
     * WHEN clicking FAQs
     * THEN then the corresponding help screen is shown
     */
    @Test
    fun whenClickingFAQs_thenHelpScreenIsShown() {
        composeTestRule.onNodeWithText("FAQs").performScrollToAndClick()
        composeTestRule.waitForNavigationTo(
            navController = navController,
            targetRoute = CoverDropDestinations.HELP_FAQ
        )
    }

    /**
     * GIVEN the user is on the About screen
     * WHEN clicking privacy policy
     * THEN then the corresponding help screen is shown
     */
    @Test
    fun whenClickingPrivacyPolicy_thenHelpScreenIsShown() {
        composeTestRule.onNodeWithText("Privacy policy").performScrollToAndClick()
        composeTestRule.waitForNavigationTo(
            navController = navController,
            targetRoute = CoverDropDestinations.HELP_PRIVACY_POLICY
        )
    }

    /**
     * GIVEN the user is on the About screen
     * WHEN clicking compose message
     * THEN then the corresponding help screen is shown
     */
    @Test
    fun whenClickingCraftMessage_thenHelpScreenIsShown() {
        composeTestRule.onNodeWithText("Compose your first message").performScrollToAndClick()
        composeTestRule.waitForNavigationTo(
            navController = navController,
            targetRoute = CoverDropDestinations.HELP_CRAFT_MESSAGE_ROUTE
        )
    }

    /**
     * GIVEN the user is on the About screen
     * WHEN clicking keeping passphrases safe
     * THEN the corresponding help screen is shown
     */
    @Test
    fun whenClickingKeepingPassphrasesSafe_thenHelpScreenIsShown() {
        composeTestRule.onNodeWithText("Keeping passphrases safe").performScrollToAndClick()
        composeTestRule.waitForNavigationTo(
            navController = navController,
            targetRoute = CoverDropDestinations.HELP_KEEPING_PASSPHRASES_SAFE_ROUTE
        )
    }

    /**
     * GIVEN the user is on the About screen
     * WHEN clicking what to expect as a reply
     * THEN the corresponding help screen is shown
     */
    @Test
    fun whenClickingWhatToExpectAsAReply_thenHelpScreenIsShown() {
        composeTestRule.onNodeWithText("What to expect as a reply").performScrollToAndClick()
        composeTestRule.waitForNavigationTo(
            navController = navController,
            targetRoute = CoverDropDestinations.HELP_REPLY_EXPECTATIONS
        )
    }

    /**
     * GIVEN the user is on the About screen
     * WHEN clicking source protection
     * THEN the corresponding help screen is shown
     */
    @Test
    fun whenClickingSourceProtection_thenHelpScreenIsShown() {
        composeTestRule.onNodeWithText("Source protection").performScrollToAndClick()
        composeTestRule.waitForNavigationTo(
            navController = navController,
            targetRoute = CoverDropDestinations.HELP_SOURCE_PROTECTION
        )
    }
}
