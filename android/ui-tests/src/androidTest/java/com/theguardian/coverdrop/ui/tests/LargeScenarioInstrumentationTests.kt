package com.theguardian.coverdrop.ui.tests

import android.util.Log
import androidx.activity.compose.setContent
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.filterToOne
import androidx.compose.ui.test.hasClickAction
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onSiblings
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performTextInput
import androidx.navigation.compose.ComposeNavigator
import androidx.navigation.testing.TestNavHostController
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.CoverDropConfiguration
import com.theguardian.coverdrop.core.ICoverDropLib
import com.theguardian.coverdrop.core.ui.models.DraftMessage
import com.theguardian.coverdrop.core.ui.models.UiPassphraseWord
import com.theguardian.coverdrop.core.ui.models.toUiPassphrase
import com.theguardian.coverdrop.ui.navigation.CoverDropDestinations
import com.theguardian.coverdrop.ui.navigation.CoverDropNavGraph
import com.theguardian.coverdrop.ui.tests.utils.CoverDropPrivateDataRepositoryMock
import com.theguardian.coverdrop.ui.tests.utils.CoverDropPublicDataRepositoryMock
import com.theguardian.coverdrop.ui.tests.utils.CoverDropPublicDataRepositoryMock.SimulatedBehaviour.NO_DEFAULT_JOURNALIST
import com.theguardian.coverdrop.ui.tests.utils.MockedPassphraseBehavior
import com.theguardian.coverdrop.ui.tests.utils.performCoverDropEnterPassphraseWords
import com.theguardian.coverdrop.ui.tests.utils.performScrollToAndClick
import com.theguardian.coverdrop.ui.tests.utils.pressBack
import com.theguardian.coverdrop.ui.tests.utils.waitForNavigationTo
import com.theguardian.coverdrop.ui.tests.utils.waitUntilTagIsPresent
import com.theguardian.coverdrop.ui.tests.utils.waitUntilTextIsDisplayed
import com.theguardian.coverdrop.ui.theme.CoverDropSurface
import dagger.hilt.android.testing.HiltAndroidRule
import dagger.hilt.android.testing.HiltAndroidTest
import kotlinx.coroutines.runBlocking
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TestName
import org.junit.runner.RunWith
import javax.inject.Inject

const val SESSION_UNLOCK_TIMEOUT_MS = 45_000L

@HiltAndroidTest
@RunWith(AndroidJUnit4::class)
class LargeScenarioInstrumentationTests {

    @get:Rule
    var hiltRule = HiltAndroidRule(this)

    @get:Rule
    val composeTestRule = createAndroidComposeRule<TestActivity>()

    @get:Rule
    val testName = TestName()

    @Inject
    lateinit var coverDropConfiguration: CoverDropConfiguration

    @Inject
    lateinit var lib: ICoverDropLib

    private lateinit var navController: TestNavHostController
    private lateinit var publicDataRespository: CoverDropPublicDataRepositoryMock
    private lateinit var privateDataRepository: CoverDropPrivateDataRepositoryMock

    @Before
    fun setup() {
        hiltRule.inject()
        composeTestRule.activity.setContent {
            CoverDropSurface {
                navController = TestNavHostController(LocalContext.current)
                navController.navigatorProvider.addNavigator(ComposeNavigator())
                CoverDropNavGraph(navController = navController)
            }
        }

        // we require random passphrases for this test
        privateDataRepository = lib.getPrivateDataRepository() as CoverDropPrivateDataRepositoryMock
        privateDataRepository.setPassphraseBehavior(MockedPassphraseBehavior.RANDOM)

        // we require no default journalist for this test
        publicDataRespository = lib.getPublicDataRepository() as CoverDropPublicDataRepositoryMock
        publicDataRespository.simulateBehaviour(NO_DEFAULT_JOURNALIST)
    }

    @Test
    fun testStorageCreationAndUnlockingWithNewConversation() {
        // ensure app is started
        composeTestRule.waitUntilTextIsDisplayed("Get started")

        // enter the new session flow
        composeTestRule.onNodeWithText("Get started").performClick()

        // confirm dialog
        composeTestRule.onNodeWithText("Yes, start a new conversation").performClick()
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.HOW_THIS_WORKS_ROUTE
        )

        // go through tutorial screen
        composeTestRule.onNodeWithText("Continue").performClick()
        composeTestRule.onNodeWithText("Continue").performClick()
        composeTestRule.onNodeWithText("Set up my passphrase").performClick()
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.NEW_SESSION_ROUTE
        )

        // switch passphrase to visible
        composeTestRule.onNodeWithTag("primary_button_reveal_passphrase").performClick()
        composeTestRule.waitForIdle()

        // extract passphrase from views
        val passphraseWords = getPassphraseFromUi()
        Log.w("LargeScenario", "passphraseWords=$passphraseWords")

        // advance to confirmation screen
        composeTestRule.onNodeWithText("I have remembered my passphrase").performClick()
        composeTestRule.waitUntilTextIsDisplayed("Enter passphrase")

        // enter the passphrases
        composeTestRule.performCoverDropEnterPassphraseWords(passphraseWords)

        // confirm and wait for session to unlock and new message screen to show
        composeTestRule.onNodeWithText("Confirm passphrase").performScrollToAndClick()
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.NEW_MESSAGE_ROUTE
        )

        // change recipient
        composeTestRule.waitUntilTextIsDisplayed("no recipient selected")
        composeTestRule.onNodeWithText("no recipient selected").performClick()
        composeTestRule.onNodeWithText("Journalists").performClick()
        composeTestRule.onNodeWithText("Alice")
            .onSiblings()
            .filterToOne(hasClickAction())
            .performClick()

        // add message and send
        val message = "A unique message string"
        composeTestRule.onNodeWithTag("edit_message").performScrollToAndClick()
        composeTestRule.waitForIdle()
        Thread.sleep(250)
        composeTestRule.onNodeWithTag("edit_message").performTextInput(message)

        composeTestRule.onNodeWithText("Send message").performScrollToAndClick()

        // pass confirmation screen
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.MESSAGE_SENT_ROUTE
        )
        composeTestRule.onNodeWithText("Review conversation").performClick()

        // wait for inbox
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.INBOX_ROUTE
        )
        composeTestRule.waitUntilTextIsDisplayed("Messaging with")

        // open the conversation
        composeTestRule.onNodeWithText("Alice").performClick()
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.CONVERSATION_ROUTE
        )

        // make sure message is shown
        composeTestRule.waitUntilTextIsDisplayed(message)

        // send another message
        val message2 = "Another unique message string"
        composeTestRule.onNodeWithText("Send a new message").performClick()
        composeTestRule.waitForIdle()
        Thread.sleep(250)
        composeTestRule.onNodeWithTag("edit_message").performTextInput(message2)
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithText("Send message").performClick()

        // check it is shown
        composeTestRule.waitForIdle()
        composeTestRule.waitUntilTextIsDisplayed(message2)

        // go back to inbox
        composeTestRule.pressBack()
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.INBOX_ROUTE
        )

        // logout using the leave message vault button
        composeTestRule.onNodeWithText("Leave message vault").performClick()
        composeTestRule.onNodeWithText("Log out").performClick()
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.ENTRY_ROUTE
        )

        // login using the continue session flow
        composeTestRule.onNodeWithText("Check your message vault").performClick()
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.CONTINUE_SESSION_ROUTE
        )

        // enter the same passphrase (there are some extra wait for the scroll and focus change
        // animations to finish)
        composeTestRule.performCoverDropEnterPassphraseWords(passphraseWords)

        // confirm and wait for session to unlock and new message screen to show
        composeTestRule.onNodeWithText("Confirm Passphrase").performScrollToAndClick()
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.INBOX_ROUTE,
            timeoutMs = SESSION_UNLOCK_TIMEOUT_MS
        )

        // check in the conversation that both messages are still there
        composeTestRule.onNodeWithText("Alice").performClick()
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.CONVERSATION_ROUTE
        )
        composeTestRule.waitUntilTextIsDisplayed(message)
        composeTestRule.waitUntilTextIsDisplayed(message2)

        // go back to the inbox and delete the messaging vault
        composeTestRule.runOnUiThread { navController.popBackStack() }
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.INBOX_ROUTE
        )
        composeTestRule.onNodeWithText("Delete message vault").performClick()
        composeTestRule.waitUntilTextIsDisplayed("Delete your message vault?")
        composeTestRule.onNodeWithText("Delete everything").performClick()

        // check that we are back at the entry screen
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.ENTRY_ROUTE
        )

        // try to login again (this should now fail)
        composeTestRule.onNodeWithText("Check your message vault").performClick()
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.CONTINUE_SESSION_ROUTE
        )

        // enter the same passphrase (there are some extra wait for the scroll and focus change
        // animations to finish)
        composeTestRule.performCoverDropEnterPassphraseWords(passphraseWords)
        composeTestRule.onNodeWithText("Confirm Passphrase").performScrollToAndClick()

        // wait for error message to appear
        composeTestRule.waitUntilTagIsPresent("warning_box_text")
        composeTestRule.onNodeWithTag("warning_box_text").performScrollTo()
        composeTestRule
            .onNodeWithText("Failed to open message vault.", substring = true)
            .assertIsDisplayed()
    }

    @Test
    fun test_whenFastNavigationBackViaContinueSession_thenReturnedToEntryScreen() {
        // ensure app is started
        composeTestRule.waitUntilTextIsDisplayed("Get started")

        // create a new CoverDrop session in the background
        val (journalistInfo, passphrase) = runBlocking {
            composeTestRule.waitUntil(timeoutMillis = SESSION_UNLOCK_TIMEOUT_MS) {
                lib.getInitializationSuccessful().value
            }

            val passphrase = lib.getPrivateDataRepository().generatePassphrase()
            lib.getPrivateDataRepository().createOrResetStorage(passphrase)

            val journalistInfo = lib.getPublicDataRepository().getAllJournalists().first()
            lib.getPrivateDataRepository().createNewConversation(
                journalistInfo.id,
                DraftMessage(text = "Hello")
            )

            lib.getPrivateDataRepository().lock()

            Pair(journalistInfo, passphrase)
        }

        // login using the continue session flow
        composeTestRule.onNodeWithText("Check your message vault").performClick()
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.CONTINUE_SESSION_ROUTE
        )

        // enter the same passphrase (there are some extra wait for the scroll and focus change
        // animations to finish)
        composeTestRule.performCoverDropEnterPassphraseWords(passphrase.toUiPassphrase())

        // confirm and wait for session to unlock and new message screen to show
        composeTestRule.onNodeWithText("Confirm Passphrase").performScrollToAndClick()
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.INBOX_ROUTE,
            timeoutMs = SESSION_UNLOCK_TIMEOUT_MS
        )

        // open a message thread
        composeTestRule.onNodeWithText(journalistInfo.displayName).performClick()
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.CONVERSATION_ROUTE
        )

        // trigger two navigation back events in quick succession
        composeTestRule.runOnUiThread {
            navController.popBackStack()
            navController.popBackStack()
        }

        // check that we are back at the entry screen
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.ENTRY_ROUTE
        )
    }

    @Test
    fun testCreateSessionAlwaysShowsAFreshPassphrase() {
        // ensure app is started
        composeTestRule.waitUntilTextIsDisplayed("Get started")

        // enter the new session flow
        composeTestRule.onNodeWithText("Get started").performClick()

        // confirm dialog
        composeTestRule.onNodeWithText("Yes, start a new conversation").performClick()
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.HOW_THIS_WORKS_ROUTE
        )

        // go through tutorial screen
        composeTestRule.onNodeWithText("Continue").performClick()
        composeTestRule.onNodeWithText("Continue").performClick()
        composeTestRule.onNodeWithText("Set up my passphrase").performClick()
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.NEW_SESSION_ROUTE
        )

        // switch passphrase to visible
        composeTestRule.onNodeWithTag("primary_button_reveal_passphrase").performClick()
        composeTestRule.waitForIdle()

        // extract passphrase from views
        val passphraseWords1 = getPassphraseFromUi()
        Log.w("LargeScenario", "passphraseWords1=$passphraseWords1")

        // go back to the previous screen
        composeTestRule.pressBack()

        // enter the new session flow again
        composeTestRule.onNodeWithText("Set up my passphrase").performClick()
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.NEW_SESSION_ROUTE
        )

        // switch passphrase to visible
        composeTestRule.onNodeWithTag("primary_button_reveal_passphrase").performClick()
        composeTestRule.waitForIdle()

        // extract passphrase from views
        val passphraseWords2 = getPassphraseFromUi()
        Log.w("LargeScenario", "passphraseWords2=$passphraseWords2")

        // check that the passphrases are different (yes, there is a tiniest chance that we are
        // unlucky which we ignore)
        assertThat(passphraseWords1).isNotEqualTo(passphraseWords2)
    }

    private fun getPassphraseFromUi(): List<UiPassphraseWord> {
        return composeTestRule
            .onAllNodesWithTag("passphrase_box")
            .fetchSemanticsNodes()
            .map { it.config[SemanticsProperties.Text] }
            .map { UiPassphraseWord(it.first().toString()) }
    }
}
