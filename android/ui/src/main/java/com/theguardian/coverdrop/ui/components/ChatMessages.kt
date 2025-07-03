package com.theguardian.coverdrop.ui.components

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import com.theguardian.coverdrop.core.models.Message
import com.theguardian.coverdrop.ui.R
import com.theguardian.coverdrop.ui.theme.CoverDropPreviewSurface
import com.theguardian.coverdrop.ui.utils.bold
import java.time.Instant

@Composable
fun ChatMessages(messages: List<Message>, remotePartyName: String, now: Instant) {
    // we track as the messages swap from side to side so we only show the name when it changes
    var lastMessageFromRemote: Boolean? = null

    Column(Modifier.fillMaxWidth(1f)) {
        for (message in messages) {
            val alignment = when (message) {
                is Message.Received -> Alignment.Start
                else -> Alignment.End
            }
            Row(
                Modifier
                    .align(alignment)
                    .fillMaxWidth(0.8f)
            ) {
                Column {
                    val thisMessageFromRemote = message.isFromRemote()
                    val showName = lastMessageFromRemote != thisMessageFromRemote
                    lastMessageFromRemote = thisMessageFromRemote

                    if (showName) {
                        Text(
                            text = if (thisMessageFromRemote) {
                                remotePartyName
                            } else {
                                stringResource(id = R.string.component_chat_bubble_you)
                            },
                            style = MaterialTheme.typography.body2.bold(),
                            modifier = Modifier
                                .padding(start = 12.dp, end = 12.dp, top = 4.dp)
                                .align(alignment)
                        )
                    }

                    ChatBubble(message, now)
                }
            }
        }
    }
}

@Preview(device = Devices.PIXEL_6)
@Composable
fun ChatMessagesPreview() = CoverDropPreviewSurface {
    val messages = listOf(
        Message.Sent(
            message = "Hey, I can talk about A, B, C! And I write some more words so this is a multi-line message",
            timestamp = Instant.parse("2023-03-17T13:37:00Z")
        ),
        Message.Received(
            message = "How interesting! Unfortunately, A and B are already known. I'm looking for something new.",
            timestamp = Instant.parse("2023-03-17T20:05:00Z")
        ),
        Message.Sent(
            message = "But what about C?",
            timestamp = Instant.parse("2023-03-18T10:13:00Z")
        ),
        Message.Pending(
            message = "Anyway, this is the info I have ...",
            timestamp = Instant.parse("2023-03-18T10:15:00Z")
        ),
    )
    ChatMessages(
        messages = messages,
        remotePartyName = "Alice",
        now = Instant.parse("2023-03-18T10:16:00Z"),
    )
}
