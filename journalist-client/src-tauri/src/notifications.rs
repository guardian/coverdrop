use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt as _;
use tokio::sync::mpsc::{channel, Sender};

pub struct NotificationRequest {
    maybe_title: Option<String>,
    body: String,
}

#[derive(Clone)]
pub struct Notifications(Sender<NotificationRequest>);

impl Notifications {
    pub async fn send_with_default_title(&self, body: impl Into<String>) {
        self.send(None, body).await
    }

    /// Request a notification, if it fails to send then we log an error.
    /// Should only be used for non-critical notifications.
    pub async fn send(&self, maybe_title: Option<String>, body: impl Into<String>) {
        if let Err(e) = self.try_send(maybe_title, body).await {
            tracing::error!("Failed to send notification to task: {:?}", e);
        }
    }

    /// Request a notification, returning an error if the in-app notification service
    /// could not be sent the request. Note that this function being successful does not
    /// guarantee that the notification will appear since the handling of notifications by
    /// the OS is managed by a separate task.
    pub async fn try_send(
        &self,
        maybe_title: Option<String>,
        body: impl Into<String>,
    ) -> anyhow::Result<()> {
        self.0
            .send(NotificationRequest {
                maybe_title,
                body: body.into(),
            })
            .await?;

        Ok(())
    }
}

pub fn start_notification_service(app_handle: &AppHandle, default_title: String) -> Notifications {
    let (tx, mut rx) = channel::<NotificationRequest>(100);

    tauri::async_runtime::spawn({
        let app_handle = app_handle.clone();

        async move {
            while let Some(req) = rx.recv().await {
                if let Err(e) = app_handle
                    .notification()
                    .builder()
                    .title(req.maybe_title.unwrap_or(default_title.clone()))
                    .body(&req.body)
                    .show()
                {
                    tracing::error!("Failed to send desktop notification: {:?}", e);
                }
            }
        }
    });

    Notifications(tx)
}
