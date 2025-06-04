package com.theguardian.coverdrop.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import com.theguardian.coverdrop.ui.theme.CoverDropPreviewSurface
import com.theguardian.coverdrop.ui.theme.Padding
import com.theguardian.coverdrop.ui.utils.bold

const val QUOTE_OPEN = "'"
const val QUOTE_CLOSE = "'"

@Composable
fun BlockQuote(
    text: String,
    authorName: String,
    authorTagLine: String,
    modifier: Modifier = Modifier,
    addQuotes: Boolean = true,
) {
    val fullText = if (addQuotes) "$QUOTE_OPEN$text$QUOTE_CLOSE" else text
    Row(modifier = modifier.height(IntrinsicSize.Max)) {
        Box(
            modifier = Modifier
                .width(4.dp)
                .fillMaxHeight(1f)
                .background(MaterialTheme.colors.primary, RectangleShape),
        )
        Column {
            Text(
                text = fullText,
                style = MaterialTheme.typography.body1,
                modifier = Modifier.padding(
                    start = Padding.L,
                    end = Padding.None,
                    bottom = Padding.L,
                    top = Padding.None
                )
            )
            Text(
                text = authorName,
                style = MaterialTheme.typography.body1.bold(),
                modifier = Modifier.padding(horizontal = Padding.L, vertical = Padding.None)
            )
            Text(
                text = authorTagLine,
                style = MaterialTheme.typography.body1,
                modifier = Modifier.padding(
                    start = Padding.L,
                    end = Padding.None,
                    bottom = Padding.None,
                    top = Padding.S
                )
            )
        }
    }

}

@Preview(device = Devices.PIXEL_6, showSystemUi = false)
@Composable
fun BlockQuotePreview() = CoverDropPreviewSurface {
    BlockQuote(
        text = "We receive 100s of tips a day, messages that stand out have facts",
        authorName = "Joe Bloggs",
        authorTagLine = "Head of investigations",
    )
}

@Preview(device = Devices.PIXEL_6, showSystemUi = false, locale = "ar")
@Composable
fun BlockQuotePreviewRTL() = CoverDropPreviewSurface {
    BlockQuote(
        text = "أهلاً",
        authorName = "Joe Bloggs",
        authorTagLine = "Head of investigations",
    )
}
