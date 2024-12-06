use tokio::process::Command;
use std::net::Ipv4Addr;
use color_eyre::eyre::{eyre, Result};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    if !cfg!(target_os = "windows") {
        return Err(eyre!("This program only runs on Windows."));
    }

    match get_local_ip() {
        Some(local_ip) => {
            println!("Scanning LAN from IP: {}", local_ip);

            let subnet = get_subnet(&local_ip);
            let tasks: Vec<_> = (1..=254)
                .map(|i| {
                    let ip = format!("{}.{}.{}.{}", subnet.0, subnet.1, subnet.2, i);
                    tokio::spawn(async move {
                        if let Some(rtt) = ping_rtt(&ip).await {
                            println!("Active IP: {}, RTT: {:.1} ms", ip, rtt);
                        }
                    })
                })
                .collect();

            // 全タスクの完了を待機
            futures::future::join_all(tasks).await;
        }
        None => println!("Could not determine local IP address. Exiting..."),
    }

    Ok(())
}

fn get_local_ip() -> Option<Ipv4Addr> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    if let std::net::SocketAddr::V4(addr) = socket.local_addr().ok()? {
        Some(addr.ip().clone())
    } else {
        None
    }
}

fn get_subnet(local_ip: &Ipv4Addr) -> (u8, u8, u8) {
    let [a, b, c, _] = local_ip.octets();
    (a, b, c)
}

async fn ping_rtt(ip: &str) -> Option<f64> {
    let start = Instant::now();
    if Command::new("ping")
        .args(["-n", "1", "-w", "100", ip]) // "-w" はタイムアウト設定（ms）
        .output()
        .await
        .map_or(false, |output| output.status.success())
    {
        Some(start.elapsed().as_secs_f64() * 1000.0) // RTTをmsに変換
    } else {
        None
    }
}
