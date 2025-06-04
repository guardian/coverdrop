package com.theguardian.coverdrop.ui.theme

import androidx.compose.material.darkColors
import androidx.compose.ui.graphics.Color

val PrimaryYellow = Color(0xFFFFE500) // gsource: brandAlt.200
val BackgroundNeutral = Color(0xFF3F464A) // gsource: neutral.20
val SurfaceNeutral = Color(0xFF303538) // gsource: neutral.40, specialReport.200

val TextBlack = Color(0xFF000000)
val TextWhite = Color(0xFFFFFFFF)
val TextWhiteMuted = Color(0xFFE4E5E8) // gsource: specialReport.700

val NeutralMiddle = Color(0xFF999999) // gsource: neutral.60
val NeutralBrighter = Color(0xFFDCDCDC) // gsource: neutral.86

val BackgroundWarningPastelRed = Color(0xFFFF9081) // gsource: brandText.error
val BackgroundInfoPastelBlue = Color(0xFFABC2C9) // gsource: specialReport.500

val WarningPastelRed = Color(0xFFFF9081) // gsource: brandText.error
val InfoPastelBlue = Color(0xFFC1D8FC) // gsource: specialReport.500

val ChatTextColorPending = Color(0xFFFF7F0F) // from the exported svg
val ChatTextColorSent = Color(0xFF22874D) // from the exported svg

val SurfaceBorder = Color(0xFF999999) // gsource: neutral.60

/**
 * Our main color palette for the app based on gsource colors. The background is a neutral, darkish
 * grey, with a yellow primary color for buttons. Text is white on dark backgrounds.
 *
 * Some larger elements (e.g. cards) use a slightly darker grey for the surface color.
 */
internal val CoverDropColorPalette = darkColors(
    // General background
    background = BackgroundNeutral,
    onBackground = TextWhite,

    // Background for UI elements such as text boxes and cards
    surface = SurfaceNeutral,
    onSurface = TextWhite,

    // Primary color for the app (typically only buttons)
    primary = PrimaryYellow,
    primaryVariant = PrimaryYellow,
    onPrimary = TextBlack,

    secondary = PrimaryYellow,
    secondaryVariant = PrimaryYellow,
    onSecondary = TextBlack,
)
