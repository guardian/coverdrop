use std::net::{Ipv4Addr, TcpListener, TcpStream};

pub async fn port_in_use(port: u16) -> bool {
    TcpListener::bind((Ipv4Addr::LOCALHOST, port)).is_err()
}

pub async fn port_available(port: u16) -> bool {
    TcpStream::connect((Ipv4Addr::LOCALHOST, port)).is_ok()
}

pub async fn wait_for_port_active(port: u16, silent: bool) {
    let mut attempts = 0;
    while attempts < 10 && !port_available(port).await {
        if !silent {
            println!("Port {} not active, waiting...", port);
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        attempts += 1;
    }
}
