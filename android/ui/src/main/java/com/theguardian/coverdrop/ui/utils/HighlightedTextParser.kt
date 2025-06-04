package com.theguardian.coverdrop.ui.utils

import androidx.compose.material.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.withStyle

/**
 * The simplest possible parser for the example text. It supports using the `~` character to switch
 * between normal text and highlighted text.
 */
fun parseHighlightedTextIntoAnnotated(text: String, highlightColor: Color): AnnotatedString {
    val parts = text.split("~")
    val highlightStyle = SpanStyle(
        color = highlightColor,
        fontWeight = FontWeight.W500
    )

    return buildAnnotatedString {
        parts.forEachIndexed { index, part ->
            if (index % 2 == 0) {
                append(part)
            } else {
                withStyle(style = highlightStyle) {
                    append(part)
                }
            }
        }
    }
}

@Composable
fun highlightText(resId: Int): AnnotatedString {
    return parseHighlightedTextIntoAnnotated(
        text = stringResource(id = resId),
        highlightColor = MaterialTheme.colors.primary
    )
}

@Composable
fun highlightText(text: String): AnnotatedString {
    return parseHighlightedTextIntoAnnotated(
        text = text,
        highlightColor = MaterialTheme.colors.primary
    )
}
