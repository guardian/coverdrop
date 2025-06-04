package com.theguardian.coverdrop.ui.tests

import androidx.activity.compose.setContent
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.navigation.compose.ComposeNavigator
import androidx.navigation.testing.TestNavHostController
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.ICoverDropLib
import com.theguardian.coverdrop.core.LockState
import com.theguardian.coverdrop.ui.navigation.CoverDropDestinations
import com.theguardian.coverdrop.ui.navigation.CoverDropNavGraph
import com.theguardian.coverdrop.ui.tests.utils.CoverDropLibMock
import com.theguardian.coverdrop.ui.tests.utils.performScrollToAndClick
import com.theguardian.coverdrop.ui.tests.utils.pressBack
import com.theguardian.coverdrop.ui.tests.utils.waitForNavigationTo
import com.theguardian.coverdrop.ui.theme.CoverDropSurface
import com.theguardian.coverdrop.ui.utils.SampleDataProvider
import dagger.hilt.android.testing.HiltAndroidRule
import dagger.hilt.android.testing.HiltAndroidTest
import kotlinx.coroutines.runBlocking
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TestName
import org.junit.runner.RunWith
import javax.inject.Inject

@HiltAndroidTest
@RunWith(AndroidJUnit4::class)
class MessageSentScreenTest {

    @get:Rule
    var hiltRule = HiltAndroidRule(this)

    @get:Rule
    val composeTestRule = createAndroidComposeRule<TestActivity>()

    @get:Rule
    val testName = TestName()

    @Inject
    lateinit var coverDropLib: ICoverDropLib

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
                    startDestination = CoverDropDestinations.ENTRY_ROUTE
                )
                navController.navigate(CoverDropDestinations.MESSAGE_SENT_ROUTE)
            }
        }
    }

    @Test
    fun whenLaunched_thenYourMessageSentScreenShown() {
        composeTestRule.onNodeWithText("Your message will be received by a journalist soon")
            .assertIsDisplayed()
    }

    /**
     * AC-MS-1
     *
     * GIVEN the user visits the message sent screen
     * WHEN the user clicks "Go to your inbox"
     * THEN they are shown the inbox
     *
     */
    @Test
    fun whenClickGoToInbox_thenNavigateToInbox_andCanReturn() {
        composeTestRule.onNodeWithText("Review conversation").performClick()

        composeTestRule.waitForNavigationTo(navController, CoverDropDestinations.INBOX_ROUTE)
    }

    /**
     * GIVEN the screen is open
     * WHEN the user clicks the help button
     * THEN the user is navigated to the respective help screen
     */
    @Test
    fun whenBannerClicked_thenNavigatesToHelpCraftMessageScreen() {
        composeTestRule.onNodeWithText("What to expect as a reply").performScrollToAndClick()
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.HELP_REPLY_EXPECTATIONS
        )
    }

    /**
     * GIVEN the we are on the message sent screen
     * WHEN clicking the "Leave inbox" button
     * THEN the user is logged out and navigated back to the entry screen
     */
    @Test
    fun whenLeavingViaLeaveInboxButton_thenLoggedOut() {
        val mockedCoverDropLib = (coverDropLib as CoverDropLibMock)
        runBlocking {
            mockedCoverDropLib.getPrivateDataRepository()
                .unlock(SampleDataProvider().getShortPassphrase())
            assertThat(coverDropLib.getPrivateDataRepository().getLockState())
                .isEqualTo(LockState.UNLOCKED)
        }

        composeTestRule.onNodeWithText("Log out from Secure Messaging").performClick()
        composeTestRule.onNodeWithText("Log out").performClick()
        composeTestRule.waitForNavigationTo(navController, CoverDropDestinations.ENTRY_ROUTE)

        composeTestRule.waitUntil(timeoutMillis = 1_000) {
            coverDropLib.getPrivateDataRepository().getLockState() == LockState.LOCKED
        }
    }

    /**
     * GIVEN the we are on the message sent screen
     * WHEN clicking the "X" icon in the top bar
     * THEN the user is logged out and navigated back to the entry screen
     */
    @Test
    fun whenLeavingViaTopBar_thenLoggedOut() {
        val mockedCoverDropLib = (coverDropLib as CoverDropLibMock)
        runBlocking {
            mockedCoverDropLib.getPrivateDataRepository()
                .unlock(SampleDataProvider().getShortPassphrase())
            assertThat(coverDropLib.getPrivateDataRepository().getLockState())
                .isEqualTo(LockState.UNLOCKED)
        }

        composeTestRule.onNodeWithTag("top_bar_navigation_action").performClick()
        composeTestRule.onNodeWithText("Log out").performClick()
        composeTestRule.waitForNavigationTo(navController, CoverDropDestinations.ENTRY_ROUTE)

        composeTestRule.waitUntil(timeoutMillis = 1_000) {
            coverDropLib.getPrivateDataRepository().getLockState() == LockState.LOCKED
        }
    }

    /**
     * GIVEN the we are on the message sent screen
     * WHEN the user pressed the back button
     * THEN the user is logged out and navigated back to the entry screen
     */
    @Test
    fun whenLeavingViaBackButton_thenLoggedOut() {
        val mockedCoverDropLib = (coverDropLib as CoverDropLibMock)
        runBlocking {
            mockedCoverDropLib.getPrivateDataRepository()
                .unlock(SampleDataProvider().getShortPassphrase())
            assertThat(coverDropLib.getPrivateDataRepository().getLockState())
                .isEqualTo(LockState.UNLOCKED)
        }

        runBlocking { composeTestRule.awaitIdle() }
        composeTestRule.pressBack()
        composeTestRule.onNodeWithText("Log out").performClick()
        composeTestRule.waitForNavigationTo(navController, CoverDropDestinations.ENTRY_ROUTE)

        composeTestRule.waitUntil(timeoutMillis = 1_000) {
            coverDropLib.getPrivateDataRepository().getLockState() == LockState.LOCKED
        }
    }
}
