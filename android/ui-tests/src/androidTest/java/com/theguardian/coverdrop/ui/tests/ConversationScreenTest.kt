package com.theguardian.coverdrop.ui.tests

import androidx.activity.compose.setContent
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.test.assertIsEnabled
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.isDisplayed
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import androidx.navigation.compose.ComposeNavigator
import androidx.navigation.testing.TestNavHostController
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.ICoverDropLib
import com.theguardian.coverdrop.core.models.JournalistInfo
import com.theguardian.coverdrop.core.models.JournalistVisibility
import com.theguardian.coverdrop.core.models.Message
import com.theguardian.coverdrop.core.models.MessageThread
import com.theguardian.coverdrop.ui.navigation.CoverDropDestinations
import com.theguardian.coverdrop.ui.navigation.CoverDropNavGraph
import com.theguardian.coverdrop.ui.tests.utils.CoverDropLibMock
import com.theguardian.coverdrop.ui.tests.utils.waitUntilTagIsPresent
import com.theguardian.coverdrop.ui.tests.utils.waitUntilTextIsDisplayed
import com.theguardian.coverdrop.ui.theme.CoverDropSurface
import com.theguardian.coverdrop.ui.utils.COVERDROP_SAMPLE_DATA
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
class ConversationScreenTest {

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

    @Before
    fun setupAppNavHost() {
        hiltRule.inject()

        composeTestRule.activity.setContent {
            CoverDropSurface {
                navController = TestNavHostController(LocalContext.current)
                navController.navigatorProvider.addNavigator(ComposeNavigator())
                CoverDropNavGraph(
                    navController = navController, startDestination =
                        CoverDropDestinations.CONVERSATION_ROUTE.replace(
                            "{id}",
                            journalistId
                        )
                )
            }
        }
    }

    /**
     * AC-SC-1
     *
     * GIVEN the thread is empty
     * WHEN the user opens a conversation
     * THEN an empty state explanation is shown
     */
    @Test
    fun whenEmptyThread_thenEmptyStateTextShown() {
        val mockedCoverDropLib = (coverDropLib as CoverDropLibMock)
        mockedCoverDropLib.getPrivateDataRepository().addConversationForId(
            id = journalistId,
            thread = MessageThread(journalistInfo, emptyList())
        )

        composeTestRule.waitUntilTextIsDisplayed(
            "There are no messages in this conversation.",
            allowSubstringMatch = true
        )
    }

    /**
     * AC-SC-2
     *
     * GIVEN the thread is not empty
     * WHEN the user opens a conversation
     * THEN the thread is shown and includes both user messages and journalist replies
     */
    @Test
    fun whenNonEmptyThread_thenThreadMessagesAreShown() {
        val mockedCoverDropLib = (coverDropLib as CoverDropLibMock)
        mockedCoverDropLib.getPrivateDataRepository().addConversationForId(
            id = journalistId,
            thread = MessageThread(
                journalistInfo, listOf(
                    Message.sent("hello from user"),
                    Message.received("reply from journalist")
                )
            )
        )

        composeTestRule.waitUntilTextIsDisplayed("hello from user")
        composeTestRule.waitUntilTextIsDisplayed("reply from journalist")
    }

    /**
     * AC-SC-3
     *
     * GIVEN a conversation is opened
     * WHEN composing a short message
     * THEN progress bar changes and send button is enabled
     *
     * AC-SC-4
     *
     * GIVEN a conversation is opened
     * WHEN composing a too long message
     * THEN an error is shown and the send button is not enabled
     */
    @Test
    fun whenComposingMessage_thenProgressBarUpdatesAndSendEnabled_AND_whenTooLong_thenCannotSend() {
        val mockedCoverDropLib = (coverDropLib as CoverDropLibMock)
        mockedCoverDropLib.getPrivateDataRepository().addConversationForId(
            id = journalistId,
            thread = MessageThread(journalistInfo, emptyList())
        )

        // opens the composer
        composeTestRule.onNodeWithText("Send a new message").performClick()

        composeTestRule.waitUntilTagIsPresent("message_limit_indicator")
        composeTestRule.waitUntilTagIsPresent("edit_message")
        composeTestRule.onNodeWithText("Send message").assertIsNotEnabled()

        val limitBefore = composeTestRule.onNodeWithTag("message_limit_indicator")
            .fetchSemanticsNode()
            .config[SemanticsProperties.ProgressBarRangeInfo]
        assertThat(limitBefore.current).isLessThan(0.1f)

        // enter a short message
        composeTestRule.onNodeWithTag("edit_message").performClick()
        composeTestRule.onNodeWithTag("edit_message")
            .performTextInput("This is a medium-sized message going beyond 10% capacity.")

        composeTestRule.onNodeWithText("Send message").assertIsEnabled()
        val limitShortMessage = composeTestRule.onNodeWithTag("message_limit_indicator")
            .fetchSemanticsNode()
            .config[SemanticsProperties.ProgressBarRangeInfo]
        assertThat(limitShortMessage.current).isGreaterThan(0.1f)

        // enter a too long message
        composeTestRule.onNodeWithTag("edit_message")
            .performTextInput(COVERDROP_SAMPLE_DATA.getSampleMessage(300))

        composeTestRule.onNodeWithText("Send message").assertIsNotEnabled()
        val limitTooMuch = composeTestRule.onNodeWithTag("message_limit_indicator")
            .fetchSemanticsNode()
            .config[SemanticsProperties.ProgressBarRangeInfo]
        assertThat(limitTooMuch.current).isGreaterThan(0.9f)
    }

    /**
     * AC-SC-5
     *
     * GIVEN a message has been sent
     * WHEN user stays in the same conversation
     * THEN the new message is displayed in the thread
     */
    @Test
    fun whenMessageSent_thenThreadUpdates() {
        val mockedCoverDropLib = (coverDropLib as CoverDropLibMock)
        mockedCoverDropLib.getPrivateDataRepository().addConversationForId(
            id = journalistId,
            thread = MessageThread(
                journalistInfo, listOf(
                    Message.sent("hello from user"),
                    Message.received("reply from journalist")
                )
            )
        )

        // open the composer and send message
        composeTestRule.onNodeWithText("Send a new message").performClick()
        composeTestRule.onNodeWithTag("edit_message").performTextInput("new user message")
        composeTestRule.onNodeWithText("Send message").performClick()

        // observe new message in thread
        composeTestRule.waitUntilTextIsDisplayed("new user message")
    }

    /**
     * GIVEN there are many messages in the thread
     * WHEN the users navigates to the conversation
     * THEN the UI scrolls to the bottom of the thread
     */
    @Test
    fun whenManyMessages_thenScrollToBottom() {
        val mockedCoverDropLib = (coverDropLib as CoverDropLibMock)
        val messages = COVERDROP_SAMPLE_DATA.getSampleThread(numMessages = 20)
        mockedCoverDropLib.getPrivateDataRepository().addConversationForId(
            id = journalistId,
            thread = messages
        )

        runBlocking { composeTestRule.awaitIdle() }
        composeTestRule.onNodeWithText("id19", substring = true).isDisplayed()
    }

    /**
     * GIVEN there are many messages in the thread
     * WHEN the user sends a new message
     * THEN the UI scrolls to the bottom of the thread
     */
    @Test
    fun whenSendingNewMessage_thenScrollToBottom() {
        val mockedCoverDropLib = (coverDropLib as CoverDropLibMock)
        val messages = COVERDROP_SAMPLE_DATA.getSampleThread(numMessages = 20)
        mockedCoverDropLib.getPrivateDataRepository().addConversationForId(
            id = journalistId,
            thread = messages
        )

        runBlocking { composeTestRule.awaitIdle() }
        composeTestRule.onNodeWithText("Send a new message").performClick()
        composeTestRule.onNodeWithTag("edit_message").performTextInput("new user message")
        composeTestRule.onNodeWithText("Send message").performClick()

        composeTestRule.waitUntilTextIsDisplayed("new user message")
    }
}
