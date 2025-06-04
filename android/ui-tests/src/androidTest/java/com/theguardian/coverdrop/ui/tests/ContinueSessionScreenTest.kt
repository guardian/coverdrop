package com.theguardian.coverdrop.ui.tests

import androidx.activity.compose.setContent
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.assert
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.assertIsEnabled
import androidx.compose.ui.test.assertIsFocused
import androidx.compose.ui.test.assertIsNotEnabled
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
import com.theguardian.coverdrop.core.ui.models.UiPassphraseWord
import com.theguardian.coverdrop.core.ui.models.toUiPassphrase
import com.theguardian.coverdrop.ui.components.PassphraseWordHiddenKey
import com.theguardian.coverdrop.ui.components.PassphraseWordInvalidKey
import com.theguardian.coverdrop.ui.navigation.CoverDropDestinations
import com.theguardian.coverdrop.ui.navigation.CoverDropNavGraph
import com.theguardian.coverdrop.ui.tests.utils.TEST_INVALID_WORD
import com.theguardian.coverdrop.ui.tests.utils.TEST_PASSPHRASE
import com.theguardian.coverdrop.ui.tests.utils.performCoverDropEnterPassphraseWords
import com.theguardian.coverdrop.ui.tests.utils.performScrollToAndClick
import com.theguardian.coverdrop.ui.tests.utils.pressBack
import com.theguardian.coverdrop.ui.tests.utils.typeKey
import com.theguardian.coverdrop.ui.tests.utils.waitForNavigationTo
import com.theguardian.coverdrop.ui.tests.utils.waitUntilTagIsPresent
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

@HiltAndroidTest
@RunWith(AndroidJUnit4::class)
class ContinueSessionScreenTest {

    @get:Rule
    var hiltRule = HiltAndroidRule(this)

    @get:Rule
    val composeTestRule = createAndroidComposeRule<TestActivity>()

    @get:Rule
    val testName = TestName()

    private lateinit var navController: TestNavHostController

    @Inject
    lateinit var coverDropLib: ICoverDropLib

    @Before
    fun setupAppNavHost() {
        hiltRule.inject()

        composeTestRule.activity.setContent {
            CoverDropSurface {
                navController = TestNavHostController(LocalContext.current)
                navController.navigatorProvider.addNavigator(ComposeNavigator())
                CoverDropNavGraph(
                    navController = navController,
                    startDestination = CoverDropDestinations.CONTINUE_SESSION_ROUTE,
                )
            }
        }
    }

    /**
     * AC-EP-1
     *
     * GIVEN the user visits the enter passphrase screen
     * WHEN they type a word into the first text field
     * THEN the passphrase is shown
     *
     * AC-EP-2
     *
     * GIVEN the user visits the enter passphrase screen
     * WHEN the user clicks hide passphrase icon
     * THEN the passphrase text is hidden
     */
    @Test
    fun whenFirstInteraction_thenPassphraseIsShown() {
        // AC-EP-1
        composeTestRule
            .onNodeWithTag("passphrase_edit_0")
            .performTextInput("penguin")

        composeTestRule
            .onNodeWithTag("passphrase_edit_0")
            .assert(SemanticsMatcher.expectValue(PassphraseWordHiddenKey, false))

        // AC-EP-2
        composeTestRule
            .onNodeWithTag("passphrase_edit_0_hide")
            .performClick()

        composeTestRule
            .onNodeWithTag("passphrase_edit_0")
            .assert(SemanticsMatcher.expectValue(PassphraseWordHiddenKey, true))

        // checking the reverse of AC-EP-2
        composeTestRule
            .onNodeWithTag("passphrase_edit_0_reveal")
            .performClick()

        composeTestRule
            .onNodeWithTag("passphrase_edit_0")
            .assert(SemanticsMatcher.expectValue(PassphraseWordHiddenKey, false))
    }

    /**
     * AC-EP-3
     *
     * GIVEN the user visits the enter passphrase screen
     * WHEN the user has not entered all valid words into the text fields
     * THEN the confirm passphrase button is disabled
     *
     * AC-EP-4
     *
     * GIVEN the user visits the enter passphrase screen
     * WHEN the user has entered all valid words into the text fields
     * THEN the confirm passphrase button is enabled
     */
    @Test
    fun whenNotAllWordsEntered_thenButtonDisabled_butWhenAllEntered_thenButtonEnabled() {
        // AC-EP-3
        composeTestRule.onNodeWithTag("passphrase_edit_0").performTextInput("penguin")
        composeTestRule.onNodeWithTag("passphrase_edit_1").performTextInput("wombat")
        composeTestRule.onNodeWithTag("passphrase_edit_2").performTextInput("puffin")

        composeTestRule.onNodeWithText("Confirm Passphrase").assertIsNotEnabled()

        // AC-EP-4
        composeTestRule.onNodeWithTag("passphrase_edit_3").performTextInput("dinosaur")

        composeTestRule.onNodeWithText("Confirm Passphrase").assertIsEnabled()
    }

    /**
     * AC-EP-5
     *
     * GIVEN the user visits the enter passphrase screen and the user has entered all valid words into the text fields
     * WHEN the confirm passphrase button is pressed
     * THEN the secure storage should be unlocked and the user should be taken to the new message page
     */
    @Test
    fun givenValidPassphrase_whenConfirmIsClicked_thenUnlockedAndNextScreen() {
        composeTestRule.performCoverDropEnterPassphraseWords(TEST_PASSPHRASE.toUiPassphrase())
        composeTestRule.onNodeWithText("Confirm Passphrase").performScrollToAndClick()

        // await transition to inbox
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.INBOX_ROUTE
        )

        assertThat(coverDropLib.getPrivateDataRepository().getLockState())
            .isEqualTo(LockState.UNLOCKED)
    }

    /**
     * AC-EP-6
     *
     * GIVEN the user visits the enter passphrase screen and the user has entered all valid words into the text fields but the passphrase is invalid
     * WHEN the confirm passphrase button is pressed
     * THEN the secure storage should remain locked and an error message should be displayed; the error message should mention that this could mean that no storage was previously created
     */
    @Test
    fun givenInvalidPassphrase_whenConfirmIsClicked_thenErrorDisplayedAndRemainsLocked() {
        // reversed -> valid words, but wrong passphrase
        composeTestRule.performCoverDropEnterPassphraseWords(
            TEST_PASSPHRASE.toUiPassphrase().reversed()
        )
        composeTestRule.onNodeWithText("Confirm Passphrase").performScrollToAndClick()

        // wait for error message to appear
        composeTestRule.waitUntilTagIsPresent("warning_box_text")
        composeTestRule.onNodeWithTag("warning_box_text").performScrollTo()

        composeTestRule
            .onNodeWithText("Failed to open message vault.", substring = true)
            .assertIsDisplayed()
        composeTestRule
            .onNodeWithText(
                "Either you haven't set up a vault, or the passphrase was wrong.",
                substring = true
            )
            .assertIsDisplayed()
        assertThat(coverDropLib.getPrivateDataRepository().getLockState())
            .isEqualTo(LockState.LOCKED)
    }

    /**
     * GIVEN the user visits the enter passphrase screen and the user has entered all valid words into the text fields but one word is not plausible (e.g. contains a number)
     * WHEN the confirm passphrase button is pressed
     * THEN the secure storage should remain locked and an error message should indicate that this passphrase would never be valid
     */
    @Test
    fun givenImplausiblePassphraseWord_whenConfirmIsClicked_thenErrorDisplayedAndRemainsLocked() {
        val passphrase = TEST_PASSPHRASE.toUiPassphrase().toMutableList()

        // replace first word with a non-word entry
        passphrase[0] = UiPassphraseWord(TEST_INVALID_WORD)
        composeTestRule.performCoverDropEnterPassphraseWords(passphrase)
        composeTestRule.onNodeWithText("Confirm Passphrase").performScrollToAndClick()

        // wait for error message to appear
        composeTestRule.waitUntilTagIsPresent("warning_box_text")
        composeTestRule.onNodeWithTag("warning_box_text").performScrollTo()

        composeTestRule
            .onNodeWithText(
                "The passphrase cannot be right because it contains words that are not in the word list.",
            )
            .assertIsDisplayed()
        assertThat(coverDropLib.getPrivateDataRepository().getLockState())
            .isEqualTo(LockState.LOCKED)
    }

    /**
     * GIVEN the user is on the continue session screen
     * WHEN user clicks the "I do not have a passphrase yet" button
     * THEN the app changes to the new session flow
     */
    @Test
    fun whenClickingDoesNotHavePassphraseYet_thenChangesToTheNewSessionFlow() {
        // SETUP: simulate coming from the entry screen (as we pop the stack during navigation)
        runBlocking { composeTestRule.awaitIdle() }
        composeTestRule.runOnUiThread {
            navController.setCurrentDestination(CoverDropDestinations.ENTRY_ROUTE)
            navController.navigate(CoverDropDestinations.CONTINUE_SESSION_ROUTE)
        }
        composeTestRule.waitForNavigationTo(
            navController = navController,
            targetRoute = CoverDropDestinations.CONTINUE_SESSION_ROUTE
        )

        // starting from the continue session flow, click the button for already having a passphrase
        composeTestRule.onNodeWithText("I do not have a passphrase yet").performClick()

        // a confirmation dialog should appear and we click yes
        composeTestRule.onNodeWithText("Yes, start a new conversation").performClick()

        // observe navigation to the new session flow
        composeTestRule.waitForNavigationTo(
            navController = navController,
            targetRoute = CoverDropDestinations.HOW_THIS_WORKS_ROUTE
        )

        // when navigating up/back, the continue session flow is no longer on the navigation stack
        composeTestRule.pressBack()
        composeTestRule.waitForNavigationTo(
            navController = navController,
            targetRoute = CoverDropDestinations.ENTRY_ROUTE
        )
    }

    /**
     * GIVEN the user is on the continue session screen
     * WHEN entering words character by character and some are bad prefixes
     * THEN the bad prefixes are flagged
     */
    @Test
    fun whenEnteringWordsCharacterByCharacter_thenBadPrefixesAreFlagged() {
        val node = composeTestRule.onNodeWithTag("passphrase_edit_0")
        node.assertIsDisplayed()

        // empty string is valid
        node.performTextInput("")
        node.assert(SemanticsMatcher.expectValue(PassphraseWordInvalidKey, false))

        // entering the first letter is valid
        node.performTextInput("a")
        node.assert(SemanticsMatcher.expectValue(PassphraseWordInvalidKey, false))

        // entering a bad prefix is invalid
        node.performTextInput("z") // there is no word starting with az
        node.assert(SemanticsMatcher.expectValue(PassphraseWordInvalidKey, true))

        // remove last bad character and everything should be good again
        node.typeKey(Key.Backspace)
        node.assert(SemanticsMatcher.expectValue(PassphraseWordInvalidKey, false))

        // entering the remainder of the valid word
        node.performTextInput("lbum")
        node.assert(SemanticsMatcher.expectValue(PassphraseWordInvalidKey, false))

        // entering more than te valid wordl should be invalid
        node.performTextInput("a")
        node.assert(SemanticsMatcher.expectValue(PassphraseWordInvalidKey, true))
    }

    /**
     * GIVEN the user is on the continue session screen with an initially unlocked session
     * WHEN the user goes ahead to unlock their session
     * THEN the session is first locked and then unlocked
     */
    @Test
    fun givenInitiallyUnlockedSession_whenUnlocking_thenSessionIsLockedAndUnlocked() {
        val flow = coverDropLib.getLockFlow()
        assertThat(flow.replayCache).isEqualTo(emptyList<LockState>())

        // unlock the session
        runBlocking {
            coverDropLib.getPrivateDataRepository().unlock(TEST_PASSPHRASE)
        }
        assertThat(flow.replayCache).isEqualTo(listOf(LockState.UNLOCKED))

        // enter the passphrase and confirm
        composeTestRule.performCoverDropEnterPassphraseWords(TEST_PASSPHRASE.toUiPassphrase())
        composeTestRule.onNodeWithText("Confirm Passphrase").performScrollToAndClick()

        // await transition to inbox
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.INBOX_ROUTE
        )
        assertThat(flow.replayCache).isEqualTo(
            listOf(
                LockState.UNLOCKED,
                LockState.LOCKED,
                LockState.UNLOCKED
            )
        )
    }

    /**
     * GIVEN the user is on the continue session screen with an initially locked session
     * WHEN the user goes ahead to unlock their session
     * THEN the session is first locked and then unlocked
     */
    @Test
    fun givenInitiallyLockedSession_whenUnlocking_thenSessionIsLockedAndUnlocked() {
        val flow = coverDropLib.getLockFlow()
        assertThat(flow.replayCache).isEqualTo(emptyList<LockState>())

        // enter the passphrase and confirm
        composeTestRule.performCoverDropEnterPassphraseWords(TEST_PASSPHRASE.toUiPassphrase())
        composeTestRule.onNodeWithText("Confirm Passphrase").performScrollToAndClick()

        // await transition to inbox
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.INBOX_ROUTE
        )
        assertThat(flow.replayCache).isEqualTo(
            listOf(
                LockState.LOCKED,
                LockState.UNLOCKED
            )
        )
    }

    /**
     * GIVEN the user is on the continue session screen
     * WHEN the user uses the ENTER key on the keyboard
     * THEN the focus moves through the passphrase fields and eventually selects the confirm button
     */
    @Test
    fun whenUsingEnterKey_thenFocusMovesThroughFieldsAndConfirms() {
        // focus on the first field
        val field1 = composeTestRule.onNodeWithTag("passphrase_edit_0")
        field1.performScrollToAndClick()
        field1.assertIsFocused()
        field1.performTextInput("penguin")
        field1.typeKey(Key.Enter)

        // next field should be focussed
        val field2 = composeTestRule.onNodeWithTag("passphrase_edit_1")
        field2.assertIsFocused()
        field2.performTextInput("wombat")
        field2.typeKey(Key.Enter)

        // next field should be focussed
        val field3 = composeTestRule.onNodeWithTag("passphrase_edit_2")
        field3.assertIsFocused()
        field3.performTextInput("puffin")
        field3.typeKey(Key.Enter)

        // next field should be focussed
        val field4 = composeTestRule.onNodeWithTag("passphrase_edit_3")
        field4.assertIsFocused()
        field4.performTextInput("dinosaur")
        field4.typeKey(Key.Enter)

        runBlocking { composeTestRule.awaitIdle() }

        // and finally the confirm button should be focussed
        val confirmButton = composeTestRule.onNodeWithText("Confirm Passphrase")
        confirmButton.assertIsFocused()
    }
}
