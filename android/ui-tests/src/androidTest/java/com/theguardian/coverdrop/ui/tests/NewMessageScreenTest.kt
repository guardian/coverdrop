package com.theguardian.coverdrop.ui.tests

import android.annotation.SuppressLint
import androidx.activity.compose.setContent
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.test.assertIsEnabled
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.assertTextContains
import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performTextInput
import androidx.navigation.compose.ComposeNavigator
import androidx.navigation.testing.TestNavHostController
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.ICoverDropLib
import com.theguardian.coverdrop.core.LockState
import com.theguardian.coverdrop.ui.navigation.CoverDropDestinations
import com.theguardian.coverdrop.ui.navigation.CoverDropNavGraph
import com.theguardian.coverdrop.ui.tests.utils.CoverDropLibMock
import com.theguardian.coverdrop.ui.tests.utils.CoverDropPrivateDataRepositoryMock.SimulatedBehaviour.FAIL_ON_CREATE_NEW_CONVERSATION
import com.theguardian.coverdrop.ui.tests.utils.CoverDropPublicDataRepositoryMock
import com.theguardian.coverdrop.ui.tests.utils.CoverDropPublicDataRepositoryMock.SimulatedBehaviour.ONLY_ONE_JOURNALIST_AVAILABLE
import com.theguardian.coverdrop.ui.tests.utils.performScrollToAndClick
import com.theguardian.coverdrop.ui.tests.utils.pressBack
import com.theguardian.coverdrop.ui.tests.utils.randomString
import com.theguardian.coverdrop.ui.tests.utils.waitForNavigationTo
import com.theguardian.coverdrop.ui.theme.CoverDropSurface
import com.theguardian.coverdrop.ui.utils.COVERDROP_SAMPLE_DATA
import com.theguardian.coverdrop.ui.utils.SampleDataProvider
import com.theguardian.coverdrop.ui.viewmodels.SelectedRecipientState.SingleRecipientForced
import com.theguardian.coverdrop.ui.viewmodels.SelectedRecipientViewModel
import dagger.hilt.android.testing.HiltAndroidRule
import dagger.hilt.android.testing.HiltAndroidTest
import kotlinx.coroutines.delay
import kotlinx.coroutines.runBlocking
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TestName
import org.junit.runner.RunWith
import javax.inject.Inject

@HiltAndroidTest
@RunWith(AndroidJUnit4::class)
class NewMessageScreenTest {

    @get:Rule
    var hiltRule = HiltAndroidRule(this)

    @get:Rule
    val composeTestRule = createAndroidComposeRule<TestActivity>()

    @get:Rule
    val testName = TestName()

    @Inject
    lateinit var coverDropLib: ICoverDropLib

    private lateinit var navController: TestNavHostController
    private lateinit var sharedSelectedRecipientViewModel: SelectedRecipientViewModel

    @Before
    @SuppressLint("ViewModelConstructorInComposable")
    fun setupAppNavHost() {
        hiltRule.inject()

        composeTestRule.activity.setContent {
            CoverDropSurface {
                sharedSelectedRecipientViewModel =
                    SelectedRecipientViewModel(coverDropLib.getPublicDataRepository())
                navController = TestNavHostController(LocalContext.current)
                navController.navigatorProvider.addNavigator(ComposeNavigator())
                CoverDropNavGraph(
                    navController = navController,
                    startDestination = CoverDropDestinations.ENTRY_ROUTE,
                    sharedSelectedRecipientViewModel = sharedSelectedRecipientViewModel,
                )
                navController.navigate(CoverDropDestinations.NEW_MESSAGE_ROUTE)
            }
        }

        // ensure that lateinit variables are ready
        runBlocking { composeTestRule.awaitIdle() }
    }

    /**
     * AC-RS-1
     *
     * GIVEN the user visits the new message screen
     * WHEN press the change recipient button
     * THEN the recipient picker is shown
     */
    @Test
    fun whenClickingChangeRecipient_thenRecipientSelectionScreenIsShown() {
        composeTestRule.onNodeWithTag("edit_recipient").performClick()

        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.RECIPIENT_SELECTION_ROUTE
        )
    }

    /**
     *
     * GIVEN the user visits the new message screen
     * WHEN the user enter a message into the message box which is an ok compressed size
     * THEN then the error message "Please shorten your message" does not appear
     * AND the button is enabled
     */
    @Test
    fun whenEnteringShortMessage_thenErrorMessageNotShown() {
        val nodeEditMessage = composeTestRule.onNodeWithTag("edit_message")

        nodeEditMessage.performTextInput(randomString(32))

        composeTestRule.onNodeWithText("Send message").assertIsEnabled()

        composeTestRule.onNodeWithText("Please shorten your message", substring = true)
            .assertDoesNotExist()
    }

    /**
     * AC-RS-2
     *
     * GIVEN the user visits the new message screen
     * WHEN the user enter a message into the message box longer than the maximum (calculated in compressed bytes)
     * THEN then the message appears "Please shorten your message"
     * AND the button is disabled
     */
    @Test
    fun whenEnteringTooLongMessage_thenErrorMessageShown() {
        val nodeEditMessage = composeTestRule.onNodeWithTag("edit_message")

        nodeEditMessage.performTextInput(randomString(1024))

        composeTestRule.onNodeWithText("Send message").assertIsNotEnabled()

        composeTestRule.onNodeWithText("Please shorten your message", substring = true)
            .assertExists()
    }

    /**
     * AC-RS-5
     *
     * GIVEN the user visits the new message screen and the user has entered all valid values into the recipient, name and message fields
     * WHEN the user presses the send message button
     * THEN the user is taken to confirmation screen
     */
    @Test
    fun whenAllEnteredAndSubmit_thenMessageIsSentAndNavigateToInbox() {
        val recipient = COVERDROP_SAMPLE_DATA.getTeams().first()
        sharedSelectedRecipientViewModel.selectRecipient(recipient)

        val nodeEditMessage = composeTestRule.onNodeWithTag("edit_message")
        nodeEditMessage.performTextInput(COVERDROP_SAMPLE_DATA.getSampleMessage())

        composeTestRule.onNodeWithText("Send message").performScrollToAndClick()

        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.MESSAGE_SENT_ROUTE
        )
    }

    /**
     * GIVEN the screen is open
     * WHEN the user clicks the help banner
     * THEN the user is navigated to the respective help screen
     *
     * AND THEN
     *
     * GIVEN that help screen is shown
     * WHEN the user clicks the button for information on source protection
     * THEN the user is navigated to the respective help screen
     */
    @Test
    fun whenBannerClicked_thenNavigatesToHelpCraftMessageScreen() {
        composeTestRule.onNodeWithText("Compose your first message").performClick()
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.HELP_CRAFT_MESSAGE_ROUTE
        )

        composeTestRule.onNodeWithText("Source protection").performScrollTo()

        composeTestRule.onNodeWithText("Source protection").performClick()
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.HELP_SOURCE_PROTECTION
        )
    }

    /**
     * GIVEN there is a recipient selected and some text typed
     * WHEN the user navigates to the help screen and back
     * THEN the local state is preserved
     */
    @Test
    fun whenNavigatingToHelpAndBack_thenMessageIsNotLost() {
        val expectedRecipient = COVERDROP_SAMPLE_DATA.getTeams().last()
        val expectedMessage = COVERDROP_SAMPLE_DATA.getSampleMessage()
        sharedSelectedRecipientViewModel.selectRecipient(expectedRecipient)

        // Edit message
        val nodeEditMessage = composeTestRule.onNodeWithTag("edit_message")
        nodeEditMessage.performTextInput(expectedMessage)

        // Navigate away and back
        composeTestRule.onNodeWithText("Compose your first message").performClick()
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.HELP_CRAFT_MESSAGE_ROUTE
        )
        composeTestRule.pressBack()

        // Check recipient and text are still there
        composeTestRule.onNodeWithTag("edit_recipient")
            .assertTextContains(expectedRecipient.displayName)
        composeTestRule.onNodeWithTag("edit_message").assertTextEquals(expectedMessage)
    }

    /**
     * GIVEN the user has selected a recipient and entered a message
     * WHEN the sending operation fails due to an internal problem
     * THEN the user is shown an error message
     */
    @Test
    fun whenSendingFails_thenErrorMessageIsShown() {
        val recipient = COVERDROP_SAMPLE_DATA.getTeams().first()
        sharedSelectedRecipientViewModel.selectRecipient(recipient)

        val nodeEditMessage = composeTestRule.onNodeWithTag("edit_message")
        nodeEditMessage.performTextInput(COVERDROP_SAMPLE_DATA.getSampleMessage())

        val mockedPrivateRepo = (coverDropLib as CoverDropLibMock).getPrivateDataRepository()
        try {
            mockedPrivateRepo.simulateBehaviour(FAIL_ON_CREATE_NEW_CONVERSATION)

            composeTestRule.onNodeWithText("Send message").performScrollToAndClick()
            runBlocking {
                delay(200);
                composeTestRule.awaitIdle();
                delay(200);
            }
            composeTestRule.onNodeWithText(
                "Failed to create a new conversation",
                substring = true,
            ).assertExists()
        } finally {
            mockedPrivateRepo.clearSimulatedBehaviours()
        }
    }

    /**
     * GIVEN there is only one journalist available (i.e. beta recipient forced scenario)
     * WHEN clicking on the recipient selection button
     * THEN a popup is shown explaining that the user can only send to the beta recipient
     */
    @Test
    fun whenBetaRecipient_thenPopupExplainsBetaRecipient() {
        val publicDataRepo =
            coverDropLib.getPublicDataRepository() as CoverDropPublicDataRepositoryMock
        try {
            publicDataRepo.simulateBehaviour(ONLY_ONE_JOURNALIST_AVAILABLE)

            // force reloading from the repository
            sharedSelectedRecipientViewModel.forceReinitialize()
            composeTestRule.waitUntil {
                sharedSelectedRecipientViewModel.getSelectedRecipient().value is SingleRecipientForced
            }

            val betaRecipient = runBlocking { publicDataRepo.getAllJournalists().single() }
            sharedSelectedRecipientViewModel.selectRecipient(betaRecipient)

            composeTestRule.onNodeWithTag("edit_recipient").performClick()
            // there is unfortunately no easy way to test whether a toast message is shown
            Thread.sleep(250)
        } finally {
            publicDataRepo.clearSimulatedBehaviours()
        }
    }

    /**
     * GIVEN we are writing a new message
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
     * GIVEN we are writing a new message
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
