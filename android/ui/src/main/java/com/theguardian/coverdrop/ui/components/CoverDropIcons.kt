package com.theguardian.coverdrop.ui.components

import androidx.compose.foundation.layout.size
import androidx.compose.material.Icon
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ChevronRight
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.DeleteForever
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Visibility
import androidx.compose.material.icons.filled.VisibilityOff
import androidx.compose.material.icons.filled.Warning
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.theguardian.coverdrop.ui.R


sealed class CoverDropIcons(val icon: ImageVector, val contentDescription: Int) {
    data object ChevronRight : CoverDropIcons(
        icon = Icons.Default.ChevronRight,
        contentDescription = R.string.icon_content_description_chevron_right
    )

    data object Close : CoverDropIcons(
        icon = Icons.Default.Close,
        contentDescription = R.string.icon_content_description_close
    )

    data object Delete : CoverDropIcons(
        icon = Icons.Default.DeleteForever,
        contentDescription = R.string.icon_content_description_delete
    )

    data object Edit : CoverDropIcons(
        icon = Icons.Default.Edit,
        contentDescription = R.string.icon_content_description_edit
    )

    data object Hide : CoverDropIcons(
        icon = Icons.Default.VisibilityOff,
        contentDescription = R.string.icon_content_description_hide
    )

    data object Info : CoverDropIcons(
        icon = Icons.Default.Info,
        contentDescription = R.string.icon_content_description_info
    )

    data object Refresh : CoverDropIcons(
        icon = Icons.Default.Refresh,
        contentDescription = R.string.icon_content_description_refresh
    )

    data object Reveal : CoverDropIcons(
        icon = Icons.Default.Visibility,
        contentDescription = R.string.icon_content_description_reveal
    )

    data object Warning : CoverDropIcons(
        icon = Icons.Default.Warning,
        contentDescription = R.string.icon_content_description_warning
    )

    @Composable
    fun AsComposable(modifier: Modifier = Modifier, size: Dp = 18.dp, tint: Color = Color.White) {
        Icon(
            imageVector = icon,
            modifier = Modifier
                .size(size)
                .then(modifier),
            contentDescription = contentDescription.toString(),
            tint = tint,
        )
    }
}
