package com.theguardian.coverdrop.ui.components

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material.Button
import androidx.compose.material.ButtonDefaults
import androidx.compose.material.Divider
import androidx.compose.material.LocalContentAlpha
import androidx.compose.material.LocalContentColor
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Text
import androidx.compose.material.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.theguardian.coverdrop.ui.theme.BackgroundNeutral
import com.theguardian.coverdrop.ui.theme.CoverDropPreviewSurface
import com.theguardian.coverdrop.ui.theme.CoverDropSampleTheme
import com.theguardian.coverdrop.ui.theme.NeutralMiddle
import com.theguardian.coverdrop.ui.theme.Padding
import com.theguardian.coverdrop.ui.theme.RoundedCornerShape

private const val BUTTON_INNER_PADDING_DP = 3.5f
private const val BUTTON_OUTER_PADDING_BOTTOM_DP = 1.5f

@Composable
fun PrimaryButton(
    text: String,
    modifier: Modifier = Modifier,
    icon: CoverDropIcons? = null,
    enabled: Boolean = true,
    onClick: () -> Unit,
) {
    Button(
        onClick = onClick,
        enabled = enabled,
        modifier = Modifier
            .padding(bottom = BUTTON_OUTER_PADDING_BOTTOM_DP.dp)
            .then(modifier),
        shape = RoundedCornerShape.HalfRounded,
    ) {
        Row(modifier = Modifier.padding(BUTTON_INNER_PADDING_DP.dp)) {
            PrependedOptionalIcon(icon)
            Text(
                text = text,
                fontWeight = FontWeight.Bold,
                textAlign = TextAlign.Center,
                letterSpacing = 0.sp,
            )
        }
    }
}

@Preview
@Composable
fun PrimaryButtonPreview() = CoverDropSampleTheme {
    PrimaryButton(text = "Primary Button", icon = CoverDropIcons.Refresh, onClick = {})
}

@Preview
@Composable
fun PrimaryButtonPreviewLongText() = CoverDropSampleTheme {
    PrimaryButton(
        text = "A very super mega long text that wraps around to a second line",
        icon = CoverDropIcons.Refresh,
        onClick = {})
}

@Composable
fun SecondaryButton(
    text: String,
    modifier: Modifier = Modifier,
    icon: CoverDropIcons? = null,
    onClick: () -> Unit,
) {
    Button(
        onClick = onClick,
        modifier = Modifier
            .padding(bottom = BUTTON_OUTER_PADDING_BOTTOM_DP.dp)
            .then(modifier),
        shape = RoundedCornerShape.HalfRounded,
        border = BorderStroke(1.dp, Color.White),
        colors = ButtonDefaults.outlinedButtonColors(
            backgroundColor = BackgroundNeutral,
            contentColor = MaterialTheme.colors.onBackground
        ),
    ) {
        Row(modifier = Modifier.padding(BUTTON_INNER_PADDING_DP.dp)) {
            PrependedOptionalIcon(icon)
            Text(
                text = text,
                fontWeight = FontWeight.Bold,
                textAlign = TextAlign.Center,
                letterSpacing = 0.sp,
            )
        }
    }
}

@Preview
@Composable
fun SecondaryButtonPreviewLongText() = CoverDropSampleTheme {
    SecondaryButton(text = "Secondary Button", icon = CoverDropIcons.Refresh, onClick = {})
}

@Preview
@Composable
fun SecondaryButtonPreview() = CoverDropSampleTheme {
    SecondaryButton(
        text = "A very super mega long text that wraps around to a second line",
        icon = CoverDropIcons.Refresh,
        onClick = {})
}

@Composable
fun FlatTextButton(
    text: String,
    modifier: Modifier = Modifier,
    icon: CoverDropIcons? = null,
    onClick: () -> Unit,
) {
    TextButton(onClick = onClick, modifier = modifier) {
        PrependedOptionalIcon(icon, tint = MaterialTheme.colors.onBackground)
        Text(
            text = text,
            fontWeight = FontWeight.Bold,
            color = MaterialTheme.colors.onBackground,
            textAlign = TextAlign.Center,
            letterSpacing = 0.sp,
        )
    }
}

@Preview
@Composable
fun FlatTextButtonPreview() = CoverDropSampleTheme {
    FlatTextButton(text = "Flat Text Button", icon = CoverDropIcons.Refresh, onClick = {})
}

@Composable
fun ChevronTextButton(
    text: String,
    modifier: Modifier = Modifier,
    onClick: () -> Unit,
) {
    TextButton(onClick = onClick, modifier = modifier.fillMaxWidth()) {
        Row(horizontalArrangement = Arrangement.SpaceBetween, modifier = modifier.fillMaxWidth()) {
            Text(
                text = text,
                fontWeight = FontWeight.Bold,
                color = MaterialTheme.colors.onBackground,
                letterSpacing = 0.sp,
            )
            CoverDropIcons.ChevronRight.AsComposable()
        }
    }
}

@Composable
fun ChevronTextDirectlyAfterButton(
    text: String,
    modifier: Modifier = Modifier,
    onClick: () -> Unit,
) {
    TextButton(onClick = onClick, modifier = modifier, contentPadding = PaddingValues(0.dp)) {
        Text(
            text = text,
            fontWeight = FontWeight.Bold,
            color = MaterialTheme.colors.onBackground,
            letterSpacing = 0.sp,
        )
        CoverDropIcons.ChevronRight.AsComposable()
    }
}

@Preview
@Composable
fun ChevronTextButtonPreview() = CoverDropSampleTheme {
    ChevronTextButton(text = "Chevron Text Button", onClick = {})
}

@Preview
@Composable
private fun ChevronTextDirectlyAfterButtonPreview() = CoverDropSampleTheme {
    ChevronTextDirectlyAfterButton(text = "Chevron Text Button", onClick = {})
}

data class ChevronTextButtonGroupRowInformation(
    val text: String,
    val onClick: () -> Unit = {},
)

@Composable
fun ChevronTextButtonGroup(
    buttons: List<ChevronTextButtonGroupRowInformation>,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape.M)
            .background(MaterialTheme.colors.surface)
    ) {
        buttons.forEachIndexed { index, (text, onClick) ->
            val isLast = index == buttons.size - 1

            TextButton(
                onClick = onClick,
                modifier = modifier
                    .fillMaxWidth()
            ) {
                Row(
                    horizontalArrangement = Arrangement.SpaceBetween,
                    modifier = modifier
                        .fillMaxWidth()
                        .padding(horizontal = Padding.M)
                ) {
                    Text(
                        text = text,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colors.onBackground,
                        letterSpacing = 0.sp,
                    )
                    CoverDropIcons.ChevronRight.AsComposable(tint = NeutralMiddle)
                }
            }

            if (!isLast) Divider(
                color = NeutralMiddle,
                thickness = 0.5.dp,
                modifier = Modifier.padding(start = Padding.L)
            )
        }
    }
}

@Preview
@Composable
fun ChevronTextButtonGroupPreview() = CoverDropPreviewSurface {
    ChevronTextButtonGroup(
        listOf(
            ChevronTextButtonGroupRowInformation("Button 1"),
            ChevronTextButtonGroupRowInformation("Button 2"),
            ChevronTextButtonGroupRowInformation("Button 3"),
        )
    )
}

@Composable
private fun PrependedOptionalIcon(
    coverDropIcon: CoverDropIcons?,
    tint: Color = LocalContentColor.current.copy(alpha = LocalContentAlpha.current),
) {
    if (coverDropIcon != null) {
        coverDropIcon.AsComposable(tint = tint)
        Spacer(modifier = Modifier.width(10.dp))
    }
}
