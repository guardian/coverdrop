package com.theguardian.coverdrop.ui.tests.utils

import androidx.activity.ComponentActivity
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEvent
import androidx.compose.ui.input.key.NativeKeyEvent
import androidx.compose.ui.input.key.nativeKeyCode
import androidx.compose.ui.test.*
import androidx.compose.ui.test.junit4.AndroidComposeTestRule
import androidx.navigation.testing.TestNavHostController
import androidx.test.ext.junit.rules.ActivityScenarioRule
import com.theguardian.coverdrop.core.ui.models.UiPassphrase
import kotlinx.coroutines.runBlocking
import org.junit.rules.TestRule
import java.util.Random

private const val WAIT_UNTIL_DEFAULT_TIMEOUT_MS = 10_000L

fun randomString(length: Int): String {
    val random = Random()
    val chars = CharArray(length) { 'a' + random.nextInt(26) }
    return chars.concatToString()
}

fun <A : ComponentActivity> AndroidComposeTestRule<ActivityScenarioRule<A>, A>.pressBack() {
    this.runOnUiThread {
        this.activityRule.scenario.onActivity { activity -> activity.onBackPressedDispatcher.onBackPressed() }
    }
}

fun <R : TestRule, A : ComponentActivity> AndroidComposeTestRule<R, A>.waitUntilTextIsDisplayed(
    text: String,
    allowSubstringMatch: Boolean = false,
    timeoutMs: Long = WAIT_UNTIL_DEFAULT_TIMEOUT_MS,
) {
    this.waitUntil(timeoutMillis = timeoutMs) {
        this.onAllNodesWithText(text, substring = allowSubstringMatch)
            .fetchSemanticsNodes()
            .isNotEmpty()
    }
}

fun <R : TestRule, A : ComponentActivity> AndroidComposeTestRule<R, A>.waitUntilTagIsPresent(
    tag: String,
    timeoutMs: Long = WAIT_UNTIL_DEFAULT_TIMEOUT_MS,
) {
    this.waitUntil(timeoutMillis = timeoutMs) {
        this.onAllNodesWithTag(tag)
            .fetchSemanticsNodes()
            .isNotEmpty()
    }
}

fun <R : TestRule, A : ComponentActivity> AndroidComposeTestRule<R, A>.waitForNavigationTo(
    navController: TestNavHostController,
    targetRoute: String,
    timeoutMs: Long = WAIT_UNTIL_DEFAULT_TIMEOUT_MS,
) {
    this.waitUntil(timeoutMillis = timeoutMs) {
        navController.getCurrentRoute() == targetRoute
    }
    runBlocking { awaitIdle() }
}

fun <R : TestRule, A : ComponentActivity> AndroidComposeTestRule<R, A>.performCoverDropEnterPassphraseWords(
    passphraseWords: UiPassphrase,
) {
    passphraseWords.forEachIndexed { i, word ->
        val node = onNodeWithTag("passphrase_edit_$i")

        // the extra waits are necessary, as the emulators are a bit flaky with the layout
        // timings when both a scroll view and the software keyboard are involved...
        node.performScrollTo()
        waitForIdle()
        Thread.sleep(100)

        node.performTextInput(word.content)
        waitForIdle()
        Thread.sleep(100)
    }
}

fun SemanticsNodeInteraction.performScrollToAndClick() {
    performScrollTo()
    performClick()
}

fun SemanticsNodeInteraction.typeKey(key: Key) {
    performKeyPress(KeyEvent(NativeKeyEvent(NativeKeyEvent.ACTION_DOWN, key.nativeKeyCode)))
    performKeyPress(KeyEvent(NativeKeyEvent(NativeKeyEvent.ACTION_UP, key.nativeKeyCode)))
}

fun TestNavHostController.getCurrentRoute(): String? = this.currentDestination?.route
