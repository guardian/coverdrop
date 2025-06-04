package com.theguardian.coverdrop.ui.components

import android.content.Context
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import com.theguardian.coverdrop.ui.R
import com.theguardian.coverdrop.ui.theme.CoverDropPreviewSurface
import com.theguardian.coverdrop.ui.theme.Padding
import com.theguardian.coverdrop.ui.theme.PrimaryYellow

@Composable
fun StrapLine(modifier: Modifier = Modifier, content: @Composable() () -> Unit = {}) {
    val context = LocalContext.current
    Column(
        modifier = Modifier
                then modifier
    ) {
        Text(
            text = getHeaderSpannable(context),
            style = MaterialTheme.typography.h1,
            modifier = Modifier.testTag("coverdrop_strap_line_header")
        )
        Text(
            text = stringResource(R.string.component_strapline_content_explanation),
            modifier = Modifier.padding(top = Padding.M),
        )
        content()
    }
}

fun getHeaderSpannable(context: Context) = buildAnnotatedString {
    append(context.getString(R.string.component_strapline_header_part_1))
    append(" ")
    withStyle(style = SpanStyle(PrimaryYellow)) {
        append(context.getString(R.string.component_strapline_header_part_2))
    }
    append(" and ")
    withStyle(style = SpanStyle(PrimaryYellow)) {
        append(context.getString(R.string.component_strapline_header_part_3))
    }
}

@Preview(
    name = "Strapline",
    device = Devices.PIXEL_6,
    showSystemUi = false
)
@Composable
fun StrapLinePreview() = CoverDropPreviewSurface {
    StrapLine()
}
