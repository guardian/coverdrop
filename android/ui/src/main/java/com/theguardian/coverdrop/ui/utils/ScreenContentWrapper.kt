package com.theguardian.coverdrop.ui.utils

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.calculateStartPadding
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.LayoutDirection

data class ScreenInsets(
    val top: Dp,
    val bottom: Dp,
    val horizontal: Dp
)

@Composable
fun rememberScreenInsets(): ScreenInsets {
    val safeInsets = WindowInsets.safeDrawing
    val paddingValues = safeInsets.asPaddingValues()
    return ScreenInsets(
        top = paddingValues.calculateTopPadding(),
        bottom = paddingValues.calculateBottomPadding(),
        horizontal = paddingValues.calculateStartPadding(LayoutDirection.Ltr)
    )
}

@Composable
fun ScreenContentWrapper(
    modifier: Modifier = Modifier,
    content: @Composable () -> Unit
) {
    val insets = rememberScreenInsets()

    Box(
        modifier = modifier
            .fillMaxSize()
            .padding(
                start = insets.horizontal,
                end = insets.horizontal
            )
    ) {
        content()
    }
}
