package com.theguardian.coverdrop.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.InlineTextContent
import androidx.compose.foundation.text.appendInlineContent
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.Divider
import androidx.compose.material.Icon
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Text
import androidx.compose.material.TextField
import androidx.compose.material.TextFieldDefaults
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Lock
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.Placeholder
import androidx.compose.ui.text.PlaceholderVerticalAlign
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.sp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.navigation.NavHostController
import com.theguardian.coverdrop.core.models.JournalistInfo
import com.theguardian.coverdrop.core.models.Message
import com.theguardian.coverdrop.core.models.MessageThread
import com.theguardian.coverdrop.ui.R
import com.theguardian.coverdrop.ui.components.ChatMessages
import com.theguardian.coverdrop.ui.components.CoverDropTopAppBar
import com.theguardian.coverdrop.ui.components.MessageLimitIndicator
import com.theguardian.coverdrop.ui.components.PrimaryButton
import com.theguardian.coverdrop.ui.components.SecondaryButton
import com.theguardian.coverdrop.ui.theme.CoverDropSurface
import com.theguardian.coverdrop.ui.theme.NeutralMiddle
import com.theguardian.coverdrop.ui.theme.Padding
import com.theguardian.coverdrop.ui.theme.RoundedCorners
import com.theguardian.coverdrop.ui.utils.COVERDROP_SAMPLE_DATA
import com.theguardian.coverdrop.ui.viewmodels.ConversationViewModel
import java.time.Duration
import java.time.Instant

@Composable
fun ConversationRoute(
    navController: NavHostController,
) {
    val viewModel = hiltViewModel<ConversationViewModel>()

    val activeMessageThreadState = viewModel.activeConversation.collectAsState()
    val messagePercentSize = viewModel.messageSizeState.collectAsState(0f)

    ConversationScreen(
        thread = activeMessageThreadState.value,
        onMessageChanged = { newValue -> viewModel.onMessageChanged(newValue) },
        totalMessageSizePercent = messagePercentSize.value,
        onSendMessage = { viewModel.onSendMessage() },
        navigateBack = { navController.navigateUp() },
    )
}

@Composable
private fun ConversationScreen(
    thread: MessageThread?,
    navigateBack: () -> Unit = {},
    onMessageChanged: (String) -> Unit = {},
    totalMessageSizePercent: Float = 0f,
    onSendMessage: () -> Unit = {},
    initialMessage: String? = null,
    now: Instant = Instant.now(),
) {
    val messages = thread?.messages ?: emptyList()

    Column(modifier = Modifier.fillMaxHeight(1f)) {
        CoverDropTopAppBar(
            onNavigationOptionPressed = navigateBack,
        )

        val scrollState = rememberScrollState()
        Column(
            modifier = Modifier
                .verticalScroll(scrollState)
                .fillMaxWidth(1f)
                .weight(1f)
        ) {
            ConversationMainContent(messages, thread, now)
        }

        LaunchedEffect(messages) {
            scrollState.animateScrollTo(scrollState.maxValue)
        }

        Divider(color = NeutralMiddle)

        Column(
            modifier = Modifier.padding(start = Padding.M, end = Padding.M, bottom = Padding.M)
        ) {
            ConversationMessageComposer(
                onMessageChanged,
                totalMessageSizePercent,
                onSendMessage,
                initialMessage
            )
        }
    }
}

@Composable
private fun ConversationMainContent(
    messages: List<Message>,
    thread: MessageThread?,
    now: Instant,
) {
    if (messages.isEmpty()) {
        Text(
            text = stringResource(R.string.screen_conversation_empty),
            textAlign = TextAlign.Center,
            modifier = Modifier
                .fillMaxWidth()
                .padding(Padding.M),
        )
    } else {
        thread?.recipient?.let {
            SecureConversationHeading(it)
        }

        ChatMessages(messages, thread?.recipient?.displayName ?: "", now)

        // information box if the last message comes from the user; depends on the message status
        when (messages.lastOrNull()) {
            is Message.Pending -> Text(
                text = stringResource(R.string.screen_conversation_pending),
                textAlign = TextAlign.Center,
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(Padding.M),
            )

            is Message.Sent -> Text(
                text = stringResource(R.string.screen_conversation_sent),
                textAlign = TextAlign.Center,
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(Padding.M),
            )

            else -> {}
        }
    }
}

@Composable
private fun SecureConversationHeading(info: JournalistInfo) {
    val text = buildAnnotatedString {
        appendInlineContent("secure_conversation_icon", "[icon]")
        append(
            stringResource(
                R.string.screen_conversation_heading_secure_conversation,
                info.displayName
            )
        )
    }

    val inlineContent = mapOf(
        "secure_conversation_icon" to InlineTextContent(
            placeholder = Placeholder(
                width = 22.sp,
                height = 18.sp,
                placeholderVerticalAlign = PlaceholderVerticalAlign.TextCenter
            ),
            children = {
                Icon(
                    imageVector = Icons.Default.Lock,
                    contentDescription = null,
                    tint = MaterialTheme.colors.onBackground,
                )
            }
        )
    )

    Text(
        text = text,
        inlineContent = inlineContent,
        fontWeight = FontWeight.W500,
        textAlign = TextAlign.Center,
        modifier = Modifier
            .fillMaxWidth()
            .padding(top = Padding.L, bottom = Padding.M, start = Padding.M, end = Padding.M)
    )
}

@Composable
private fun ConversationMessageComposer(
    onMessageChanged: (String) -> Unit = {},
    totalMessageSizePercent: Float,
    onSendMessage: () -> Unit = {},
    testMessage: String? = null,
) {
    var isComposing by rememberSaveable { mutableStateOf(testMessage != null) }
    val focusManager = LocalFocusManager.current
    val foregroundColor = MaterialTheme.colors.onBackground

    if (isComposing) {
        var messageText by rememberSaveable(stateSaver = TextFieldValue.Saver) {
            mutableStateOf(TextFieldValue(testMessage ?: ""))
        }
        val tooLong = totalMessageSizePercent >= 1.0f
        MessageLimitIndicator(percentFull = totalMessageSizePercent, messagesBelow = false)
        TextField(
            value = messageText,
            onValueChange = { messageText = it; onMessageChanged(it.text) },
            singleLine = false,
            maxLines = 3,
            colors = TextFieldDefaults.textFieldColors(
                backgroundColor = Color.Transparent,
                unfocusedIndicatorColor = Color.Transparent,
                focusedIndicatorColor = Color.Transparent,
            ),
            modifier = Modifier
                .padding(top = Padding.S)
                .fillMaxWidth()
                .background(MaterialTheme.colors.surface)
                .drawBehind {
                    drawRoundRect(
                        color = foregroundColor,
                        style = Stroke(width = 2f),
                        cornerRadius = CornerRadius(
                            x = RoundedCorners.XS.toPx(),
                            y = RoundedCorners.XS.toPx()
                        )
                    )
                }
                .testTag("edit_message")
        )
        PrimaryButton(
            text = stringResource(R.string.screen_new_message_button_send_message),
            enabled = !tooLong && messageText.text.isNotEmpty(),
            onClick = {
                focusManager.clearFocus()
                onMessageChanged(messageText.text)
                onSendMessage()
                isComposing = false
            },
            modifier = Modifier.padding(top = Padding.M)
        )

    } else {
        SecondaryButton(
            text = stringResource(R.string.screen_conversation_button_send_new),
            onClick = { isComposing = true },
            modifier = Modifier
                .padding(top = Padding.S)
                .fillMaxWidth(1f)
        )
    }
}

@Preview(device = Devices.PIXEL_6)
@Composable
fun ConversationScreenPreviewEmpty() = CoverDropSurface {
    ConversationScreen(thread = null)
}

@Preview(device = Devices.PIXEL_6)
@Composable
fun ConversationScreenPreviewShortConversation() = CoverDropSurface {
    val thread = COVERDROP_SAMPLE_DATA.getSampleThread(numMessages = 1)
    ConversationScreen(thread = thread)
}

@Preview(device = Devices.PIXEL_6)
@Composable
fun ConversationScreenPreviewLongConversation() = CoverDropSurface {
    val thread = COVERDROP_SAMPLE_DATA.getSampleThread()
    ConversationScreen(thread = thread)
}

@Preview(device = Devices.PIXEL_6)
@Composable
fun ConversationScreenPreviewInComposeMode() = CoverDropSurface {
    val thread = COVERDROP_SAMPLE_DATA.getSampleThread(numMessages = 1, lastMessageIsSent = true)
    ConversationScreen(thread = thread, totalMessageSizePercent = 0.1f, initialMessage = "Hello")
}

@Preview(device = Devices.PIXEL_6)
@Composable
fun ConversationScreenPreviewInComposeModeWithTooLongMessage() = CoverDropSurface {
    val thread = COVERDROP_SAMPLE_DATA.getSampleThread(numMessages = 1)
    val message = COVERDROP_SAMPLE_DATA.getSampleMessage(wordCount = 100)
    ConversationScreen(thread = thread, totalMessageSizePercent = 1.1f, initialMessage = message)
}

@Preview(device = Devices.PIXEL_6)
@Composable
fun ConversationScreenPreviewWithExpiringAndExpiredMessages() = CoverDropSurface {
    val thread = COVERDROP_SAMPLE_DATA.getSampleThread(numMessages = 2)
    ConversationScreen(
        thread = thread,
        totalMessageSizePercent = 1.1f,
        now = Instant.now() + Duration.ofDays(13).plusHours(22)
            .plusMinutes(30) // 13 days, 21 hours, 30 minutes in the future
    )
}
