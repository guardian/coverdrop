use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt as _;
use tokio::sync::mpsc::{channel, Sender};

pub struct NotificationRequest {
    title: String,
    body: String,
}

#[derive(Clone)]
pub struct Notifications(Sender<NotificationRequest>);

impl Notifications {
    pub async fn send_with_default_title(&self, body: impl Into<String>) {
        self.send("CoverDrop", body).await
    }

    /// Request a notification, if it fails to send then we log an error.
    /// Should only be used for non-critical notifications.
    pub async fn send(&self, title: impl Into<String>, body: impl Into<String>) {
        if let Err(e) = self.try_send(title, body).await {
            tracing::error!("Failed to send notification to task: {:?}", e);
        }
    }

    /// Request a notification, returning an error if the in-app notification service
    /// could not be sent the request. Note that this function being successful does not
    /// guarantee that the notification will appear since the handling of notifications by
    /// the OS is managed by a separate task.
    pub async fn try_send(
        &self,
        title: impl Into<String>,
        body: impl Into<String>,
    ) -> anyhow::Result<()> {
        self.0
            .send(NotificationRequest {
                title: title.into(),
                body: body.into(),
            })
            .await?;

        Ok(())
    }
}

pub fn start_notification_service(app_handle: &AppHandle) -> Notifications {
    let (tx, mut rx) = channel::<NotificationRequest>(100);

    tauri::async_runtime::spawn({
        let app_handle = app_handle.clone();

        async move {
            while let Some(req) = rx.recv().await {
                if let Err(e) = app_handle
                    .notification()
                    .builder()
                    .title(&req.title)
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
