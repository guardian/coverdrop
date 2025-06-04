package com.theguardian.coverdrop.ui.utils

import android.content.Context
import android.content.ContextWrapper
import androidx.activity.ComponentActivity

/**
 * Find the [ComponentActivity] from the [Context] by traversing up the [ContextWrapper] hierarchy.
 */
tailrec fun Context.findComponentActivity(): ComponentActivity? = when (this) {
    is ComponentActivity -> this
    is ContextWrapper -> baseContext.findComponentActivity()
    else -> null
}
