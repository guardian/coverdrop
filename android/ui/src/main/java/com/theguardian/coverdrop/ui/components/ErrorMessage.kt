package com.theguardian.coverdrop.ui.components

import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import com.theguardian.coverdrop.ui.theme.CoverDropPreviewSurface
import com.theguardian.coverdrop.ui.theme.Padding
import com.theguardian.coverdrop.ui.theme.WarningPastelRed
import com.theguardian.coverdrop.ui.utils.COVERDROP_SAMPLE_DATA

@Composable
fun ErrorMessageWithIcon(
    text: String,
    icon: CoverDropIcons,
    modifier: Modifier = Modifier,
    colorBorder: Color = WarningPastelRed,
    colorText: Color = WarningPastelRed,
) {
    Row(
        modifier = modifier
            .border(width = 2.dp, color = colorBorder)
            .padding(Padding.M)
    ) {
        icon.AsComposable(size = 18.dp, tint = colorBorder, modifier = Modifier.padding(top = 2.dp))
        Text(
            text = text,
            style = MaterialTheme.typography.body1.copy(color = colorText),
            modifier = Modifier
                .padding(start = Padding.S)
                .testTag("warning_box_text")
        )
    }
}

@Preview
@Composable
fun ErrorMessageWithIconPreview() = CoverDropPreviewSurface {
    val errorMessage = COVERDROP_SAMPLE_DATA.getSampleErrorMessage(isFatal = false)
    ErrorMessageWithIcon(
        text = errorMessage.getString(LocalContext.current),
        icon = CoverDropIcons.Info,
    )
}
