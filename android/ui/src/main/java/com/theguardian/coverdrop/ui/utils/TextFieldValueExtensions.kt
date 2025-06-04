package com.theguardian.coverdrop.ui.utils

import androidx.compose.ui.text.input.TextFieldValue

fun TextFieldValue.copyRestrictedToMaxLength(maxLength: Int): TextFieldValue {
    return if (this.text.length <= maxLength) {
        this
    } else {
        TextFieldValue(
            text = text.substring(0, maxLength),
            selection = selection,
            composition = composition,
        )
    }
}
