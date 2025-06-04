package com.theguardian.coverdrop.ui.tests

import android.os.Bundle
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

        setContent {
        }
    }
}
