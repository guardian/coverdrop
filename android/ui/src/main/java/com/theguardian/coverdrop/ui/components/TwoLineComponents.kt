package com.theguardian.coverdrop.ui.components

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material.Card
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import com.theguardian.coverdrop.ui.theme.BackgroundInfoPastelBlue
import com.theguardian.coverdrop.ui.theme.CoverDropPreviewSurface
import com.theguardian.coverdrop.ui.theme.NeutralMiddle
import com.theguardian.coverdrop.ui.theme.Padding
import com.theguardian.coverdrop.ui.theme.RoundedCornerShape
import com.theguardian.coverdrop.ui.theme.WarningPastelRed

@Composable
fun TwoLineBanner(
    firstLine: String,
    secondLine: String,
    icon: CoverDropIcons? = CoverDropIcons.Info,
    backgroundColor: Color = BackgroundInfoPastelBlue,
    onClick: () -> Unit = {},
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .background(color = backgroundColor)
            .clickable { onClick() }
    ) {
        TwoLineContent(
            firstLine = firstLine,
            secondLine = secondLine,
            color = MaterialTheme.colors.onPrimary,
            icon = icon,
            modifier = Modifier.padding(
                start = Padding.L, // the L padding aligns it with the main text of the screens
                end = Padding.M,
                top = Padding.M,
                bottom = Padding.M,
            )
        )
    }
}

@Composable
fun TwoLineButton(
    firstLine: String,
    secondLine: String,
    icon: CoverDropIcons? = CoverDropIcons.Info,
    onClick: () -> Unit,
) {
    Card(
        border = BorderStroke(width = 1.dp, color = NeutralMiddle),
        shape = RoundedCornerShape.S,
        backgroundColor = MaterialTheme.colors.background,
        contentColor = MaterialTheme.colors.onBackground,
        modifier = Modifier.clickable { onClick() }
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = Padding.M, vertical = Padding.M)
        ) {
            TwoLineContent(
                firstLine = firstLine,
                secondLine = secondLine,
                color = MaterialTheme.colors.onBackground,
                icon = icon,
            )
        }
    }
}

@Composable
fun TwoLineContent(
    firstLine: String,
    secondLine: String,
    color: Color,
    modifier: Modifier = Modifier,
    icon: CoverDropIcons? = null,
) {
    Row(modifier = modifier) {
        icon?.AsComposable(size = 18.dp, tint = color, modifier = Modifier.padding(top = 2.dp))
        Row(horizontalArrangement = Arrangement.SpaceBetween, modifier = Modifier.fillMaxWidth()) {
            Column(modifier = Modifier.padding(start = Padding.S)) {
                Text(
                    text = firstLine,
                    style = MaterialTheme.typography.body1.copy(
                        fontWeight = FontWeight.W700,
                        color = color
                    )
                )
                Text(
                    text = secondLine,
                    modifier = Modifier.padding(top = 2.dp),
                    style = MaterialTheme.typography.body1.copy(color = color)
                )
            }
            CoverDropIcons.ChevronRight.AsComposable(
                modifier = Modifier.align(Alignment.CenterVertically),
                size = 24.dp,
                tint = color
            )
        }
    }
}

@Preview
@Composable
private fun TwoLineBannerPreview() = CoverDropPreviewSurface {
    TwoLineBanner(
        firstLine = "Craft your first message",
        secondLine = "Learn more",
    )
}

@Preview
@Composable
private fun TwoLineBannerWarning() = CoverDropPreviewSurface {
    TwoLineBanner(
        firstLine = "Attention, attention!",
        secondLine = "This is a warning message",
        backgroundColor = WarningPastelRed,
        icon = CoverDropIcons.Warning,
    )
}

@Preview
@Composable
private fun TwoLineButtonPreview() = CoverDropPreviewSurface {
    TwoLineButton(
        firstLine = "Source protection",
        secondLine = "Read more",
        onClick = {}
    )
}
