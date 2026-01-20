use clap::Parser;

#[derive(Parser)]
#[command(name = "sentinel")]
#[command(about = "A desktop client for journalist users of the CoverDrop service")]
pub struct Cli {
    /// Prevent background tasks from starting when vault is unlocked
    #[arg(long)]
    pub no_background_tasks: bool,

    /// launch tauri instance (headless by default)
    #[arg(long)]
    pub launch_tauri_instance: bool,
}
