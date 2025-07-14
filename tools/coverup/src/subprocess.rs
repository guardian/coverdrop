use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;
use tokio::sync::watch;

pub async fn wait_for_subprocess(mut child: Child, name: &str) -> anyhow::Result<Child> {
    // We want to make sure that the tunnel process gets killed if the rust process is exited.
    // Child doesn't implement Copy, so I had to do this signal thing in order to both .wait() on
    // the process and .kill() it if ctrl+c is pressed
    let (sigint_sender, mut sigint_receiver) = watch::channel(());

    ctrlc::set_handler(move || {
        let _ = sigint_sender.send(());
    })
    .expect("Set Ctrl-C handler");

    // wait on either the tunnel stopping (due to e.g. network outage), or ctrl+c
    tokio::select! {
        _ = sigint_receiver.changed() => {
            println!("Ctrl-C received, terminating {name} process");
            let _ = child.kill().await;
            Ok(child)
        }
        _ = child.wait() => {
            println!("{name} process exited");
            Ok(child)
        }
    }
}

pub async fn create_subprocess(
    name: &'static str,
    command: &str,
    with_logs: bool,
) -> anyhow::Result<Child> {
    let mut child = tokio::process::Command::new("sh")
        .kill_on_drop(true)
        .arg("-c")
        .arg(command)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    if with_logs {
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut stderr_reader = BufReader::new(stderr).lines();

        tokio::spawn({
            async move {
                while let Some(line) = stdout_reader.next_line().await.unwrap_or(None) {
                    println!("{name} stdout: {line}");
                }
            }
        });

        tokio::spawn({
            async move {
                while let Some(line) = stderr_reader.next_line().await.unwrap_or(None) {
                    eprintln!("{name} stderr: {line}");
                }
            }
        });
    }

    let child_id = child
        .id()
        .ok_or_else(|| anyhow::anyhow!("Failed to get child process id"))?;

    println!("{name} started. {name} process id: {child_id:?}");
    Ok(child)
}
