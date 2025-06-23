package com.theguardian.coverdrop.ui.tests

import android.annotation.SuppressLint
import androidx.activity.compose.setContent
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.filterToOne
import androidx.compose.ui.test.hasClickAction
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.onFirst
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onSiblings
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.navigation.compose.ComposeNavigator
import androidx.navigation.testing.TestNavHostController
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.ICoverDropLib
import com.theguardian.coverdrop.core.models.JournalistVisibility
import com.theguardian.coverdrop.ui.navigation.CoverDropDestinations
import com.theguardian.coverdrop.ui.navigation.CoverDropNavGraph
import com.theguardian.coverdrop.ui.theme.CoverDropSurface
import com.theguardian.coverdrop.ui.utils.COVERDROP_SAMPLE_DATA
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
class RecipientSelectionScreenTest {

    @get:Rule
    var hiltRule = HiltAndroidRule(this)

    @get:Rule
    val composeTestRule = createAndroidComposeRule<TestActivity>()

    @get:Rule
    val testName = TestName()

    private lateinit var navController: TestNavHostController

    @Inject
    lateinit var coverDropLib: ICoverDropLib

    private lateinit var sharedSelectedRecipientViewModel: SelectedRecipientViewModel

    @SuppressLint("ViewModelConstructorInComposable")
    @Before
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
                    startDestination = CoverDropDestinations.RECIPIENT_SELECTION_ROUTE,
                    sharedSelectedRecipientViewModel = sharedSelectedRecipientViewModel,
                )
            }
        }

        // ensure that lateinit variables are ready
        runBlocking { composeTestRule.awaitIdle() }
    }

    /**
     * AC-RS-1
     *
     * GIVEN user visits the recipient selection screen
     * WHEN loaded
     * THEN all teams are shown
     */
    @Test
    fun whenLoaded_thenAllTeamsAreShown() {
        val teams = COVERDROP_SAMPLE_DATA.getTeams()

        composeTestRule.onNodeWithText(teams.first().displayName).assertIsDisplayed()

        composeTestRule.onNodeWithText(teams.last().displayName).performScrollTo()
            .assertIsDisplayed()
    }

    /**
     * AC-RS-2
     *
     * GIVEN user visits the recipient selection screen
     * WHEN journalist tab is clicked
     * THEN all journalists are shown
     *
     * AC-RS-5
     *
     * GIVEN user visits the recipient selection screen
     * WHEN the "Select" button next to a journalist is clicked
     * THEN the journalist is returned as confirmed id
     */
    @Test
    fun whenJournalistsTabClicked_thenJournalistsAreShown() {
        val firstJournalist = COVERDROP_SAMPLE_DATA.getJournalists().first()

        // AC-RS-2
        composeTestRule.onNodeWithText("Journalists").performClick()

        composeTestRule.onNodeWithText(firstJournalist.displayName).assertIsDisplayed()
        composeTestRule.onAllNodesWithText(firstJournalist.description)
            .onFirst()
            .assertIsDisplayed()

        // AC-RS-5
        composeTestRule.onNodeWithText(firstJournalist.displayName)
            .onSiblings()
            .filterToOne(hasClickAction())
            .performClick()

        assertThat(sharedSelectedRecipientViewModel.getSelectedRecipient().value)
            .isEqualTo(SelectedRecipientState.SingleRecipientWithChoice(firstJournalist))
    }

    /**
     * AC-RS-3
     *
     * GIVEN user visits the recipient selection screen
     * WHEN a team item is clicked
     * THEN the team confirmation screen is shown
     *
     * AC-RS-4
     *
     * GIVEN user visits the recipient selection screen
     * WHEN a team item is clicked and then "Select team" is clicked
     * THEN the team is returned as confirmed id
     */
    @Test
    fun whenTeamCardIsClicked_thenConfirmationIsShown_andCanBeConfirmed() {
        val firstTeam = COVERDROP_SAMPLE_DATA.getTeams().first()

        // AC-RS-3
        composeTestRule.onNodeWithText(firstTeam.displayName).performClick()

        // AC-RS-4
        composeTestRule.onNodeWithText(firstTeam.description, substring = true).assertIsDisplayed()
        composeTestRule.onNodeWithText("Select team").performClick()

        assertThat(sharedSelectedRecipientViewModel.getSelectedRecipient().value)
            .isEqualTo(SelectedRecipientState.SingleRecipientWithChoice(firstTeam))
    }

    /**
     * GIVEN user visits the recipient selection screen
     * WHEN the "Journalists" tab is clicked
     * THEN all journalists are shown, except hidden ones
     */
    @Test
    fun whenJournalistIsHidden_thenNotShown() {
        val shownJournalist = COVERDROP_SAMPLE_DATA.getJournalists()
            .first { it.visibility == JournalistVisibility.VISIBLE }
        val hiddenJournalist = COVERDROP_SAMPLE_DATA.getJournalists()
            .first { it.visibility == JournalistVisibility.HIDDEN }

        composeTestRule.onNodeWithText("Journalists").performClick()

        composeTestRule.onNodeWithText(shownJournalist.displayName).assertIsDisplayed()
        composeTestRule.onNodeWithText(hiddenJournalist.displayName).assertDoesNotExist()
    }

    /**
     * GIVEN user has clicked a team and is on the confirmation screen
     * WHEN clicking on the back button
     * THEN back to the overview list
     */
    @Test
    fun whenBackButtonClicked_onConfirmationScreen_thenBackToRecipientSelection() {
        val firstTeam = COVERDROP_SAMPLE_DATA.getTeams().first()
        composeTestRule.onNodeWithText("Journalists").assertIsDisplayed()

        composeTestRule.onNodeWithText(firstTeam.displayName).performClick()
        composeTestRule.onNodeWithText("Select team").assertIsDisplayed()
        composeTestRule.onNodeWithText("Journalists").assertDoesNotExist()

        composeTestRule.activity.onBackPressedDispatcher.onBackPressed()

        composeTestRule.onNodeWithText("Journalists").assertIsDisplayed()
    }
}
