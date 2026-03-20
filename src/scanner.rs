use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::time::timeout;

/// Result of scanning a single port.
#[derive(Debug, Clone)]
pub struct PortResult {
    pub ip: Ipv4Addr,
    pub port: u16,
    pub open: bool,
}

/// Scan a single port on a given IP address.
/// Returns true if the port is open (TCP connect succeeds within timeout).
async fn scan_port(ip: Ipv4Addr, port: u16, conn_timeout: Duration) -> bool {
    let addr = SocketAddr::new(ip.into(), port);
    timeout(conn_timeout, TcpStream::connect(addr))
        .await
        .map(|r| r.is_ok())
        .unwrap_or(false)
}

/// Scan multiple ports on multiple hosts concurrently.
/// Returns a list of results (only open ports unless show_all is true).
pub async fn scan(
    hosts: &[Ipv4Addr],
    ports: &[u16],
    conn_timeout: Duration,
    concurrency: usize,
    verbose: bool,
) -> Vec<PortResult> {
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let total = hosts.len() * ports.len();
    let scanned = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let mut handles = Vec::with_capacity(total);

    for &ip in hosts {
        for &port in ports {
            let sem = semaphore.clone();
            let scanned = scanned.clone();

            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                let open = scan_port(ip, port, conn_timeout).await;

                if verbose {
                    let done = scanned.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                    eprint!("\rScanning: {}/{} ", done, total);
                }

                PortResult { ip, port, open }
            }));
        }
    }

    let mut results = Vec::new();
    for handle in handles {
        if let Ok(result) = handle.await {
            if result.open {
                results.push(result);
            }
        }
    }

    if verbose {
        eprintln!();
    }

    // Sort by IP then port
    results.sort_by(|a, b| a.ip.cmp(&b.ip).then(a.port.cmp(&b.port)));
    results
}
