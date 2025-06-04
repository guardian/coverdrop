package com.theguardian.coverdrop.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material.CircularProgressIndicator
import androidx.compose.material.Divider
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Surface
import androidx.compose.material.Text
import androidx.compose.material.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import com.theguardian.coverdrop.ui.theme.BackgroundNeutral
import com.theguardian.coverdrop.ui.theme.CoverDropPreviewSurface
import com.theguardian.coverdrop.ui.theme.NeutralMiddle
import com.theguardian.coverdrop.ui.theme.Padding
import com.theguardian.coverdrop.ui.theme.RoundedCornerShape

@Composable
fun CoverDropProgressDialog(
    headingText: String,
    onDismissClick: () -> Unit,
) {
    Dialog(onDismissRequest = { onDismissClick() }) {
        Surface(
            shape = RoundedCornerShape.L,
            color = MaterialTheme.colors.background,
        ) {
            Box(
                modifier = Modifier, contentAlignment = Alignment.Center
            ) {
                Column(modifier = Modifier.padding(0.dp)) {

                    Text(
                        text = headingText,
                        fontWeight = FontWeight.Bold,
                        modifier = Modifier.padding(Padding.L)
                    )

                    Divider(
                        color = Color.Gray,
                        modifier = Modifier.fillMaxWidth(),
                    )

                    Column(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        CircularProgressIndicator(modifier = Modifier.padding(20.dp))
                    }
                }
            }
        }
    }
}

@Composable
fun CoverDropConfirmationDialog(
    headingText: String,
    bodyText: String,
    confirmText: String,
    onConfirmClick: () -> Unit,
    dismissText: String,
    onDismissClick: () -> Unit,
) {
    Dialog(onDismissRequest = { onDismissClick() }) {
        Surface(
            shape = RoundedCornerShape.L, color = BackgroundNeutral
        ) {
            Box(
                modifier = Modifier, contentAlignment = Alignment.Center
            ) {
                Column(modifier = Modifier.padding(0.dp)) {
                    Text(
                        text = headingText,
                        fontWeight = FontWeight.Bold,
                        modifier = Modifier.padding(Padding.L)
                    )

                    Divider(
                        color = NeutralMiddle,
                        modifier = Modifier.fillMaxWidth(),
                    )

                    Text(bodyText, modifier = Modifier.padding(all = Padding.L))

                    Row(
                        modifier = Modifier
                            .padding(all = Padding.L)
                            .fillMaxWidth(),
                        horizontalArrangement = Arrangement.End,
                    ) {
                        Spacer(modifier = Modifier.weight(1f))

                        FlatTextButton(text = dismissText, onClick = onDismissClick)

                        PrimaryButton(
                            onClick = onConfirmClick,
                            text = confirmText,
                            modifier = Modifier.wrapContentWidth()
                        )
                    }
                }
            }
        }
    }
}

@Composable
fun CoverDropErrorDialog(
    headingText: String,
    bodyText: String,
    dismissText: String,
    onDismissClick: () -> Unit,
) {
    Dialog(onDismissRequest = { onDismissClick() }) {
        Surface(
            shape = RoundedCornerShape.L, color = BackgroundNeutral
        ) {
            Box(
                modifier = Modifier, contentAlignment = Alignment.Center
            ) {
                Column(modifier = Modifier.padding(0.dp)) {
                    Text(
                        text = headingText,
                        fontWeight = FontWeight.Bold,
                        modifier = Modifier.padding(Padding.L)
                    )

                    Divider(
                        color = Color.Gray,
                        modifier = Modifier.fillMaxWidth(),
                    )

                    Text(bodyText, modifier = Modifier.padding(all = Padding.L))

                    Row(
                        modifier = Modifier
                            .padding(all = Padding.L)
                            .fillMaxWidth(),
                        horizontalArrangement = Arrangement.End,
                    ) {
                        TextButton(onClick = onDismissClick) {
                            Text(text = dismissText, color = MaterialTheme.colors.onBackground)
                        }
                    }
                }
            }
        }
    }
}

@Preview("Start a new conversation dialog")
@Composable
fun PreviewStartANewConversationDialog() = CoverDropPreviewSurface {
    CoverDropConfirmationDialog(
        headingText = "Heading text",
        bodyText = "Body text.",
        confirmText = "confirm",
        onConfirmClick = {},
        dismissText = "dismiss",
        onDismissClick = {},
    )
}

@Preview("Progress dialog")
@Composable
fun PreviewProgressDialog() = CoverDropPreviewSurface {
    CoverDropProgressDialog(
        headingText = "Deleting messages",
        onDismissClick = {},
    )
}

@Preview("Error dialog")
@Composable
fun PreviewErrorDialog() = CoverDropPreviewSurface {
    CoverDropErrorDialog(
        headingText = "Error",
        bodyText = "An error occurred.",
        dismissText = "Dismiss",
        onDismissClick = {},
    )
}
