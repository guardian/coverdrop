use std::{
    fmt::{self, Display},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    ops::Deref,
    sync::Arc,
    time::Duration as StdDuration,
};

use axum::{
    extract::{FromRef, Path, State},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use clap::ValueEnum;
use http::StatusCode;
use serde_json::json;
use thiserror::Error;
use tokio::{net::TcpListener, sync::RwLock, time::sleep};

use crate::time;

use super::Task;

pub const TASK_RUNNER_API_PORT: u16 = 4444;

type RwLockedTasks = Arc<RwLock<Vec<RunningTask>>>;

#[derive(Clone, FromRef)]
struct TaskRunnerState {
    tasks: RwLockedTasks,
}

impl TaskRunnerState {
    pub fn new(tasks: RwLockedTasks) -> Self {
        TaskRunnerState { tasks }
    }
}

struct RunningTask {
    pub next_scheduled_execution: DateTime<Utc>,
    pub inner: Box<dyn Task + Send + Sync>,
}

impl Deref for RunningTask {
    type Target = Box<dyn Task + Send + Sync>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq)]
pub enum RunnerMode {
    /// Will only execute tasks based on their defined interval. Normal operation.
    Timer,
    /// Will only allow execution via a web server. Useful in testing
    ManuallyTriggered,
    /// Executes tasks based on a timer as well as allowing manual execution.
    /// This is not often useful but might be helpful for debugging issues in
    /// a production-like environment where we don't want to interrupt normal operations
    /// but would like to be able to manually trigger some actions.
    TimerAndManuallyTriggered,
}

impl RunnerMode {
    pub fn triggerable(&self) -> bool {
        matches!(
            self,
            Self::ManuallyTriggered | Self::TimerAndManuallyTriggered
        )
    }

    pub fn timerbased(&self) -> bool {
        matches!(self, Self::Timer | Self::TimerAndManuallyTriggered)
    }
}

impl Display for RunnerMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

/// Run tasks, either on a schedule, or manually triggered over a
/// web interface or both.
pub struct TaskRunner {
    mode: RunnerMode,
    tasks: Arc<RwLock<Vec<RunningTask>>>,
}

impl TaskRunner {
    pub fn new(mode: RunnerMode) -> Self {
        Self {
            mode,
            tasks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn add_task(&self, task: impl Task + Send + Sync + 'static) -> &Self {
        tracing::debug!("Adding task '{}'", task.name());

        let mut tasks = self.tasks.write().await;

        tasks.push(RunningTask {
            // We want the task to initially execute immediately
            // Subsequent exceutions will be scheduled using `now() + task.interval()`
            next_scheduled_execution: time::now(),
            inner: Box::new(task),
        });

        self
    }

    pub async fn run(&mut self) {
        tracing::info!("Starting task runner in {} mode", self.mode);

        if self.mode.triggerable() {
            let task_runner_state = TaskRunnerState::new(self.tasks.clone());

            // Spawn and immediately drop a task handle, since this is only for
            // testing purposes we're not too fussed about handling a failure elegantly
            tokio::task::spawn(async move {
                let app = Router::new()
                    .route("/tasks", get(get_tasks))
                    .route("/tasks/{name}/trigger", post(post_task_trigger))
                    .with_state(task_runner_state);

                let socket_addr =
                    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), TASK_RUNNER_API_PORT);

                tracing::info!("Starting tasks server on http://{:?}", socket_addr);

                // Run the server, panicing if anything goes wrong. Since this is just for testing
                // recovery isn't that important
                let listener = TcpListener::bind(&socket_addr).await.expect("bind socket");

                axum::serve(listener, app)
                    .await
                    .expect("Task server to run")
            });
        }

        if self.mode.timerbased() {
            loop {
                // Open new scope so we don't sleep with the write lock open
                {
                    let mut tasks = self.tasks.write().await;
                    for task in tasks.iter_mut() {
                        let now = time::now();

                        if task.next_scheduled_execution < now {
                            tracing::info!("Running task: {}", task.name());

                            if let Err(e) = task.run().await {
                                tracing::error!("Failed to run task {}: {}", task.name(), e);
                            }

                            let task_interval = task.interval();

                            task.next_scheduled_execution = now + task_interval;
                        }
                    }
                }

                // This sleep duration is currently set to 1 second so that the task running works
                // nicely with time travel in the integration tests. The runner sleeps for 1 second and
                // then checks if any tasks need to run. In the future, once we migrate entirely to
                // using manually triggered tasks in the integration tests this sleep timer could be dynamic
                // to allow the runner to be less wasteful.
                sleep(StdDuration::from_secs(1)).await;
            }
        } else {
            // If we're only manually triggered, just sleep forever
            loop {
                sleep(StdDuration::from_secs(60)).await;
            }
        }
    }
}

#[derive(Debug, Error)]
enum TaskError {
    #[error("task not found")]
    NotFound,
    #[error("task execution failed")]
    ExecutionFailed,
}

impl IntoResponse for TaskError {
    fn into_response(self) -> Response {
        let (status, err_msg): (StatusCode, String) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "task not found".into()),
            Self::ExecutionFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "task execution failed".into(),
            ),
        };

        tracing::error!("Error from task service: {}", err_msg);

        let body = Json(json!({
            "error": err_msg,
        }));

        (status, body).into_response()
    }
}

async fn post_task_trigger(
    Path(name): Path<String>,
    State(tasks): State<RwLockedTasks>,
) -> Result<(), TaskError> {
    let mut tasks = tasks.write().await;

    let Some(task) = tasks.iter_mut().find(|task| task.name() == name) else {
        return Err(TaskError::NotFound);
    };

    tracing::info!("Manually triggered task: {}", task.name());

    if let Err(e) = task.run().await {
        tracing::error!("Failed to manually run task {}: {}", task.name(), e);
        return Err(TaskError::ExecutionFailed);
    }

    task.next_scheduled_execution = time::now() + task.interval();
    Ok(())
}

async fn get_tasks(State(tasks): State<RwLockedTasks>) -> Json<Vec<String>> {
    let tasks = tasks.read().await;

    let task_names = tasks.iter().map(|task| task.name().to_string()).collect();

    Json(task_names)
}
