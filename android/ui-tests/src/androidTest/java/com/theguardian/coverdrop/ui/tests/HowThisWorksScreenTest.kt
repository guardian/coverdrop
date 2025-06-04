package com.theguardian.coverdrop.ui.tests

import androidx.activity.compose.setContent
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.navigation.compose.ComposeNavigator
import androidx.navigation.testing.TestNavHostController
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.theguardian.coverdrop.core.ICoverDropLib
import com.theguardian.coverdrop.ui.navigation.CoverDropDestinations
import com.theguardian.coverdrop.ui.navigation.CoverDropNavGraph
import com.theguardian.coverdrop.ui.tests.utils.waitForNavigationTo
import com.theguardian.coverdrop.ui.theme.CoverDropSurface
import dagger.hilt.android.testing.HiltAndroidRule
import dagger.hilt.android.testing.HiltAndroidTest
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TestName
import org.junit.runner.RunWith
import javax.inject.Inject

@HiltAndroidTest
@RunWith(AndroidJUnit4::class)
class HowThisWorksScreenTest {

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
                    startDestination = CoverDropDestinations.HOW_THIS_WORKS_ROUTE
                )
            }
        }
    }

    @Test
    fun whenLaunched_thenHowThisWorksScreenShown() {
        composeTestRule.onNodeWithText("How this works").assertIsDisplayed()
    }

    /**
     * GIVEN the user is on the How This Works screen
     * WHEN clicking continue
     * THEN the pager advances to the next information
     */
    @Test
    fun whenClickContinue_thenNavigateToNewSession_andCanReturn() {
        composeTestRule.onNodeWithText("Send a message").performClick()

        composeTestRule.onNodeWithText("Continue").performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithText("Check back for a response").assertIsDisplayed()

        composeTestRule.onNodeWithText("Continue").performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithText("Memorise your passphrase").assertIsDisplayed()
    }

    /**
     * GIVEN the user has navigated to the last page of the How This Works screen
     * WHEN clicking the button
     * THEN navigate to the New Session screen
     */
    @Test
    fun whenClickSetMyPassphrase_thenNavigateToNewSession() {
        composeTestRule.onNodeWithText("Continue").performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithText("Continue").performClick()
        composeTestRule.waitForIdle()

        composeTestRule.onNodeWithText("Set up my passphrase").performClick()
        composeTestRule.waitForNavigationTo(navController, CoverDropDestinations.NEW_SESSION_ROUTE)
    }
}
