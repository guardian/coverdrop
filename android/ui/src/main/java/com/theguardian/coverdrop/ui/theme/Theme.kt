package com.theguardian.coverdrop.ui.theme

import androidx.compose.material.MaterialTheme
import androidx.compose.runtime.Composable


@Composable
fun CoverDropSampleTheme(content: @Composable () -> Unit) {

    MaterialTheme(
        colors = CoverDropColorPalette,
        typography = CoverdropTypography,
        content = content,
    )
}
