package com.theguardian.coverdrop.ui.tests

import androidx.activity.compose.setContent
import androidx.compose.ui.platform.LocalContext
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
import com.theguardian.coverdrop.core.models.JournalistInfo
import com.theguardian.coverdrop.core.models.JournalistVisibility
import com.theguardian.coverdrop.core.models.Message
import com.theguardian.coverdrop.core.models.MessageThread
import com.theguardian.coverdrop.ui.navigation.CoverDropDestinations
import com.theguardian.coverdrop.ui.navigation.CoverDropNavGraph
import com.theguardian.coverdrop.ui.tests.utils.CoverDropLibMock
import com.theguardian.coverdrop.ui.tests.utils.pressBack
import com.theguardian.coverdrop.ui.tests.utils.waitForNavigationTo
import com.theguardian.coverdrop.ui.tests.utils.waitUntilTextIsDisplayed
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
class InboxScreenTest {

    @get:Rule
    var hiltRule = HiltAndroidRule(this)

    @get:Rule
    val composeTestRule = createAndroidComposeRule<TestActivity>()

    @get:Rule
    val testName = TestName()

    @Inject
    lateinit var coverDropLib: ICoverDropLib

    private lateinit var navController: TestNavHostController

    private val journalistId = "1"
    private val journalistInfo = JournalistInfo(
        id = journalistId,
        displayName = "Charles Darwin",
        sortName = "Darwin Charles",
        description = "",
        isTeam = false,
        tag = "",
        visibility = JournalistVisibility.VISIBLE,
    )

    private val journalistId2 = "2"
    private val journalistInfo2 = JournalistInfo(
        id = journalistId2,
        displayName = "Joe Bloggs",
        sortName = "Bloggs Joe",
        description = "",
        isTeam = true,
        tag = "",
        visibility = JournalistVisibility.VISIBLE,
    )

    @Before
    fun setupAppNavHost() {
        hiltRule.inject()

        composeTestRule.activity.setContent {
            CoverDropSurface {
                navController = TestNavHostController(LocalContext.current)
                navController.navigatorProvider.addNavigator(ComposeNavigator())
                CoverDropNavGraph(
                    navController = navController,
                    startDestination = CoverDropDestinations.ENTRY_ROUTE,
                )
                navController.navigate(CoverDropDestinations.INBOX_ROUTE)
            }
        }
    }

    /**
     * GIVEN at least one conversation
     * WHEN the user open the inbox
     * THEN then the conversation is shown and navigates to the conversation screen
     */
    @Test
    fun whenHasConversation_thenShownAndClickable() {
        val mockedCoverDropLib = (coverDropLib as CoverDropLibMock)
        mockedCoverDropLib.getPrivateDataRepository().addConversationForId(
            id = journalistId2,
            thread = MessageThread(journalistInfo2, listOf(Message.sent("Hello Joe")))
        )
        mockedCoverDropLib.getPrivateDataRepository().addConversationForId(
            id = journalistId,
            thread = MessageThread(journalistInfo, listOf(Message.sent("Hello Charles")))
        )

        composeTestRule.waitUntilTextIsDisplayed("Messaging with")
        composeTestRule.waitUntilTextIsDisplayed("Charles Darwin")
        composeTestRule.waitUntilTextIsDisplayed("Joe Bloggs")

        composeTestRule.onNodeWithText("Charles Darwin").performClick()
        composeTestRule.waitForNavigationTo(
            navController = navController,
            targetRoute = CoverDropDestinations.CONVERSATION_ROUTE
        )
        composeTestRule.waitUntilTextIsDisplayed("Hello Charles")
    }

    /**
     * GIVEN no conversation
     * WHEN the user open the inbox
     * THEN then the empty screen is shown
     */
    @Test
    fun whenNoConversation_thenEmptyScreenShown() {
        composeTestRule.waitUntilTextIsDisplayed("You have no messages", allowSubstringMatch = true)

        // We also check the button for starting a new conversation
        composeTestRule.onNodeWithText("Send a new message").performClick()
        composeTestRule.waitForNavigationTo(navController, CoverDropDestinations.NEW_MESSAGE_ROUTE)
    }

    /**
     * GIVEN two conversation without any messages
     * WHEN the user open the inbox
     * THEN the conversation is shown and navigates to the conversation screen
     * AND the timestamp simply says "empty"
     */
    @Test
    fun whenNoMessages_thenEmptyConversationsShown() {
        val mockedCoverDropLib = (coverDropLib as CoverDropLibMock)
        mockedCoverDropLib.getPrivateDataRepository().addConversationForId(
            id = journalistId,
            thread = MessageThread(journalistInfo, emptyList())
        )
        mockedCoverDropLib.getPrivateDataRepository().addConversationForId(
            id = journalistId2,
            thread = MessageThread(journalistInfo2, emptyList())
        )

        composeTestRule.waitUntilTextIsDisplayed("Charles Darwin")
        composeTestRule.waitUntilTextIsDisplayed("No messages")

        composeTestRule.onNodeWithText("Charles Darwin").performClick()
        composeTestRule.waitForNavigationTo(
            navController = navController,
            targetRoute = CoverDropDestinations.CONVERSATION_ROUTE
        )
        composeTestRule.waitUntilTextIsDisplayed("There are no messages in this conversation. This might be because they expired after 14 days and have been automatically deleted.")
    }

    /**
     * GIVEN at least one conversation
     * WHEN the user follows the vault deletion flow
     * THEN the user is logged out and the session is locked
     */
    @Test
    fun whenDeleteVault_thenEmptyScreenShown() {
        val mockedCoverDropLib = (coverDropLib as CoverDropLibMock)
        mockedCoverDropLib.getPrivateDataRepository().addConversationForId(
            id = journalistId,
            thread = MessageThread(journalistInfo, listOf(Message.sent("hello")))
        )

        composeTestRule.waitUntilTextIsDisplayed("Charles Darwin")

        // Tease the UI a bit :)
        composeTestRule.onNodeWithText("Delete message vault").performClick()
        composeTestRule.waitUntilTextIsDisplayed("Delete your message vault?")
        composeTestRule.onNodeWithText("Keep").performClick()

        // Nothing should have changed, since we aborted
        composeTestRule.waitUntilTextIsDisplayed("Charles Darwin")

        composeTestRule.onNodeWithText("Delete message vault").performClick()
        composeTestRule.waitUntilTextIsDisplayed("Delete your message vault?")
        composeTestRule.onNodeWithText("Delete everything").performClick()

        // Confirm we are logged out and the session is locked
        composeTestRule.waitForNavigationTo(navController, CoverDropDestinations.ENTRY_ROUTE)
        composeTestRule.waitUntil(timeoutMillis = 1_000) {
            coverDropLib.getPrivateDataRepository().getLockState() == LockState.LOCKED
        }
    }

    /**
     * GIVEN we are in the inbox
     * WHEN clicking the "About" button
     * THEN the about screen is shown
     */
    @Test
    fun whenClickingAbout_thenAboutScreenShown() {
        composeTestRule.onNodeWithText("About Secure Messaging").performClick()
        composeTestRule.waitForNavigationTo(navController, CoverDropDestinations.ABOUT_ROUTE)
    }

    /**
     * GIVEN we are in an unlocked inbox
     * WHEN clicking the "Leave inbox" button
     * THEN the user is logged out and navigated back to the entry screen
     */
    @Test
    fun whenLeavingInboxViaLeaveInboxButton_thenLoggedOut() {
        val mockedCoverDropLib = (coverDropLib as CoverDropLibMock)
        runBlocking {
            mockedCoverDropLib.getPrivateDataRepository()
                .unlock(SampleDataProvider().getShortPassphrase())
            assertThat(coverDropLib.getPrivateDataRepository().getLockState())
                .isEqualTo(LockState.UNLOCKED)
        }

        composeTestRule.onNodeWithText("Leave message vault").performClick()
        composeTestRule.onNodeWithText("Log out").performClick()
        composeTestRule.waitForNavigationTo(navController, CoverDropDestinations.ENTRY_ROUTE)

        composeTestRule.waitUntil(timeoutMillis = 1_000) {
            coverDropLib.getPrivateDataRepository().getLockState() == LockState.LOCKED
        }
    }

    /**
     * GIVEN we are in an unlocked inbox
     * WHEN clicking the "X" icon in the top bar
     * THEN the user is logged out and navigated back to the entry screen
     */
    @Test
    fun whenLeavingInboxViaTopBar_thenLoggedOut() {
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
     * GIVEN we are in an unlocked inbox
     * WHEN the user pressed the back button
     * THEN the user is logged out and navigated back to the entry screen
     */
    @Test
    fun whenLeavingInboxViaBackButton_thenLoggedOut() {
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
