package com.theguardian.coverdrop.ui.screens

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import com.theguardian.coverdrop.core.security.IntegrityGuard
import com.theguardian.coverdrop.core.security.IntegrityViolation
import com.theguardian.coverdrop.ui.R
import com.theguardian.coverdrop.ui.components.CoverDropTopAppBar
import com.theguardian.coverdrop.ui.components.SecondaryButton
import com.theguardian.coverdrop.ui.components.TopBarNavigationOption
import com.theguardian.coverdrop.ui.theme.CoverDropSurface
import com.theguardian.coverdrop.ui.theme.Padding
import com.theguardian.coverdrop.ui.utils.findComponentActivity
import java.util.EnumSet

@Composable
internal fun IntegrityViolationScreen(violations: EnumSet<IntegrityViolation>) {
    Column(modifier = Modifier.fillMaxHeight(1f)) {
        val context = LocalContext.current
        CoverDropTopAppBar(
            navigationOption = TopBarNavigationOption.Exit,
            onNavigationOptionPressed = { context.findComponentActivity()?.finish() }
        )

        Column(
            modifier = Modifier
                .verticalScroll(rememberScrollState())
                .weight(1f)
                .padding(Padding.L),
            verticalArrangement = Arrangement.Center,
            horizontalAlignment = Alignment.Start,
        ) {
            Text(
                text = stringResource(id = R.string.screen_violations_header),
                fontWeight = FontWeight.Bold,
            )
            for (violation in violations) {
                Text(
                    modifier = Modifier.padding(top = Padding.M),
                    text = "- ${violation.description}",
                )
            }
            Text(
                modifier = Modifier.padding(top = Padding.XL),
                text = stringResource(id = R.string.screen_violations_dismiss_and_ignore_explanation),
            )
            Spacer(modifier = Modifier.weight(1f))
            SecondaryButton(
                modifier = Modifier
                    .padding(vertical = Padding.M)
                    .fillMaxWidth(1f),
                text = stringResource(id = R.string.screen_violations_dismiss_button),
                onClick = { IntegrityGuard.INSTANCE.snooze(violations) }
            )
        }
    }
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun IntegrityViolationScreen_previewVarious() = CoverDropSurface {
    IntegrityViolationScreen(
        violations = EnumSet.of(
            IntegrityViolation.DEBUGGABLE,
            IntegrityViolation.OVERLAPPED_WINDOW
        )
    )
}
