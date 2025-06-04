package com.theguardian.coverdrop.ui.screens

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentSize
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.Card
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Tab
import androidx.compose.material.TabRow
import androidx.compose.material.TabRowDefaults
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation.NavHostController
import androidx.navigation.compose.rememberNavController
import com.google.accompanist.pager.ExperimentalPagerApi
import com.google.accompanist.pager.HorizontalPager
import com.google.accompanist.pager.pagerTabIndicatorOffset
import com.google.accompanist.pager.rememberPagerState
import com.theguardian.coverdrop.ui.R
import com.theguardian.coverdrop.ui.components.CoverDropIcons
import com.theguardian.coverdrop.ui.components.CoverDropTopAppBar
import com.theguardian.coverdrop.ui.components.ErrorMessageWithIcon
import com.theguardian.coverdrop.ui.components.PrimaryButton
import com.theguardian.coverdrop.ui.components.SecondaryButton
import com.theguardian.coverdrop.ui.theme.BackgroundNeutral
import com.theguardian.coverdrop.ui.theme.CoverDropSurface
import com.theguardian.coverdrop.ui.theme.NeutralMiddle
import com.theguardian.coverdrop.ui.theme.Padding
import com.theguardian.coverdrop.ui.theme.TextWhiteMuted
import com.theguardian.coverdrop.ui.utils.COVERDROP_SAMPLE_DATA
import com.theguardian.coverdrop.ui.viewmodels.JournalistCardInfo
import com.theguardian.coverdrop.ui.viewmodels.RecipientSelectionState
import com.theguardian.coverdrop.ui.viewmodels.RecipientSelectionViewModel
import com.theguardian.coverdrop.ui.viewmodels.SelectedRecipientViewModel
import com.theguardian.coverdrop.ui.viewmodels.TeamCardInfo
import com.theguardian.coverdrop.ui.viewmodels.toJournalistCardInfo
import com.theguardian.coverdrop.ui.viewmodels.toTeamsCardInfo
import kotlinx.coroutines.launch


@Composable
fun RecipientSelectionRoute(
    sharedSelectedRecipientViewModel: SelectedRecipientViewModel,
    navController: NavHostController,
) {
    val viewModel = hiltViewModel<RecipientSelectionViewModel>()

    val screenState = viewModel.screenState.collectAsStateWithLifecycle()
    val teams = viewModel.teams.collectAsStateWithLifecycle()
    val journalists = viewModel.journalists.collectAsStateWithLifecycle()
    val currentlySelectedTeam = viewModel.selectedTeam.collectAsStateWithLifecycle()

    RecipientSelectionScreen(
        navController = navController,
        screenState = screenState.value,
        teams = teams.value,
        journalists = journalists.value,
        currentlySelectedTeam = currentlySelectedTeam.value,
        selectTeam = { id -> viewModel.selectTeam(id = id) },
        confirmTeam = { id ->
            viewModel.confirmRecipient(
                id = id,
                outputViewModel = sharedSelectedRecipientViewModel,
                onFinished = { navController.popBackStack() },
            )
        },
        backToList = { viewModel.backToList() },
        selectAndConfirmJournalist = { id ->
            viewModel.confirmRecipient(
                id = id,
                outputViewModel = sharedSelectedRecipientViewModel,
                onFinished = { navController.popBackStack() },
            )
        },
    )
}


@Composable
private fun RecipientSelectionScreen(
    navController: NavHostController,
    screenState: RecipientSelectionState,
    teams: List<TeamCardInfo>?,
    journalists: List<JournalistCardInfo>?,
    currentlySelectedTeam: TeamCardInfo?,
    selectTeam: (String) -> Unit = {},
    confirmTeam: (String) -> Unit = {},
    backToList: () -> Unit = {},
    selectAndConfirmJournalist: (String) -> Unit = {},
    initialPage: Int = 0,
) {
    Column(modifier = Modifier.fillMaxHeight(1f)) {
        CoverDropTopAppBar(
            onNavigationOptionPressed = { navController.navigateUp() }
        )

        when (screenState) {
            RecipientSelectionState.SHOWING_SELECTION -> TeamAndJournalistSelectionContent(
                teams = teams,
                journalists = journalists,
                selectTeam = selectTeam,
                selectAndConfirmJournalist = selectAndConfirmJournalist,
                initialPage = initialPage,
            )

            RecipientSelectionState.CONFIRM_TEAM -> TeamConfirmationScreen(
                team = currentlySelectedTeam,
                confirmTeam = confirmTeam,
                backToList = backToList,
            )
        }

    }
}

private enum class SelectionTabs(
    val title: Int,
    val screen: @Composable (
        teams: List<TeamCardInfo>?,
        journalists: List<JournalistCardInfo>?,
        selectTeam: (String) -> Unit,
        selectAndConfirmJournalist: (String) -> Unit,
        onSwitchToJournalists: () -> Unit,
    ) -> Unit,
) {
    TEAMS(
        R.string.screen_select_recipient_tab_teams,
        { teams, _, selectTeam, _, onSwitchToJournalists ->
            TabTeams(
                teams,
                selectTeam,
                onSwitchToJournalists
            )
        }),
    JOURNALISTS(
        R.string.screen_select_recipient_tab_journalists,
        { _, journalists, _, selectAndConfirmJournalist, _ ->
            TabJournalists(
                journalists,
                selectAndConfirmJournalist
            )
        })
}

@Composable
@OptIn(ExperimentalPagerApi::class)
private fun TeamAndJournalistSelectionContent(
    teams: List<TeamCardInfo>?,
    journalists: List<JournalistCardInfo>?,
    selectTeam: (String) -> Unit = {},
    selectAndConfirmJournalist: (String) -> Unit = {},
    initialPage: Int = 0,
) {
    val coroutineScope = rememberCoroutineScope()
    val pagerState = rememberPagerState(initialPage = initialPage)
    val tabs = SelectionTabs.entries.toTypedArray()

    Column(modifier = Modifier.padding(16.dp)) {
        Text(
            text = stringResource(id = R.string.screen_select_recipient_header),
            style = MaterialTheme.typography.h1,
        )
        Text(
            text = stringResource(R.string.screen_select_recipient_text),
            modifier = Modifier.padding(top = Padding.M),
        )
    }

    TabRow(
        selectedTabIndex = pagerState.currentPage,
        indicator = { tabPositions ->
            TabRowDefaults.Indicator(
                Modifier.pagerTabIndicatorOffset(pagerState, tabPositions),
                color = MaterialTheme.colors.secondary
            )
        },
        backgroundColor = Color.Transparent
    ) {
        tabs.forEachIndexed { index, item ->
            Tab(
                selected = pagerState.currentPage == index,
                onClick = { coroutineScope.launch { pagerState.animateScrollToPage(index) } },
                text = {
                    Text(text = stringResource(item.title), overflow = TextOverflow.Ellipsis)
                }
            )
        }
    }

    val switchToJournalists = { coroutineScope.launch { pagerState.animateScrollToPage(1) } }
    HorizontalPager(
        count = tabs.size,
        state = pagerState,
        modifier = Modifier.fillMaxHeight(),
        verticalAlignment = Alignment.Top
    ) {
        tabs[it].screen(
            teams,
            journalists,
            selectTeam,
            selectAndConfirmJournalist,
        ) { switchToJournalists() }
    }
}

@Composable
private fun TabTeams(
    teams: List<TeamCardInfo>?,
    selectTeam: (String) -> Unit = {},
    onSwitchToJournalists: () -> Unit = {},
) {
    if (teams.isNullOrEmpty()) {
        Column(
            modifier = Modifier
                .padding(Padding.L)
                .fillMaxSize()
        ) {
            ErrorMessageWithIcon(
                text = stringResource(R.string.screen_select_recipient_empty_state_no_teams),
                icon = CoverDropIcons.Info,
                colorBorder = TextWhiteMuted,
                colorText = TextWhiteMuted,
                modifier = Modifier.clickable { onSwitchToJournalists() }
            )
        }
    } else {
        Column(
            modifier = Modifier
                .verticalScroll(rememberScrollState())
        ) {
            teams.forEach { team ->
                Card(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(top = Padding.M, start = Padding.M, end = Padding.M)
                        .defaultMinSize(minHeight = 48.dp)
                        .clickable { selectTeam(team.id) },
                    backgroundColor = BackgroundNeutral,
                    border = BorderStroke(0.5.dp, NeutralMiddle)
                ) {
                    Row(
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically,
                        modifier = Modifier
                            .fillMaxSize()
                            .padding(Padding.M)
                    ) {
                        Text(text = team.displayName, fontWeight = FontWeight.Bold)
                        CoverDropIcons.ChevronRight.AsComposable(
                            size = 36.dp,
                            tint = MaterialTheme.colors.onBackground
                        )
                    }
                }
            }
            Spacer(modifier = Modifier.padding(top = Padding.M))
        }
    }
}

@Composable
private fun TabJournalists(
    journalists: List<JournalistCardInfo>?,
    selectAndConfirmJournalist: (String) -> Unit = {},
) {
    Column(
        modifier = Modifier
            .verticalScroll(rememberScrollState())
    ) {
        for (journalist in journalists ?: emptyList()) {
            Card(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(top = Padding.M, start = Padding.M, end = Padding.M)
                    .defaultMinSize(minHeight = 48.dp),
                backgroundColor = BackgroundNeutral,
                border = BorderStroke(0.5.dp, NeutralMiddle)
            ) {
                Row(
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(Padding.M)
                ) {
                    Column {
                        Text(text = journalist.displayName, fontWeight = FontWeight.Bold)
                        Text(text = journalist.tagLine)
                    }
                    PrimaryButton(
                        text = stringResource(R.string.screen_select_recipient_button_select),
                        modifier = Modifier.wrapContentSize(align = Alignment.CenterEnd)
                    ) {
                        selectAndConfirmJournalist(journalist.id)
                    }
                }
            }
        }
        Spacer(modifier = Modifier.padding(top = Padding.M))
    }
}

@Composable
private fun TeamConfirmationScreen(
    team: TeamCardInfo?,
    confirmTeam: (String) -> Unit = {},
    backToList: () -> Unit = {},
) {
    if (team == null) return

    Column(modifier = Modifier.padding(16.dp)) {
        Text(text = team.displayName, style = MaterialTheme.typography.h1)
        Text(text = team.description, modifier = Modifier.padding(top = Padding.M))
        Spacer(modifier = Modifier.weight(1f))
        PrimaryButton(
            text = stringResource(R.string.screen_select_recipient_button_select_team),
            modifier = Modifier.fillMaxWidth(1f)
        ) {
            confirmTeam(
                team.id
            )
        }
        SecondaryButton(
            text = stringResource(R.string.screen_select_recipient_button_back_to_list),
            modifier = Modifier.fillMaxWidth(1f)
        ) { backToList() }
    }
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun RecipientSelectionScreen_showTeams() = CoverDropSurface {
    RecipientSelectionScreen(
        navController = rememberNavController(),
        screenState = RecipientSelectionState.SHOWING_SELECTION,
        teams = COVERDROP_SAMPLE_DATA.getTeams().map { it.toTeamsCardInfo() },
        journalists = COVERDROP_SAMPLE_DATA.getJournalists().map { it.toJournalistCardInfo() },
        currentlySelectedTeam = null,
        initialPage = 0,
    )
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun RecipientSelectionScreen_showTeamEmpty() = CoverDropSurface {
    RecipientSelectionScreen(
        navController = rememberNavController(),
        screenState = RecipientSelectionState.SHOWING_SELECTION,
        teams = emptyList(),
        journalists = COVERDROP_SAMPLE_DATA.getJournalists().map { it.toJournalistCardInfo() },
        currentlySelectedTeam = null,
        initialPage = 0,
    )
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun RecipientSelectionScreen_showJournalists() = CoverDropSurface {
    RecipientSelectionScreen(
        navController = rememberNavController(),
        screenState = RecipientSelectionState.SHOWING_SELECTION,
        teams = COVERDROP_SAMPLE_DATA.getTeams().map { it.toTeamsCardInfo() },
        journalists = COVERDROP_SAMPLE_DATA.getJournalists().map { it.toJournalistCardInfo() },
        currentlySelectedTeam = null,
        initialPage = 1,
    )
}

@Preview(device = Devices.PIXEL_6)
@Composable
private fun RecipientSelectionScreen_confirmTeam() = CoverDropSurface {
    RecipientSelectionScreen(
        navController = rememberNavController(),
        screenState = RecipientSelectionState.CONFIRM_TEAM,
        teams = emptyList(),
        journalists = emptyList(),
        currentlySelectedTeam = COVERDROP_SAMPLE_DATA.getTeams().first().toTeamsCardInfo(),
        initialPage = 1,
    )
}
