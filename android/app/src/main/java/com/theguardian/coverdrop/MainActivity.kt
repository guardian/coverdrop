package com.theguardian.coverdrop

import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material.Button
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Surface
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import com.theguardian.coverdrop.core.CoverDropLib
import com.theguardian.coverdrop.ui.activities.CoverDropActivity
import com.theguardian.coverdrop.ui.utils.ScreenContentWrapper
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch

class MainActivity : ComponentActivity() {
    private val theGuardianBrandColor = Color(0xFF0D2455)
    private val primaryColor = theGuardianBrandColor
    private val primaryBackgroundColor = Color.LightGray

    // This is a simple coroutine scope for the main activity.
    private val coroutineScope = CoroutineScope(Dispatchers.Main)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            MainScreen()
        }
    }

    @Composable
    private fun MainScreen() {
        val startScreenColors = MaterialTheme.colors.copy(
            primary = primaryColor,
            surface = primaryBackgroundColor
        )

        MaterialTheme(colors = startScreenColors) {
            Surface {
                ScreenContentWrapper {
                    MainContent()
                }
            }
        }
    }

    @Composable
    private fun MainContent() {
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(32.dp),
            contentAlignment = Alignment.Center
        ) {
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                Button(onClick = { launchCoverDrop() }) {
                    Text(text = "Open CoverDrop")
                }
                Text(
                    modifier = Modifier.padding(top = 16.dp),
                    text = "This is the launcher activity of the reference app. It serves as a placeholder where the real news reader app would be."
                )
            }
        }
    }

    override fun onResume() {
        super.onResume()
        coroutineScope.launch { CoverDropLib.onAppResume() }
    }

    override fun onPause() {
        super.onPause()
        CoverDropLib.onAppExit()
    }

    override fun onDestroy() {
        super.onDestroy()
        coroutineScope.cancel()
    }

    private fun launchCoverDrop() {
        val intent = Intent(this, CoverDropActivity::class.java)
        startActivity(intent)
    }
}
