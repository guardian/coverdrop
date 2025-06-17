package com.theguardian.coverdrop.ui.tests

import android.annotation.SuppressLint
import androidx.activity.compose.setContent
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.lifecycle.Lifecycle
import androidx.navigation.compose.ComposeNavigator
import androidx.navigation.testing.TestNavHostController
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.ICoverDropLib
import com.theguardian.coverdrop.core.LockState
import com.theguardian.coverdrop.core.api.models.SystemStatus
import com.theguardian.coverdrop.core.models.StatusEvent
import com.theguardian.coverdrop.ui.navigation.CoverDropDestinations
import com.theguardian.coverdrop.ui.navigation.CoverDropNavGraph
import com.theguardian.coverdrop.ui.tests.utils.CoverDropPublicDataRepositoryMock
import com.theguardian.coverdrop.ui.tests.utils.TEST_PASSPHRASE
import com.theguardian.coverdrop.ui.tests.utils.waitForNavigationTo
import com.theguardian.coverdrop.ui.tests.utils.waitUntilTextIsDisplayed
import com.theguardian.coverdrop.ui.theme.CoverDropSurface
import com.theguardian.coverdrop.ui.viewmodels.SelectedRecipientState
import com.theguardian.coverdrop.ui.viewmodels.SelectedRecipientViewModel
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
class EntryScreenTests {

    @get:Rule
    var hiltRule = HiltAndroidRule(this)

    @get:Rule
    val composeTestRule = createAndroidComposeRule<TestActivity>()

    @get:Rule
    val testName = TestName()

    private lateinit var navController: TestNavHostController
    private lateinit var sharedSelectedRecipientViewModel: SelectedRecipientViewModel

    @Inject
    lateinit var coverDropLib: ICoverDropLib

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
            }
        }
    }

    @Test
    fun whenLaunched_thenEntryScreenShown() {
        composeTestRule.onNodeWithText("Send us a message securely and privately")
            .assertIsDisplayed()
    }

    /**
     * AC-START-1
     *
     * GIVEN the user visits the start screen
     * WHEN the user clicks "Start a new Conversation"
     * THEN they are shown the remember passphrase screen
     *
     * AC-RP-4
     *
     * GIVEN the user visits the remember passphrase screen
     * WHEN the user clicks the back button
     * THEN they are taken back to the choose conversation page
     */
    @Test
    fun whenClickGetStarted_thenNavigateToNewSession_andCanReturn() {
        composeTestRule.onNodeWithText("Get started").performClick()

        composeTestRule.onNodeWithText("Set up your secure message vault").assertExists()

        composeTestRule.onNodeWithText("Yes, start a new conversation").performClick()

        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.HOW_THIS_WORKS_ROUTE
        )

        composeTestRule.runOnUiThread { navController.navigateUp() }

        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.ENTRY_ROUTE
        )
    }

    /**
     * AC-START-2
     *
     * GIVEN the user visits the start screen
     * WHEN the user clicks "Continue a Conversation"
     * THEN they are shown the "enter passphrase" screen
     *
     * AC-EP-7
     *
     * GIVEN the user visits the enter passphrase screen
     * WHEN the back button is pressed
     * THEN the user is taken back to the choose conversation screen
     */
    @Test
    fun whenClickCheckYourInbox_thenNavigateToContinueSession_andCanReturn() {
        composeTestRule.onNodeWithText("Check your message vault").performClick()

        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.CONTINUE_SESSION_ROUTE
        )

        composeTestRule.runOnUiThread { navController.navigateUp() }

        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.ENTRY_ROUTE
        )
    }

    /**
     * AC-START-4
     *
     * GIVEN the user visits the start screen
     * WHEN the user clicks the about CoverDrop button
     * THEN the user is shown the CoverDrop about screen
     */
    @Test
    fun whenClickAbout_thenNavigateToAbout_andCanReturn() {
        composeTestRule.onNodeWithText("About Secure Messaging").performClick()

        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.ABOUT_ROUTE
        )

        composeTestRule.runOnUiThread { navController.navigateUp() }

        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.ENTRY_ROUTE
        )
    }

    /**
     * AC-START-6
     *
     * GIVEN the user visits the start screen
     * THEN the secure storage should be locked
     */
    @Test
    fun whenVisitStartScreen_thenSecureStorageIsLocked() {
        assertThat(coverDropLib.getPrivateDataRepository().getLockState())
            .isEqualTo(LockState.LOCKED)
    }

    /**
     * GIVEN the status is unavailable
     * WHEN the user visits the start screen
     * THEN an error message is shown
     */
    @Test
    fun whenVisitStartScreen_andStatusIsUnavailable_thenShowErrorMessage() {
        val mockedPublicDataRepository =
            (coverDropLib.getPublicDataRepository() as CoverDropPublicDataRepositoryMock)
        mockedPublicDataRepository.setStatusEvent(
            StatusEvent(
                SystemStatus.UNAVAILABLE,
                false,
                "some reason"
            )
        )

        composeTestRule.onNodeWithText("currently not available", substring = true).assertExists()
        composeTestRule.onNodeWithText("some reason", substring = true).assertExists()

    }

    /**
     * GIVEN the session is unlocked (against all odds)
     * WHEN the user opens the entry screen
     * THEN the session is automatically locked
     */
    @Test
    fun whenSessionIsUnlocked_thenLockSession() {
        // navigate to some other screen
        composeTestRule.onNodeWithText("About Secure Messaging").performClick()
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.ABOUT_ROUTE
        )

        // unlock the session
        runBlocking {
            coverDropLib.getPrivateDataRepository().unlock(TEST_PASSPHRASE)
        }
        assertThat(coverDropLib.getPrivateDataRepository().getLockState())
            .isEqualTo(LockState.UNLOCKED)

        // navigate back to the entry screen and wait for the spinner to disappear
        composeTestRule.runOnUiThread { navController.navigate(CoverDropDestinations.ENTRY_ROUTE) }
        composeTestRule.waitForNavigationTo(
            navController,
            CoverDropDestinations.ENTRY_ROUTE
        )
        composeTestRule.waitUntilTextIsDisplayed("Get started")

        // check that the session is locked
        assertThat(coverDropLib.getPrivateDataRepository().getLockState())
            .isEqualTo(LockState.LOCKED)
    }

    /**
     * GIVEN the user is on the entry screen
     * WHEN the user clicks the exit icon in the top bar
     * THEN the the activity should be closed (finished)
     */
    @Test
    fun whenClickExit_thenFinishActivity() {
        composeTestRule.onNodeWithTag("top_bar_navigation_action").performClick()

        // Depending on timing, we might end up in one of the two cases
        try {
            // Case A: the activity is still finishing; this is what we see most often
            assertThat(composeTestRule.activity.isFinishing).isTrue()
        } catch (e: NullPointerException) {
            // Case B: the activity managed to finish before we checked above; in that case we
            // can check that it has been destroyed (i.e. the NPE is thrown because the activity
            // is no longer available). See #2917
            val state = composeTestRule.activityRule.scenario.state
            assertThat(state).isEqualTo(Lifecycle.State.DESTROYED)
        }
    }

    /**
     * GIVEN the user is on the entry screen
     * WHEN the user has not yet used any other feature of the app
     * THEN the selected recipient state should be initializing
     */
    @Test
    fun whenVisitStartScreen_thenSelectedRecipientStateIsInitializing() {
        // ensure that lateinit variables are ready
        runBlocking { composeTestRule.awaitIdle() }

        assertThat(sharedSelectedRecipientViewModel.getInternalStateForTesting())
            .isEqualTo(SelectedRecipientState.Initializing)
    }
}
