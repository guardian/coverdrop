package com.theguardian.coverdrop.ui.components

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.CircularProgressIndicator
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment.Companion.CenterHorizontally
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import com.theguardian.coverdrop.ui.theme.CoverDropPreviewSurface
import com.theguardian.coverdrop.ui.theme.Padding

@Composable
fun ProgressSpinnerWithText(text: String) {
    Column(
        horizontalAlignment = CenterHorizontally,
        modifier = Modifier
            .fillMaxWidth(1f)
    ) {
        CircularProgressIndicator(
            modifier = Modifier
                .padding(bottom = Padding.XL)
                .size(48.dp)
        )
        Text(text)
    }
}

@Preview
@Composable
fun ProgressSpinnerWithTextPreview() = CoverDropPreviewSurface {
    ProgressSpinnerWithText("writing up...")
}
