use std::{fs::File, path::Path, str::FromStr};

use app_state::AppStateHandle;
use clap::Parser as _;
use commands::{
    admin::{
        force_rotate_id_pk, force_rotate_msg_pk, get_logs, get_public_info,
        get_trust_anchor_digests, get_vault_keys,
    },
    backup::{get_backup_checks, perform_backup, should_require_backup},
    chats::{
        burst_cover_messages, check_message_length, get_chats, get_users, mark_as_read,
        mark_as_unread, set_custom_expiry, submit_message, update_user_alias_and_description,
        update_user_status,
    },
    profiles::get_profiles,
    vaults::{add_trust_anchor, get_colocated_password, get_vault_state, unlock_vault},
};
use logging::JournalistClientLogLayer;
use model::Profiles;
use notifications::start_notification_service;
use reqwest::Url;
use tauri::{App, Manager as _};
use tauri_plugin_dialog::{DialogExt as _, MessageDialogKind};
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

use cli::Cli;

use crate::commands::admin::update_journalist_status;

mod app_state;
mod cli;
mod commands;
mod error;
mod logging;
mod model;
mod multipass;
mod notifications;
mod tasks;

fn fail_setup_with_message(app: &mut App, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    app.dialog()
        .message(message)
        .kind(MessageDialogKind::Error)
        .title("Error")
        .blocking_show();

    Err(message.to_string().into())
}

fn handle_profiles(profiles_path: impl AsRef<Path>) -> anyhow::Result<Profiles> {
    let mut profiles = if profiles_path.as_ref().exists() {
        let profiles_file = File::open(profiles_path.as_ref())?;
        serde_json::from_reader::<File, Profiles>(profiles_file)?
    } else {
        Profiles::default()
    };

    // Update any existing profiles
    if let Some(profiles_env) = option_env!("BUILT_IN_PROFILES") {
        for profile_pair in profiles_env.split(',') {
            if let Some((stage, url)) = profile_pair.split_once('=') {
                let url = Url::from_str(url)?;
                profiles.insert(stage, url);
            }
        }
    }

    #[cfg(debug_assertions)]
    {
        if let Ok(multipass_nodes) = multipass::list_coverdrop_nodes() {
            if let Some(node) = multipass_nodes.first() {
                if let Some(local_ip) = node.local_ip() {
                    let url = format!("http://{local_ip}:30000/");
                    let url = Url::from_str(&url)?;
                    profiles.insert("DEV-AUTO", url);
                } else {
                    tracing::warn!(
                        "Unable to get IP address from multipass node in 192.168.0.0/16 subnet"
                    );
                }
            }
        } else {
            tracing::warn!("Unable to list multipass nodes, is multipass cli installed?");
        }
    }

    let json = serde_json::to_string_pretty(&profiles)?;
    std::fs::write(profiles_path.as_ref(), json)?;

    Ok(profiles)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let cli = Cli::parse();

    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            let config_dir = app.path().app_data_dir()?;

            let notifications = start_notification_service(app.app_handle());
            let app_state =
                AppStateHandle::new(app.handle().clone(), notifications, cli.no_background_tasks);

            tracing_subscriber::registry()
                .with(JournalistClientLogLayer::new(app_state.logs.clone()))
                .init();

            if let Err(e) = std::fs::create_dir_all(&config_dir) {
                return fail_setup_with_message(
                    app,
                    &format!("Failed to create application config directory: {e:?}"),
                );
            }

            let profiles_path = &config_dir.join("profiles.json");

            let profiles = match handle_profiles(profiles_path) {
                Ok(profiles) => profiles,
                Err(e) => {
                    return fail_setup_with_message(app, &format!("Failed to load profiles: {e:?}"))
                }
            };

            app.manage(app_state);
            app.manage(profiles);

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_vault_state,
            get_users,
            get_chats,
            unlock_vault,
            get_backup_checks,
            should_require_backup,
            perform_backup,
            get_colocated_password,
            get_profiles,
            submit_message,
            force_rotate_id_pk,
            force_rotate_msg_pk,
            get_public_info,
            update_journalist_status,
            check_message_length,
            mark_as_read,
            mark_as_unread,
            set_custom_expiry,
            update_user_status,
            update_user_alias_and_description,
            get_logs,
            burst_cover_messages,
            get_trust_anchor_digests,
            get_vault_keys,
            add_trust_anchor
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                window
                    .hide()
                    .expect("Could not hide main window, when CloseRequested");
            }
        })
        .build(tauri::generate_context!())
        .expect("Build tauri application")
        .run(
            #[cfg(target_os = "macos")] // the below Reopen event is only relevant on macOS
            |app_handle, event| {
                if let tauri::RunEvent::Reopen { .. } = event {
                    // Reopen is when the dock icon is clicked
                    app_handle
                        .get_webview_window("main")
                        .expect("Could not get main window on Reopen event")
                        .show()
                        .expect("Could not show main window on Reopen event");
                }
            },
            #[cfg(not(target_os = "macos"))]
            |_, _| {},
        );
}
