package com.theguardian.coverdrop.ui.components

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.wrapContentSize
import androidx.compose.material.Card
import androidx.compose.material.Divider
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import com.theguardian.coverdrop.core.models.Message
import com.theguardian.coverdrop.ui.R
import com.theguardian.coverdrop.ui.theme.ChatTextColorPending
import com.theguardian.coverdrop.ui.theme.ChatTextColorSent
import com.theguardian.coverdrop.ui.theme.CoverDropPreviewSurface
import com.theguardian.coverdrop.ui.theme.Padding
import com.theguardian.coverdrop.ui.theme.RoundedCornerShape
import com.theguardian.coverdrop.ui.theme.SurfaceBorder
import com.theguardian.coverdrop.ui.utils.bold
import com.theguardian.coverdrop.ui.utils.humanFriendlyMessageTimeString
import java.time.Duration
import java.time.Instant

@Composable
fun ChatBubble(message: Message) {
    val textContent = when (message) {
        is Message.Pending -> message.message
        is Message.Received -> message.message
        is Message.Sent -> message.message
        else -> stringResource(id = R.string.component_chat_bubble_unsupported_message_type)
    }
    val timeText = humanFriendlyMessageTimeString(
        timestamp = message.timestamp,
        justNowString = stringResource(id = R.string.component_chat_bubble_just_now)
    )

    val backgroundColour = when (message) {
        is Message.Pending -> MaterialTheme.colors.background
        else -> MaterialTheme.colors.surface
    }
    val borderStroke = when (message) {
        is Message.Pending -> BorderStroke(
            width = 1.dp,
            color = SurfaceBorder
        )

        else -> null
    }

    Card(
        modifier = Modifier
            .fillMaxWidth()
            .wrapContentSize()
            .padding(Padding.S),
        border = borderStroke,
        shape = RoundedCornerShape.M,
        backgroundColor = backgroundColour,
    ) {
        Column(modifier = Modifier.fillMaxWidth()) {

            // the message
            Text(
                modifier = Modifier.padding(Padding.M),
                text = textContent,
                style = MaterialTheme.typography.body1,
            )

            // horizontal line
            Divider(
                color = Color(1f, 1f, 1f, 0.2f),
            )

            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(Padding.M)
            ) {
                // Indicators for pending or sent messages
                Row {
                    when (message) {
                        is Message.Pending -> {
                            Image(
                                modifier = Modifier
                                    .size(16.dp)
                                    .padding(end = 4.dp),
                                painter = painterResource(id = R.drawable.ic_chat_bubble_pending),
                                contentDescription = stringResource(id = R.string.component_chat_bubble_message_pending),
                                contentScale = ContentScale.Fit,
                            )
                            Text(
                                text = stringResource(id = R.string.component_chat_bubble_message_pending),
                                color = ChatTextColorPending,
                                style = MaterialTheme.typography.body2.bold(),
                            )
                        }

                        is Message.Sent -> {
                            Image(
                                modifier = Modifier
                                    .size(16.dp)
                                    .padding(end = 4.dp),
                                painter = painterResource(id = R.drawable.ic_chat_bubble_sent),
                                contentDescription = stringResource(id = R.string.component_chat_bubble_message_sent),
                                contentScale = ContentScale.Fit,
                            )
                            Text(
                                text = stringResource(id = R.string.component_chat_bubble_message_sent),
                                color = ChatTextColorSent,
                                style = MaterialTheme.typography.body2.bold(),
                            )
                        }

                        else -> {
                            /* received messages do not have a indicated status */
                        }
                    }
                }

                Text(text = timeText, style = MaterialTheme.typography.body2)
            }
        }
    }
}

@Preview(
    name = "Chat Bubble Sent (a week ago)",
    device = Devices.PIXEL_6,
    showSystemUi = false
)
@Composable
fun ChatBubblePreviewSentOneWeekAgo() = CoverDropPreviewSurface {
    ChatBubble(
        Message.Sent(
            message = stringResource(id = R.string.component_chat_bubble_demo_text),
            timestamp = Instant.now() - Duration.ofDays(7)
        )
    )
}

@Preview(
    name = "Chat Bubble Pending (a few minutes ago)",
    device = Devices.PIXEL_6,
    showSystemUi = false
)
@Composable
fun ChatBubblePreviewPending() = CoverDropPreviewSurface {
    ChatBubble(
        Message.Pending(
            message = stringResource(id = R.string.component_chat_bubble_demo_text),
            timestamp = Instant.now() - Duration.ofMinutes(42)
        )
    )
}

@Preview(
    name = "Chat Bubble Received (just now)",
    device = Devices.PIXEL_6,
    showSystemUi = false
)
@Composable
fun ChatBubblePreviewReceivedJustNow() = CoverDropPreviewSurface {
    ChatBubble(
        Message.Received(
            message = stringResource(id = R.string.component_chat_bubble_demo_text),
            timestamp = Instant.now()
        )
    )
}

@Preview(
    name = "Chat Bubble Unsupported (just now)",
    device = Devices.PIXEL_6,
    showSystemUi = false
)
@Composable
fun ChatBubblePreviewUnknown() = CoverDropPreviewSurface {
    ChatBubble(
        Message.Unknown(timestamp = Instant.now())
    )
}
