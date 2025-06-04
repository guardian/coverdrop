package com.theguardian.coverdrop.ui.theme

import androidx.compose.material.Typography
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import com.theguardian.coverdrop.ui.R

val guardianSansFontFamily = FontFamily(
    Font(R.font.guardian_sans_regular),
    Font(R.font.guardian_sans_medium, FontWeight.Medium), // Medium is alias for W500
    Font(R.font.guardian_sans_bold, FontWeight.Bold), // Bold is alias for W700
)

val guardianHeadlineFontFamily = FontFamily(
    Font(R.font.guardian_headline_regular),
    Font(R.font.guardian_headline_bold, FontWeight.Bold), // Bold is alias for W700
)

val CoverdropTypography = Typography(
    // h1 is the largest headline, reserved for short, important text or numerals.
    h1 = TextStyle(
        fontFamily = guardianHeadlineFontFamily,
        fontSize = 32.sp,
        fontWeight = FontWeight.Bold,
        lineHeight = 1.2.em,
    ),
    // h2 is the second largest headline, reserved for short, important text or numerals.
    h2 = TextStyle(
        fontFamily = guardianHeadlineFontFamily,
        fontSize = 22.sp,
        fontWeight = FontWeight.Bold,
        lineHeight = 1.2.em,
    ),
    //  h3 is the third largest headline, reserved for short, important text or numerals.
    h3 = TextStyle(
        fontFamily = guardianSansFontFamily,
        fontSize = 17.sp,
        fontWeight = FontWeight.Bold,
        lineHeight = 1.2.em,
    ),
    // body1 is the largest body, and is typically used for long-form writing as it works well for small text sizes.
    body1 = TextStyle(
        fontFamily = guardianSansFontFamily,
        fontSize = 16.sp,
        fontWeight = FontWeight.Normal,
        lineHeight = 1.3.em,
    ),
    // body2 is the smallest body, and is typically used for long-form writing as it works well for small text sizes.
    body2 = TextStyle(
        fontFamily = guardianSansFontFamily,
        fontSize = 12.sp,
        fontWeight = FontWeight.Normal,
        lineHeight = 1.3.em,
    ),
    // caption is one of the smallest font sizes. It is used sparingly to annotate imagery or to introduce a headline.
    caption = TextStyle(
        fontFamily = guardianSansFontFamily,
        fontSize = 12.sp,
        fontWeight = FontWeight.Normal,
        lineHeight = 1.2.em,
    ),
)
