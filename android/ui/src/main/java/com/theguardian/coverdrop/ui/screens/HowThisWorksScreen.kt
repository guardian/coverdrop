package com.theguardian.coverdrop.ui.screens

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.NavHostController
import androidx.navigation.compose.rememberNavController
import com.theguardian.coverdrop.ui.R
import com.theguardian.coverdrop.ui.components.CoverDropTopAppBar
import com.theguardian.coverdrop.ui.components.PagingDotsIndicator
import com.theguardian.coverdrop.ui.components.PrimaryButton
import com.theguardian.coverdrop.ui.navigation.CoverDropDestinations
import com.theguardian.coverdrop.ui.theme.CoverDropSurface
import com.theguardian.coverdrop.ui.theme.Padding
import com.theguardian.coverdrop.ui.theme.PrimaryYellow
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch

data class HowThisWorksPagerItem(
    val titleId: Int,
    val contentId: Int,
    val imageId: Int,
)

private val pagerItems = listOf(
    HowThisWorksPagerItem(
        R.string.screen_how_this_works_pager1_title,
        R.string.screen_how_this_works_pager1_content,
        R.drawable.ic_how_this_works_1
    ),
    HowThisWorksPagerItem(
        R.string.screen_how_this_works_pager2_title,
        R.string.screen_how_this_works_pager2_content,
        R.drawable.ic_how_this_works_2
    ),
    HowThisWorksPagerItem(
        R.string.screen_how_this_works_pager3_title,
        R.string.screen_how_this_works_pager3_content,
        R.drawable.ic_how_this_works_3
    )
)


@Composable
fun HowThisWorksRoute(navController: NavHostController) {
    HowThisWorksScreen(navController = navController)
}

@Composable
fun HowThisWorksScreen(
    navController: NavHostController,
    coroutineScope: CoroutineScope = rememberCoroutineScope(),
    initialPage: Int = 0,
) {
    val pagerState = rememberPagerState(
        initialPage = initialPage,
        initialPageOffsetFraction = 0f,
        pageCount = { pagerItems.size }
    )
    val context = LocalContext.current

    Column(modifier = Modifier.fillMaxHeight(1f)) {
        CoverDropTopAppBar(
            onNavigationOptionPressed = { navController.navigateUp() }
        )
        Column(
            verticalArrangement = Arrangement.Top,
            modifier = Modifier
                .padding(Padding.L)
                .weight(1.0f)
        ) {


            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .weight(1.0f)
            ) {
                HorizontalPager(
                    state = pagerState,
                    modifier = Modifier.fillMaxSize()
                ) { page ->

                    val item = pagerItems[page]

                    Column(
                        modifier = Modifier
                            .fillMaxSize()
                            .verticalScroll(rememberScrollState())
                    ) {

                        Image(
                            modifier = Modifier
                                .weight(1f)
                                .align(Alignment.CenterHorizontally)
                                .padding(bottom = Padding.M),
                            painter = painterResource(id = item.imageId),
                            contentDescription = context.getString(item.titleId),
                            contentScale = ContentScale.Fit,
                        )


                        Text(
                            text = stringResource(R.string.screen_how_this_works_header),
                            style = MaterialTheme.typography.h1.copy(fontSize = 27.sp),
                            modifier = Modifier
                                .align(Alignment.CenterHorizontally)
                                .padding(bottom = Padding.S),
                            textAlign = TextAlign.Center,
                        )

                        Text(
                            modifier = Modifier
                                .align(Alignment.CenterHorizontally)
                                .padding(bottom = Padding.L),
                            text = context.getString(item.titleId),
                            style = MaterialTheme.typography.h2.copy(
                                color = MaterialTheme.colors.primary,
                                fontSize = 20.sp
                            ),
                            textAlign = TextAlign.Center,
                        )

                        Text(
                            modifier = Modifier
                                .align(Alignment.CenterHorizontally)
                                .padding(bottom = Padding.XL),
                            text = context.getString(item.contentId),
                            textAlign = TextAlign.Center,
                        )

                        Spacer(modifier = Modifier.weight(0.025f))
                    }
                }

                PagingDotsIndicator(
                    modifier = Modifier
                        .padding(0.dp)
                        .align(Alignment.BottomCenter),
                    totalDots = pagerItems.size,
                    selectedIndex = pagerState.currentPage,
                    selectedColor = PrimaryYellow,
                    unSelectedColor = Color.Gray,
                )
            }
        }

        val isLastPage = pagerState.currentPage == pagerItems.size - 1
        PrimaryButton(
            text = if (isLastPage) {
                stringResource(R.string.screen_how_this_works_set_up_my_passphrase)
            } else {
                stringResource(
                    R.string.screen_how_this_works_continue
                )
            },
            onClick = {
                if (isLastPage) {
                    navController.navigate(CoverDropDestinations.NEW_SESSION_ROUTE)
                } else {
                    coroutineScope.launch {
                        pagerState.animateScrollToPage(pagerState.currentPage + 1)
                    }
                }
            },
            modifier = Modifier
                .padding(start = Padding.L, end = Padding.L, bottom = Padding.XL)
                .fillMaxWidth(1f)
        )
    }
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun HowThisWorksPreview_0() = CoverDropSurface {
    HowThisWorksScreen(navController = rememberNavController(), initialPage = 0)
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun HowThisWorksPreview_1() = CoverDropSurface {
    HowThisWorksScreen(navController = rememberNavController(), initialPage = 1)
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun HowThisWorksPreview_2() = CoverDropSurface {
    HowThisWorksScreen(navController = rememberNavController(), initialPage = 2)
}

@Preview(heightDp = 500)
@Composable
private fun HowThisWorksPreview_smallScreen() {
    CoverDropSurface {
        HowThisWorksScreen(navController = rememberNavController(), initialPage = 0)
    }
}
