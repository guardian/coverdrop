package com.theguardian.coverdrop.ui.tests

import android.os.Bundle
import android.view.WindowManager
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import dagger.hilt.android.AndroidEntryPoint

/**
 * An activity for the UI tests that performs no action other than initializing an empty
 * compose state.
 */
@AndroidEntryPoint
class TestActivity : ComponentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Prevent the software keyboard from showing. It appears asynchronously after a text field
        // gains focus and then resizes the window (`adjustResize`). On slow emulators this can
        // happen between scrolling to a button and clicking it, which moves the button out of the
        // viewport and the click is lost. The tests enter text via semantics actions and do not
        // need the keyboard.
        window.addFlags(WindowManager.LayoutParams.FLAG_ALT_FOCUSABLE_IM)

        setContent {
        }
    }
}
