package com.theguardian.coverdrop.ui.components

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material.LinearProgressIndicator
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import com.theguardian.coverdrop.ui.R
import com.theguardian.coverdrop.ui.theme.CoverDropPreviewSurface
import com.theguardian.coverdrop.ui.theme.Padding
import com.theguardian.coverdrop.ui.theme.PrimaryYellow
import com.theguardian.coverdrop.ui.theme.WarningPastelRed

@Composable
fun MessageLimitIndicator(percentFull: Float, messagesBelow: Boolean = true) {
    Column(
        modifier = Modifier
            .padding(top = 8.dp)
            .fillMaxWidth()
    ) {
        if (!messagesBelow) WarningMessages(percentFull, messagesBelow)

        LinearProgressIndicator(
            modifier = Modifier
                .fillMaxWidth()
                .height(4.dp)
                .testTag("message_limit_indicator"),
            backgroundColor = Color.LightGray,
            color = if (percentFull > 1.0f) WarningPastelRed else PrimaryYellow,
            progress = percentFull
        )

        if (messagesBelow) WarningMessages(percentFull, messagesBelow)
    }
}

@Composable
private fun WarningMessages(percentFull: Float, messagesBelow: Boolean) {
    if (percentFull > 1.0f) {
        Text(
            text = stringResource(R.string.component_message_limit_indicator_text_error_message_limit_reached),
            style = TextStyle(color = WarningPastelRed),
            modifier = Modifier.padding(top = if (messagesBelow) Padding.S else 0.dp),
        )

        Text(
            text = stringResource(R.string.component_message_limit_indicator_text_error_message_please_shorten),
            style = TextStyle(color = MaterialTheme.colors.onBackground),
            modifier = Modifier.padding(bottom = if (!messagesBelow) Padding.S else 0.dp),
        )
    }
}

@Preview(name = "Message Limit Indicator - Within Limit")
@Composable
fun MessageLimitIndicatorOk() = CoverDropPreviewSurface {
    MessageLimitIndicator(
        percentFull = 0.7f
    )
}


@Preview(name = "Message Limit Indicator - Too Long")
@Composable
fun MessageLimitIndicatorTooLong() = CoverDropPreviewSurface {
    MessageLimitIndicator(
        percentFull = 1.1f
    )
}


@Preview(name = "Message Limit Indicator - Too Long - Messages On Top")
@Composable
fun MessageLimitIndicatorTooLongMessagesOnTop() = CoverDropPreviewSurface {
    MessageLimitIndicator(
        percentFull = 1.1f,
        messagesBelow = false,
    )
}
