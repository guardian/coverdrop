package com.theguardian.coverdrop.ui.screens

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material.CircularProgressIndicator
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.navigation.NavHostController
import androidx.navigation.NavOptions
import com.theguardian.coverdrop.ui.navigation.CoverDropDestinations
import com.theguardian.coverdrop.ui.theme.CoverDropSurface
import com.theguardian.coverdrop.ui.utils.ScreenContentWrapper
import com.theguardian.coverdrop.ui.viewmodels.SplashViewModel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

@Composable
fun SplashRoute(
    navController: NavHostController,
) {
    val viewModel = hiltViewModel<SplashViewModel>()

    SplashScreen(
        viewModel.canProceedState,
    ) {
        val navOptions =
            NavOptions.Builder().setPopUpTo(CoverDropDestinations.SPLASH_ROUTE, true).build()
        navController.navigate(CoverDropDestinations.ENTRY_ROUTE, navOptions)
    }
}

@Composable
fun SplashScreen(
    canProceedState: StateFlow<Boolean>,
    onProceed: () -> Unit = {},
) {
    val updatedOnProceed by rememberUpdatedState(newValue = onProceed)
    LaunchedEffect(true) {
        canProceedState.collect { canProceed ->
            if (canProceed) {
                // Minimal pause to ensure the spinner is visible and not just a flash
                delay(100)
                updatedOnProceed()
            }
        }
    }
ScreenContentWrapper {
    Column(
        modifier = Modifier.fillMaxSize(),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        CircularProgressIndicator(modifier = Modifier.testTag("coverdrop_splash_screen_spinner"))
    }
}
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun SplashScreenPreview() = CoverDropSurface {
    SplashScreen(
        canProceedState = MutableStateFlow(false),
        onProceed = {}
    )
}
