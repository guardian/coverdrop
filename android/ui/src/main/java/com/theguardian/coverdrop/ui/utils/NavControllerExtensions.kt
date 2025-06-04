package com.theguardian.coverdrop.ui.utils

import androidx.navigation.NavController
import androidx.navigation.NavOptions

/**
 * Pop the back stack up to the specified destination and then navigate to the
 * specified destination.
 */
fun NavController.popBackStackAndThenNavigate(popUpTo: String, destination: String) {
    val options = NavOptions.Builder()
        .setPopUpTo(route = popUpTo, inclusive = false)
        .build()
    navigate(destination, options)
}
