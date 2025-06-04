use common::tracing::{log_task_exit, log_task_result_exit};
use std::path::Path;

use crate::{
    dev::{
        build::cargo::cargo_metadata,
        watch::{builder::Builder, fs::FsWatcher, k8s::K8sWatcher, ui::App},
    },
    kube_client::KubeClient,
};

pub async fn watch(kubeconfig_path: impl AsRef<Path>) -> anyhow::Result<()> {
    let kube_client = KubeClient::new(&Some(&kubeconfig_path)).await?;

    let cargo_metadata = cargo_metadata().await?;

    tracing::info!("Setting up filesystem watcher");
    let mut fs_watcher = FsWatcher::new(&cargo_metadata)?;

    tracing::info!("Setting up builder");
    let mut builder = Builder::new(kube_client, fs_watcher.subscribe())?;

    tracing::info!("Setting up kubernetes watcher");
    let mut k8s_watcher = K8sWatcher::new();

    tracing::info!("Setting up UI");
    let mut ui = App::new(
        fs_watcher.subscribe(),
        builder.subscribe(),
        k8s_watcher.subscribe(),
    )?;

    tracing::info!("Setting up terminal");
    ui.set_up_terminal()?;

    // The fs watcher uses the `notify` crate which manages it's own thread pool
    // so we don't need to wrap this in a task. This watch command simply tells
    // that thread pool to start monitoring the crate paths
    fs_watcher.watch()?;

    let mut k8s_task = tokio::task::spawn({
        let kubeconfig_path = kubeconfig_path.as_ref().to_path_buf();
        async move { k8s_watcher.watch(kubeconfig_path).await }
    });
    let mut builder_task = tokio::task::spawn(async move { builder.start().await });
    let mut ui_task = tokio::task::spawn(async move { ui.start().await });

    tracing::info!("Blocking on tasks");
    tokio::select! {
        r = (&mut builder_task) => {
            log_task_exit("builder", r);

            k8s_task.abort();
            ui_task.abort();
        }
        r = (&mut k8s_task) => {
            log_task_result_exit("kubernetes", r);

            builder_task.abort();
            ui_task.abort();
        }
        r = (&mut ui_task) => {
            log_task_exit("ui", r);

            k8s_task.abort();
            builder_task.abort();
        }
    }

    tracing::info!("Task exited");

    App::restore_terminal();

    Ok(())
}
