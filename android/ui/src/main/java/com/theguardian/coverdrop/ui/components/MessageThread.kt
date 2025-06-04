package com.theguardian.coverdrop.ui.components

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentSize
import androidx.compose.material.Card
import androidx.compose.material.Divider
import androidx.compose.material.ExperimentalMaterialApi
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.layout
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.constraintlayout.compose.ConstraintLayout
import com.theguardian.coverdrop.ui.R
import com.theguardian.coverdrop.ui.theme.CoverDropPreviewSurface
import com.theguardian.coverdrop.ui.theme.Padding
import com.theguardian.coverdrop.ui.theme.RoundedCornerShape

sealed class MessageThreadViewData(
    val fullName: String,
    val dateTime: String?,
) {
    data class Active(
        val name: String,
        val time: String?,
    ) : MessageThreadViewData(name, time)

    data class Inactive(
        val name: String,
        val time: String?,
    ) : MessageThreadViewData(name, time)
}

@OptIn(ExperimentalMaterialApi::class)
@Composable
fun MessageThread(viewData: MessageThreadViewData, onClick: () -> Unit = {}) {
    val backgroundColour = MaterialTheme.colors.background
    val borderStroke = BorderStroke(1.dp, Color(1f, 1f, 1f, 0.2f))
    val iconImage = painterResource(id = R.drawable.ic_speech_bubble)
    val isActive = viewData is MessageThreadViewData.Active

    Card(
        modifier = Modifier
            .fillMaxWidth()
            .wrapContentSize()
            .padding(Padding.M)
            .alpha(if (isActive) 1.0f else 0.3f),
        onClick = onClick,
        border = borderStroke,
        shape = RoundedCornerShape.M,
    ) {
        ConstraintLayout(
            modifier = Modifier
                .background(backgroundColour)
                .fillMaxWidth()
                .padding(Padding.M)
        ) {
            val (topSection, horizontalLine, icon, lastMessage, date) = createRefs()

            ConstraintLayout(
                modifier = Modifier
                    .constrainAs(topSection) {
                        top.linkTo(parent.top)
                    }
                    .fillMaxWidth()
                    .padding(Padding.M)
            ) {
                val (messagingWith, name, chevron) = createRefs()

                Text(
                    modifier = Modifier.constrainAs(messagingWith) {
                        top.linkTo(parent.top)
                        start.linkTo(parent.start)
                    },
                    text = stringResource(id = R.string.component_message_thread_messaging_with),
                    style = MaterialTheme.typography.h1,
                    color = MaterialTheme.colors.onBackground,
                    fontSize = 14.sp
                )

                // name
                Text(
                    modifier = Modifier.constrainAs(name) {
                        start.linkTo(messagingWith.start)
                        top.linkTo(messagingWith.bottom)
                        bottom.linkTo(horizontalLine.top)
                    },
                    text = viewData.fullName,
                    style = MaterialTheme.typography.h1,
                    color = MaterialTheme.colors.primary,
                    fontSize = 24.sp
                )

                // chevron
                CoverDropIcons.ChevronRight.AsComposable(
                    modifier = Modifier
                        .constrainAs(chevron) {
                            end.linkTo(parent.end)
                            top.linkTo(parent.top)
                            bottom.linkTo(parent.bottom)
                        }
                        .padding(Padding.M),
                    size = 32.dp,
                    tint = Color.Gray,
                )
            }

            Divider(
                modifier = Modifier
                    .constrainAs(horizontalLine) {
                        top.linkTo(topSection.bottom)
                        start.linkTo(parent.start)
                        end.linkTo(parent.end)
                    }
                    .layout { measurable, constraints ->
                        val placeable = measurable.measure(
                            constraints.copy(
                                maxWidth = constraints.maxWidth + 16.dp.roundToPx(),
                            )
                        )
                        layout(placeable.width, placeable.height) {
                            placeable.place(0, 0)
                        }
                    },
                color = Color(1f, 1f, 1f, 0.2f),
            )

            // Icon
            Image(
                modifier = Modifier
                    .padding(Padding.M)
                    .constrainAs(icon) {
                        start.linkTo(topSection.start)
                        top.linkTo(horizontalLine.bottom)
                    },
                painter = iconImage,
                contentDescription = stringResource(id = R.string.component_message_thread_last_message),
                contentScale = ContentScale.Fit,
            )

            if (viewData.dateTime != null) {
                // Last message
                Text(
                    modifier = Modifier.constrainAs(lastMessage) {
                        start.linkTo(icon.end)
                        top.linkTo(icon.top)
                        bottom.linkTo(icon.bottom)
                    },
                    text = stringResource(id = R.string.component_message_thread_last_message),
                    color = MaterialTheme.colors.onBackground,
                    fontSize = 12.sp,
                )

                // Date
                Text(
                    modifier = Modifier
                        .padding(Padding.M)
                        .constrainAs(date) {
                            top.linkTo(lastMessage.top)
                            bottom.linkTo(lastMessage.bottom)
                            end.linkTo(parent.end)
                        },
                    text = viewData.dateTime,
                    color = MaterialTheme.colors.onBackground,
                    fontSize = 12.sp,
                )
            } else {
                Text(
                    modifier = Modifier.constrainAs(lastMessage) {
                        start.linkTo(icon.end)
                        top.linkTo(icon.top)
                        bottom.linkTo(icon.bottom)
                    },
                    text = stringResource(id = R.string.component_message_thread_no_messages),
                    color = MaterialTheme.colors.onBackground,
                    fontSize = 12.sp,
                )
            }
        }
    }
}

@Preview(name = "Message Thread")
@Composable
fun MessageThreadPreview() = CoverDropPreviewSurface {
    MessageThread(
        MessageThreadViewData.Active(
            name = "Harry Davies",
            time = "18 March 2023 10:13am"
        )
    )
}

@Preview(name = "Message Thread")
@Composable
fun MessageThreadPreview_noMessages() = CoverDropPreviewSurface {
    MessageThread(
        MessageThreadViewData.Active(
            name = "Harry Davies",
            time = null
        )
    )
}
