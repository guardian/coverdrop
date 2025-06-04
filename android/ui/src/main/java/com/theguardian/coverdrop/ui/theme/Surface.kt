package com.theguardian.coverdrop.ui.theme

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier

@Composable
fun CoverDropSurface(content: @Composable () -> Unit) {
    CoverDropSampleTheme {
        Surface(
            modifier = Modifier.fillMaxSize(),
            color = MaterialTheme.colors.background,
        ) { content() }
    }
}

@Composable
fun CoverDropPreviewSurface(content: @Composable () -> Unit) {
    CoverDropSampleTheme {
        Surface(
            color = MaterialTheme.colors.background,
        ) { content() }
    }
}
