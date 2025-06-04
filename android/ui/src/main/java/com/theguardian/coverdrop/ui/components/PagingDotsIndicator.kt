package com.theguardian.coverdrop.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import com.theguardian.coverdrop.ui.theme.CoverDropPreviewSurface
import com.theguardian.coverdrop.ui.theme.NeutralBrighter

@Composable
fun PagingDotsIndicator(
    modifier: Modifier,
    selectedIndex: Int,
    selectedColor: Color,
    unSelectedColor: Color,
    totalDots: Int,
) {
    Row(
        modifier = modifier
            .padding(4.dp)
            .wrapContentWidth()
            .wrapContentHeight() then (modifier)

    ) {
        repeat(totalDots) { index ->

            if (index == selectedIndex) {
                Box(
                    modifier = Modifier
                        .size(8.dp)
                        .scale(1.3f)
                        .clip(CircleShape)
                        .background(selectedColor)
                )
            } else {
                Box(
                    modifier = Modifier
                        .size(8.dp)
                        .clip(CircleShape)
                        .background(unSelectedColor)
                )
            }

            if (index != totalDots - 1) {
                Spacer(modifier = Modifier.padding(horizontal = 5.dp))
            }
        }
    }

}

@Preview
@Composable
fun DotsIndicatorPreview() = CoverDropPreviewSurface {
    PagingDotsIndicator(
        modifier = Modifier,
        selectedIndex = 1,
        selectedColor = MaterialTheme.colors.primary,
        unSelectedColor = NeutralBrighter,
        totalDots = 6
    )
}
