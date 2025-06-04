package com.theguardian.coverdrop.ui.tests

import androidx.activity.compose.setContent
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.test.assertAll
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.assertIsFocused
import androidx.compose.ui.test.hasTextExactly
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onFirst
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
import com.theguardian.coverdrop.ui.components.PASSPHRASE_MASK
import com.theguardian.coverdrop.ui.navigation.CoverDropDestinations
import com.theguardian.coverdrop.ui.navigation.CoverDropNavGraph
import com.theguardian.coverdrop.ui.tests.utils.TEST_PASSPHRASE
import com.theguardian.coverdrop.ui.tests.utils.performCoverDropEnterPassphraseWords
import com.theguardian.coverdrop.ui.tests.utils.performScrollToAndClick
import com.theguardian.coverdrop.ui.tests.utils.typeKey
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

@HiltAndroidTest
@RunWith(AndroidJUnit4::class)
class NewSessionScreenTest {

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
                    startDestination = CoverDropDestinations.NEW_SESSION_ROUTE,
                )
            }
        }
    }

    /**
     * AC-RP-1
     *
     * GIVEN the user visits the remember passphrase screen
     * WHEN it is the first interaction
     * THEN the passphrase is hidden
     */
    @Test
    fun whenFirstInteraction_thenPassphraseIsHidden() {
        composeTestRule
            .onAllNodesWithTag("passphrase_box")
            .assertCountEquals(4)
            .assertAll(hasTextExactly(PASSPHRASE_MASK))
    }

    /**
     * AC-RP-2
     *
     * GIVEN the user visits the remember passphrase screen
     * WHEN the user clicks show passphrase button
     * THEN they are shown the passphrase text
     */
    @Test
    fun whenClickedReveal_thenPassphraseIsShown() {
        composeTestRule.onNodeWithTag("primary_button_reveal_passphrase").performClick()
        composeTestRule.onAllNodesWithTag("passphrase_box")
            .assertCountEquals(4)
            .assertAll(hasTextExactly(PASSPHRASE_MASK).not())
    }

    /**
     * AC-RP-3
     *
     * GIVEN the user visits the remember passphrase screen
     * WHEN the user clicks I already have a passphrase button
     * THEN they are taken to the enter passphrase page
     */
    @Test
    fun whenClickedAlreadyHavePassphrase_thenNavigatesToEnterPassphraseScreen() {
        composeTestRule.onNodeWithText("I already have a passphrase").performClick()
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.CONTINUE_SESSION_ROUTE
        )
    }

    /**
     * AC-RP-5
     *
     * GIVEN the user visits the remember passphrase screen and the passphrase is shown
     * WHEN the user clicks the hide password button
     * THEN the passphrase is hidden
     */
    @Test
    fun whenClickedHide_thenPassphraseIsHidden() {
        composeTestRule.onNodeWithTag("primary_button_reveal_passphrase").performClick()
        composeTestRule.onNodeWithText("Hide passphrase").performClick()
        composeTestRule.onAllNodesWithTag("passphrase_box")
            .assertAll(hasTextExactly(PASSPHRASE_MASK))
    }

    /**
     * AC-RP-6
     *
     * GIVEN the user visits the remember passphrase screen
     * THEN the secure storage should be locked
     */
    @Test
    fun whenVisitsNewSessionScreen_thenSecureStorageIsLocked() {
        assertThat(coverDropLib.getPrivateDataRepository().getLockState())
            .isEqualTo(LockState.LOCKED)
    }

    /**
     * AC-RP-7
     *
     * GIVEN the user visits the remember passphrase screen
     * WHEN the user confirms they remember the passphrase
     * THEN app shows them boxes to confirm the passphrase
     */
    @Test
    fun whenConfirmRememberingPassphrase_thenShowsBoxesForConfirmingPassphrase() {
        composeTestRule.onNodeWithTag("primary_button_reveal_passphrase").performClick()
        composeTestRule.onNodeWithText("I have remembered my passphrase").performClick()

        composeTestRule.waitUntilTextIsDisplayed("Enter passphrase")
        composeTestRule.onNodeWithTag("passphrase_edit_0").assertIsDisplayed()
        composeTestRule.onNodeWithTag("passphrase_edit_1").assertIsDisplayed()
        composeTestRule.onNodeWithTag("passphrase_edit_2").assertIsDisplayed()
        composeTestRule.onNodeWithTag("passphrase_edit_3").assertIsDisplayed()
    }

    /**
     * AC-RP-8
     *
     * GIVEN the user has confirmed remembering the passphrase
     * WHEN the user enters and confirms the correct passphrase
     * THEN a new secure storage is created and unlocked and navigated to screen to enter a new message
     */
    @Test
    fun whenConfirmCorrectPassphrase_thenSecureStorageUnlockedAndCreated_andNavigateToNewMessage() {
        composeTestRule.onNodeWithTag("primary_button_reveal_passphrase").performClick()
        composeTestRule.onNodeWithText("I have remembered my passphrase").performClick()

        composeTestRule.waitUntilTextIsDisplayed("Enter passphrase")
        composeTestRule.performCoverDropEnterPassphraseWords(TEST_PASSPHRASE.toUiPassphrase())

        composeTestRule.onNodeWithText("Confirm passphrase").performScrollToAndClick()

        // await transition to new message screen
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.NEW_MESSAGE_ROUTE
        )
        assertThat(coverDropLib.getPrivateDataRepository().getLockState())
            .isEqualTo(LockState.UNLOCKED)
    }

    /**
     * AC-RP-9
     *
     * GIVEN the user has confirmed remembering the passphrase
     * WHEN the user enters and confirms an incorrect passphrase
     * THEN an error is shown
     */
    @Test
    fun whenConfirmIncorrectPassphrase_thenErrorMessage() {
        composeTestRule.onNodeWithTag("primary_button_reveal_passphrase").performClick()
        composeTestRule.onNodeWithText("I have remembered my passphrase").performClick()

        composeTestRule.waitUntilTextIsDisplayed("Enter passphrase")
        composeTestRule.performCoverDropEnterPassphraseWords(List(4) { UiPassphraseWord("puffin") })
        composeTestRule.onNodeWithText("Confirm passphrase").performScrollToAndClick()

        // wait for error message to appear
        composeTestRule.waitUntilTagIsPresent("warning_box_text")
        composeTestRule.onNodeWithTag("warning_box_text").performScrollTo()

        composeTestRule.onNodeWithText(
            text = "The passphrase you entered does not match the generated one from the previous screen",
            substring = true
        ).assertIsDisplayed()
    }

    /**
     * GIVEN the user has confirmed remembering the passphrase
     * WHEN the user enters and confirms a passphrase that contains implausible words
     * THEN an error is shown
     */
    @Test
    fun whenConfirmPassphraseWithImplausibleWords_thenErrorMessage() {
        composeTestRule.onNodeWithTag("primary_button_reveal_passphrase").performClick()
        composeTestRule.onNodeWithText("I have remembered my passphrase").performClick()

        composeTestRule.waitUntilTextIsDisplayed("Enter passphrase")
        composeTestRule.performCoverDropEnterPassphraseWords(List(4) { UiPassphraseWord("1") })
        composeTestRule.onNodeWithText("Confirm passphrase").performScrollToAndClick()

        // wait for error message to appear
        composeTestRule.waitUntilTagIsPresent("warning_box_text")
        composeTestRule.onNodeWithTag("warning_box_text").performScrollTo()

        composeTestRule.onNodeWithText(
            text = "The passphrase you entered does not match the generated one from the previous screen",
            substring = true
        ).assertIsDisplayed()
    }

    /**
     * GIVEN the screen is open
     * WHEN the user clicks the help banner
     * THEN the user is navigated to the respective help screen
     */
    @Test
    fun whenBannerClicked_thenNavigatesToHelpCraftMessageScreen() {
        composeTestRule.onNodeWithText("Keeping passphrases safe").performClick()
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.HELP_KEEPING_PASSPHRASES_SAFE_ROUTE
        )
    }

    @Test
    fun whenPassphraseWordClicked_thenAllRevealed() {
        composeTestRule.onAllNodesWithTag("passphrase_box")
            .assertAll(hasTextExactly(PASSPHRASE_MASK))
        composeTestRule.onAllNodesWithTag("passphrase_box").onFirst().performClick()
        composeTestRule.onAllNodesWithTag("passphrase_box")
            .assertAll(hasTextExactly(PASSPHRASE_MASK).not())
    }


    /**
     * GIVEN the user is on the continue session screen
     * WHEN the user uses the ENTER key on the keyboard
     * THEN the focus moves through the passphrase fields and eventually selects the confirm button
     */
    @Test
    fun whenUsingEnterKey_thenFocusMovesThroughFieldsAndConfirms() {
        composeTestRule.onNodeWithTag("primary_button_reveal_passphrase").performClick()
        composeTestRule.onNodeWithText("I have remembered my passphrase").performClick()
        composeTestRule.waitUntilTextIsDisplayed("Enter passphrase")

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
        val confirmButton = composeTestRule.onNodeWithText("Confirm passphrase")
        confirmButton.assertIsFocused()
    }
}
