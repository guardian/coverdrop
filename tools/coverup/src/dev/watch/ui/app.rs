use std::{collections::BTreeMap, iter, time::Duration};

use crossterm::event::{Event, KeyCode, KeyEventKind};
use futures::{FutureExt as _, StreamExt as _};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{
        Block, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
    DefaultTerminal, Frame,
};
use tokio::{
    sync::broadcast::Receiver,
    time::{interval, Instant},
};

use crate::{
    coverdrop_service::CoverDropService,
    dev::watch::{
        builder::BuilderSignal,
        fs::FsSignal,
        k8s::K8sSignal,
        status::{BuildStatus, ServiceStatus},
    },
};

struct AppState {
    pub frame_start: Instant,
    pub elapsed: f64,
    pub quit: bool,
    pub services_list_state: ListState,
    pub scroll_bar_state: ScrollbarState,
}

impl AppState {
    pub fn new() -> Self {
        let mut services_list_state = ListState::default();
        services_list_state.select_first();
        let scroll_bar_state = ScrollbarState::default();

        Self {
            frame_start: Instant::now(),
            elapsed: 0.0,
            quit: false,
            services_list_state,
            scroll_bar_state,
        }
    }
}

pub struct App {
    fs_signal_rx: Receiver<FsSignal>,
    builder_signal_rx: Receiver<BuilderSignal>,
    k8s_signal_rx: Receiver<K8sSignal>,

    terminal: DefaultTerminal,

    state: AppState,
    services: BTreeMap<CoverDropService, ServiceStatus>,
}

impl App {
    pub fn new(
        fs_signal_rx: Receiver<FsSignal>,
        builder_signal_rx: Receiver<BuilderSignal>,
        k8s_signal_rx: Receiver<K8sSignal>,
    ) -> anyhow::Result<Self> {
        tracing::info!("before");
        let terminal = ratatui::try_init();

        tracing::error!("{:?}", terminal);
        let terminal = terminal?;

        tracing::info!("after");

        let state = AppState::new();
        let mut services = BTreeMap::default();

        for service in CoverDropService::all() {
            services.insert(*service, ServiceStatus::default());
        }

        Ok(Self {
            fs_signal_rx,
            builder_signal_rx,
            k8s_signal_rx,
            terminal,
            state,
            services,
        })
    }

    fn get_service_or_warn(&mut self, service: &CoverDropService) -> Option<&mut ServiceStatus> {
        let maybe_service = self.services.get_mut(service);

        if maybe_service.is_none() {
            tracing::warn!("Got message about a service that we're not tracking in UI!");
        }

        maybe_service
    }

    pub fn set_up_terminal(&mut self) -> anyhow::Result<()> {
        self.terminal.clear()?;
        Ok(())
    }

    pub fn restore_terminal() {
        ratatui::restore();
    }

    pub async fn start(&mut self) {
        let mut interval = interval(Duration::from_millis(16));
        let mut reader = crossterm::event::EventStream::new();

        loop {
            if self.state.quit {
                tracing::info!("Quit flag set, closing UI");
                break;
            }

            let crossterm_event = reader.next().fuse();

            tokio::select! {
                _ = interval.tick() => {
                    if let Err(e) = self.render() {
                        // Not sure how we can meaningfully recover from an unhandled error in the render loop.
                        panic!("Error in render loop: {:?}", e);
                    }
                }
                Some(Ok(event)) = crossterm_event => {
                    if let Event::Key(key) = event {
                        if key.kind == KeyEventKind::Press {
                            match &key.code {
                                KeyCode::Char('q') => self.state.quit = true,
                                KeyCode::Up => self.state.services_list_state.select_previous(),
                                KeyCode::Down => self.state.services_list_state.select_next(),
                                _ => {}
                            }
                        }
                    }
                },
                Ok(fs_signal) = self.fs_signal_rx.recv() => {
                    match fs_signal {
                        FsSignal::Dirty(service) => {
                            if let Some(status) = self.get_service_or_warn(&service) {
                                status.dirty = true;
                            }                        },
                    }
                }
                Ok(builder_signal) = self.builder_signal_rx.recv() => {
                    match builder_signal {
                        BuilderSignal::Begin(service) => {
                            if let Some(service_status) = self.get_service_or_warn(&service) {
                                service_status.build_log.clear();
                            };
                        },
                        BuilderSignal::Status(service, build_status) => {
                            if let Some(service_status) = self.get_service_or_warn(&service) {
                                service_status.build_log.push("");
                                service_status.build_log.push("#");
                                service_status.build_log.push(format!("# {}", build_status));
                                service_status.build_log.push("#");
                                service_status.build_log.push("");
                                service_status.build_status = build_status;
                            };
                        },
                        BuilderSignal::Success(service) => {
                            if let Some(service_status) = self.get_service_or_warn(&service) {
                                service_status.dirty = false;
                            };
                        },
                        BuilderSignal::Failed(service) => {
                            if let Some(service_status) = self.get_service_or_warn(&service) {
                                service_status.dirty = true;
                            };
                        },
                        BuilderSignal::LogLine(service, line) => {
                            if let Some(service_status) = self.get_service_or_warn(&service) {
                                service_status.build_log.push(line);
                            };
                        },
                    }
                }
                Ok(k8s_signal) = self.k8s_signal_rx.recv() => {
                    match k8s_signal {
                        K8sSignal::PodsStatus(service, pods ) => {
                            if let Some(service_status) = self.get_service_or_warn(&service) {
                                service_status.pods = pods;
                            }
                        },
                    }
                }
            }
        }
    }

    fn render(&mut self) -> anyhow::Result<()> {
        let now = Instant::now();
        let frame_time = Instant::now() - self.state.frame_start;
        self.state.frame_start = now;

        self.state.elapsed += frame_time.as_secs_f64();

        self.terminal.draw(|frame| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(frame.area());

            let top_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(layout[0]);

            App::render_services_panel(
                frame,
                top_layout[0],
                self.state.elapsed,
                &mut self.state.services_list_state,
                &self.services,
            );

            App::render_log_view(
                frame,
                top_layout[1],
                &self.state.services_list_state,
                &self.services,
                &mut self.state.scroll_bar_state,
            );

            frame.render_widget(
                Paragraph::new("Panel 3")
                    .block(Block::default().borders(ratatui::widgets::Borders::ALL)),
                layout[1],
            );
        })?;

        Ok(())
    }

    const SPINNER: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

    fn render_services_panel(
        frame: &mut Frame<'_>,
        area: Rect,
        elapsed: f64,
        services_list_state: &mut ListState,
        services: &BTreeMap<CoverDropService, ServiceStatus>,
    ) {
        let items: Vec<ratatui::widgets::ListItem> = services
            .iter()
            .flat_map(|(service, status)| {
                let is_building = status.build_status != BuildStatus::Idle;

                let spinner = if is_building {
                    Self::SPINNER[(4.0 * elapsed) as usize % Self::SPINNER.len()]
                } else {
                    ' '
                };

                let service_item = ListItem::new(format!(
                    "{} {} {} {}",
                    service.as_str(),
                    if status.dirty { "[DIRTY]" } else { "" },
                    spinner,
                    status.build_status
                ));

                let pod_items: Vec<ListItem> = status
                    .pods
                    .iter()
                    .map(|pod| {
                        let status_message = if pod.is_being_deleted {
                            "Terminating"
                        } else {
                            &pod.phase
                        };

                        ListItem::new(format!("  - {} [{}]", pod.name, status_message))
                    })
                    .collect();

                let mut items = vec![service_item];
                items.extend(pod_items);

                items
            })
            .collect();

        let list = List::new(items)
            .block(Block::bordered().title("Services"))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ")
            .repeat_highlight_symbol(true);

        frame.render_stateful_widget(list, area, services_list_state);
    }

    fn render_log_view(
        frame: &mut Frame<'_>,
        area: Rect,
        services_list_state: &ListState,
        services: &BTreeMap<CoverDropService, ServiceStatus>,
        scroll_bar_state: &mut ScrollbarState,
    ) {
        enum LogTarget {
            Service(CoverDropService),
            Pod(String),
        }

        let Some(selected_index) = services_list_state.selected() else {
            // Nothing selected don't render the logs
            return;
        };

        let selected_entity_name = services
            .iter()
            .flat_map(|(key, value)| {
                let service = iter::once(LogTarget::Service(*key));

                service.chain(
                    value
                        .pods
                        .iter()
                        .map(|pod| LogTarget::Pod(pod.name.clone())),
                )
            })
            .nth(selected_index);

        let mut line_count: u16 = 0;
        let log_content = match selected_entity_name {
            Some(LogTarget::Service(service)) => {
                if let Some(status) = services.get(&service) {
                    status
                        .build_log
                        .iter()
                        .fold(String::new(), |mut acc, line| {
                            line_count += 1;
                            acc.push_str(&format!("{:0>4}: ", line_count));
                            acc.push_str(line);
                            acc.push('\n');
                            acc
                        })
                } else {
                    format!(
                        "Service '{}' not being tracked in UI state (this is a bug)",
                        service.as_str()
                    )
                }
            }
            Some(LogTarget::Pod(name)) => format!("Logs for {}", name),
            None => "No entity selected".to_string(),
        };

        let log_paragraph = Paragraph::new(log_content).block(
            Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("Logs"),
        );

        // Todo move this up to app state

        let visible_lines = area.height;

        let scroll_position = if line_count > visible_lines {
            line_count - visible_lines + 1
        } else {
            0
        };

        *scroll_bar_state = scroll_bar_state
            .position(scroll_position as usize)
            .content_length(line_count as usize);

        let log_paragraph = log_paragraph.scroll((scroll_position, 0));

        frame.render_widget(log_paragraph, area);

        frame.render_stateful_widget(
            Scrollbar::default().orientation(ScrollbarOrientation::VerticalRight),
            area,
            scroll_bar_state,
        );
    }
}
