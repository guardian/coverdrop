package com.theguardian.coverdrop.ui.activities

import android.os.Bundle
import android.util.Log
import android.view.MotionEvent
import android.view.WindowManager.LayoutParams
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.material.primarySurface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.Modifier
import androidx.core.view.WindowCompat
import androidx.navigation.NavHostController
import androidx.navigation.compose.rememberNavController
import com.theguardian.coverdrop.core.CoverDropConfiguration
import com.theguardian.coverdrop.core.CoverDropLib
import com.theguardian.coverdrop.core.ICoverDropLib
import com.theguardian.coverdrop.core.LockState
import com.theguardian.coverdrop.core.security.IBackgroundTimeoutGuard
import com.theguardian.coverdrop.core.security.IntegrityGuard
import com.theguardian.coverdrop.core.security.IntegrityViolation
import com.theguardian.coverdrop.core.security.IntegrityViolationCallback
import com.theguardian.coverdrop.core.ui.interfaces.LifecycleCallbackProxySilenceTicket
import com.theguardian.coverdrop.core.ui.interfaces.UncaughtExceptionHandlerProxySilenceTicket
import com.theguardian.coverdrop.ui.navigation.CoverDropDestinations
import com.theguardian.coverdrop.ui.navigation.CoverDropNavGraph
import com.theguardian.coverdrop.ui.screens.IntegrityViolationScreen
import com.theguardian.coverdrop.ui.theme.CoverDropColorPalette
import com.theguardian.coverdrop.ui.theme.CoverDropSurface
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import java.util.EnumSet
import javax.inject.Inject

@AndroidEntryPoint
class CoverDropActivity : ComponentActivity(), IntegrityViolationCallback {

    @Inject
    lateinit var coverDropConfiguration: CoverDropConfiguration

    @Inject
    lateinit var coverDropLib: ICoverDropLib

    @Inject
    lateinit var backgroundTimeoutGuard: IBackgroundTimeoutGuard

    // We cannot inject this property, as we want to call it before we enter `super.onCreate()`.
    // However, Dagger only finishes the injection through a life cycle callback inside that method.
    // For more background see the documentation of the `AndroidEntryPoint` annotation.
    private val silenceableLifecycleCallbackProxy =
        CoverDropLib.getSilenceableLifecycleCallbackProxy()
    private var mLifecycleSilenceTicket: LifecycleCallbackProxySilenceTicket? = null

    // For simplicity, we also manually manage the silenceable exception handler to match the
    // pattern used above.
    private val silenceableUncaughtExceptionHandler =
        CoverDropLib.getSilenceableUncaughtExceptionHandler()
    private var mExceptionHandlerSilenceTicket: UncaughtExceptionHandlerProxySilenceTicket? = null

    private var integrityViolationMutableStateFlow =
        MutableStateFlow(EnumSet.noneOf(IntegrityViolation::class.java))

    override fun onCreate(savedInstanceState: Bundle?) {
        mLifecycleSilenceTicket = silenceableLifecycleCallbackProxy.acquireSilenceTicket()
        mExceptionHandlerSilenceTicket = silenceableUncaughtExceptionHandler.acquireSilenceTicket()
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()

        if (coverDropConfiguration.disableScreenCaptureProtection) {
            // This Logcat output is safe, as it is only reachable in local test mode. And it is
            // important as it explains a difference between the expected security behaviour and
            // the actual one in local test mode.
            Log.d("CoverDropActivity", "Screen capture protection disabled as per config")
        } else {
            // Disable screenshots which also blanks out the app in the recent apps switcher.
            window.addFlags(LayoutParams.FLAG_SECURE)
        }

        IntegrityGuard.INSTANCE.addIntegrityViolationCallback(this)

        setContent {
            val integrityViolationsState = integrityViolationMutableStateFlow.collectAsState(
                initial = EnumSet.noneOf(IntegrityViolation::class.java)
            )
            WindowCompat.getInsetsController(window, window.decorView).isAppearanceLightStatusBars = false
            MainActivityContent(integrityViolationsState.value)
        }
    }

    @Composable
    fun MainActivityContent(violations: EnumSet<IntegrityViolation>) {
        val navController = rememberNavController()
        CoverDropSurface {
            // The background colour is set to the primary surface colour, so that the app
            // background matches the status bar colour.
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(CoverDropColorPalette.primarySurface)
                    .windowInsetsPadding(WindowInsets.statusBars)
            ) {
                if (violations.isNotEmpty()) {
                    IntegrityViolationScreen(violations = violations)
                } else {
                    CoverDropApp(
                        lockFlow = coverDropLib.getLockFlow(),
                        navController = navController,
                    )
                }
            }
        }
    }

    override fun onViolationsChanged(violations: EnumSet<IntegrityViolation>) {
        if (coverDropConfiguration.localTestMode) {
            // This Logcat output is safe, as it is only reachable in local test mode. And it is
            // important as it explains a difference between the expected security behaviour and
            // the actual one in local test mode.
            Log.d(
                "CoverDropActivity",
                "Ignoring integrity violations in local test mode: $violations"
            )
        } else {
            // This will show a screen that lists identified integrity violations.
            integrityViolationMutableStateFlow.value = violations
        }
    }

    override fun dispatchTouchEvent(ev: MotionEvent?): Boolean {
        ev?.let { IntegrityGuard.INSTANCE.checkMotionEvent(event = it) }
        return super.dispatchTouchEvent(ev)
    }

    /**
     * Implementation note: this should be called when the main activity of the integrating news
     * app is exited. In our test app it is a coincidence that the CoverDrop activity is also the
     * main activity.
     */
    override fun onDestroy() {
        super.onDestroy()
        window.clearFlags(LayoutParams.FLAG_SECURE)
        IntegrityGuard.INSTANCE.removeIntegrityViolationsCallback(this)
        mExceptionHandlerSilenceTicket?.release()
        mLifecycleSilenceTicket?.release()
    }

    override fun onPause() {
        super.onPause()
        backgroundTimeoutGuard.onPause()
    }

    override fun onResume() {
        super.onResume()
        backgroundTimeoutGuard.onResume()
    }
}

@Composable
fun CoverDropApp(lockFlow: SharedFlow<LockState>, navController: NavHostController) {
    // If we enter the locked state, we want to navigate back to the entry route. This might happen
    // if the user is in the app and the [BackgroundTimeoutGuard] locks the app.
    LaunchedEffect(lockFlow) {
        lockFlow.collect { lockState ->
            if (lockState == LockState.LOCKED) {
                navController.popBackStack(CoverDropDestinations.ENTRY_ROUTE, inclusive = false)
            }
        }
    }

    CoverDropNavGraph(
        navController = navController,
    )
}
