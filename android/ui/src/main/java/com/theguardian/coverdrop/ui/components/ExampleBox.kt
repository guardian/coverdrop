package com.theguardian.coverdrop.ui.components

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentSize
import androidx.compose.material.Card
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import com.theguardian.coverdrop.ui.theme.CoverDropPreviewSurface
import com.theguardian.coverdrop.ui.theme.Padding
import com.theguardian.coverdrop.ui.theme.RoundedCornerShape
import com.theguardian.coverdrop.ui.theme.SurfaceBorder
import com.theguardian.coverdrop.ui.utils.parseHighlightedTextIntoAnnotated

@Composable
fun ExampleBox(text: AnnotatedString, modifier: Modifier = Modifier) {
    Card(
        modifier = modifier
            .fillMaxWidth()
            .wrapContentSize(),
        border = BorderStroke(width = 1.dp, color = SurfaceBorder),
        shape = RoundedCornerShape.S,
        backgroundColor = MaterialTheme.colors.surface,
    ) {
        Text(
            modifier = Modifier
                .padding(vertical = Padding.M + 2.dp, horizontal = Padding.M)
                .fillMaxWidth(),
            text = text,
            style = MaterialTheme.typography.body1,
        )
    }
}

@Preview(device = Devices.PIXEL_6)
@Composable
fun ExampleBoxPreview() = CoverDropPreviewSurface {
    ExampleBox(
        text = parseHighlightedTextIntoAnnotated(
            text = "I work there as a ~warehouse manager~ so I have access to certain documents.",
            highlightColor = MaterialTheme.colors.primary
        ),
    )
}
