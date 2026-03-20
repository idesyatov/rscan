use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::time::{timeout, sleep};

/// Result of scanning a single port.
#[derive(Debug, Clone)]
pub struct PortResult {
    pub ip: Ipv4Addr,
    pub port: u16,
    pub open: bool,
    pub banner: Option<String>,
}

/// Scan configuration.
#[derive(Debug, Clone)]
pub struct ScanConfig {
    pub conn_timeout: Duration,
    pub concurrency: usize,
    pub verbose: bool,
    pub grab_banners: bool,
    pub retries: u32,
    pub rate_limit: Option<u32>, // max connections per second, None = unlimited
}

/// TCP ping: check if host is alive by trying to connect to common ports (80, 443).
pub async fn ping_host(ip: Ipv4Addr, timeout_dur: Duration) -> bool {
    for port in [80, 443, 22] {
        let addr = SocketAddr::new(ip.into(), port);
        if timeout(timeout_dur, TcpStream::connect(addr))
            .await
            .map(|r| r.is_ok())
            .unwrap_or(false)
        {
            return true;
        }
    }
    false
}

/// Discover alive hosts from a list using TCP ping.
pub async fn discover_hosts(hosts: &[Ipv4Addr], timeout_dur: Duration, concurrency: usize, verbose: bool) -> Vec<Ipv4Addr> {
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let total = hosts.len();
    let checked = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let mut handles = Vec::with_capacity(total);

    for &ip in hosts {
        let sem = semaphore.clone();
        let checked = checked.clone();

        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let alive = ping_host(ip, timeout_dur).await;

            if verbose {
                let done = checked.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                eprint!("\rHost discovery: {}/{} ", done, total);
            }

            if alive { Some(ip) } else { None }
        }));
    }

    let mut alive_hosts = Vec::new();
    for handle in handles {
        if let Ok(Some(ip)) = handle.await {
            alive_hosts.push(ip);
        }
    }

    if verbose {
        eprintln!("\nDiscovered {} alive host(s)", alive_hosts.len());
    }

    alive_hosts
}

/// Scan a single port with retries.
async fn scan_port(ip: Ipv4Addr, port: u16, conn_timeout: Duration, grab_banner: bool, retries: u32) -> (bool, Option<String>) {
    for attempt in 0..=retries {
        let addr = SocketAddr::new(ip.into(), port);

        let stream = match timeout(conn_timeout, TcpStream::connect(addr)).await {
            Ok(Ok(s)) => s,
            _ => {
                if attempt < retries {
                    continue;
                }
                return (false, None);
            }
        };

        if !grab_banner {
            return (true, None);
        }

        let banner = grab_banner_from_stream(stream, conn_timeout).await;
        return (true, banner);
    }

    (false, None)
}

/// Try to read a banner from an open TCP connection.
async fn grab_banner_from_stream(mut stream: TcpStream, read_timeout: Duration) -> Option<String> {
    let mut buf = vec![0u8; 1024];

    match timeout(read_timeout, stream.read(&mut buf)).await {
        Ok(Ok(n)) if n > 0 => {
            let raw = &buf[..n];
            let text: String = raw.iter()
                .take_while(|&&b| b != b'\n' && b != b'\r')
                .filter(|&&b| b.is_ascii_graphic() || b == b' ')
                .map(|&b| b as char)
                .collect();

            if text.is_empty() {
                None
            } else {
                Some(if text.len() > 80 {
                    format!("{}...", &text[..77])
                } else {
                    text
                })
            }
        }
        _ => None,
    }
}

/// Scan multiple ports on multiple hosts concurrently.
pub async fn scan(
    hosts: &[Ipv4Addr],
    ports: &[u16],
    config: &ScanConfig,
) -> Vec<PortResult> {
    let semaphore = Arc::new(Semaphore::new(config.concurrency));
    let total = hosts.len() * ports.len();
    let scanned = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    // Rate limiter: interval between connection launches
    let rate_interval = config.rate_limit.map(|r| {
        if r == 0 { Duration::from_secs(0) } else { Duration::from_secs_f64(1.0 / r as f64) }
    });

    let mut handles = Vec::with_capacity(total);

    for &ip in hosts {
        for &port in ports {
            let sem = semaphore.clone();
            let scanned = scanned.clone();
            let conn_timeout = config.conn_timeout;
            let grab_banners = config.grab_banners;
            let verbose = config.verbose;
            let retries = config.retries;

            // Rate limiting: sleep before spawning
            if let Some(interval) = rate_interval {
                if !interval.is_zero() {
                    sleep(interval).await;
                }
            }

            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                let (open, banner) = scan_port(ip, port, conn_timeout, grab_banners, retries).await;

                if verbose {
                    let done = scanned.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                    eprint!("\rScanning: {}/{} ", done, total);
                }

                PortResult { ip, port, open, banner }
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

    if config.verbose {
        eprintln!();
    }

    results.sort_by(|a, b| a.ip.cmp(&b.ip).then(a.port.cmp(&b.port)));
    results
}
