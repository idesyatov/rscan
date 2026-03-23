use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::time::{timeout, sleep};
use indicatif::{ProgressBar, ProgressStyle};

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

/// HTTP ports that need active probing.
fn is_http_port(port: u16) -> bool {
    matches!(port, 80 | 443 | 8000 | 8080 | 8443 | 8888 | 3000 | 5000 | 9090)
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

    let progress = if verbose {
        let pb = ProgressBar::new(total as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} Host discovery [{bar:30.cyan/dim}] {pos}/{len} [{elapsed_precise}<{eta_precise}]")
            .unwrap()
            .progress_chars("█░░"));
        Some(pb)
    } else {
        None
    };
    let progress = progress.map(Arc::new);

    let mut handles = Vec::with_capacity(total);

    for &ip in hosts {
        let sem = semaphore.clone();
        let pb = progress.clone();

        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let alive = ping_host(ip, timeout_dur).await;

            if let Some(ref pb) = pb {
                pb.inc(1);
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

    if let Some(pb) = progress {
        pb.finish_and_clear();
    }
    if verbose {
        eprintln!("Discovered {} alive host(s)", alive_hosts.len());
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

        let banner = if is_http_port(port) {
            grab_http_banner(stream, &ip.to_string(), conn_timeout).await
        } else {
            grab_banner_from_stream(stream, conn_timeout).await
        };
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

/// Send HTTP request and parse response for banner.
async fn grab_http_banner(mut stream: TcpStream, host: &str, read_timeout: Duration) -> Option<String> {
    let request = format!("GET / HTTP/1.0\r\nHost: {}\r\n\r\n", host);
    if stream.write_all(request.as_bytes()).await.is_err() {
        return None;
    }

    let mut buf = vec![0u8; 2048];
    match timeout(read_timeout, stream.read(&mut buf)).await {
        Ok(Ok(n)) if n > 0 => {
            let response = String::from_utf8_lossy(&buf[..n]);

            // Extract status line (first line)
            let status_line = response.lines().next().unwrap_or("").to_string();

            // Extract Server header
            let server = response.lines()
                .find(|line| line.to_lowercase().starts_with("server:"))
                .map(|line| line[7..].trim().to_string());

            if status_line.is_empty() {
                None
            } else if let Some(srv) = server {
                Some(format!("{} ({})", status_line, srv))
            } else {
                Some(status_line)
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
    // Rate limiter: interval between connection launches
    let rate_interval = config.rate_limit.map(|r| {
        if r == 0 { Duration::from_secs(0) } else { Duration::from_secs_f64(1.0 / r as f64) }
    });

    let progress = if config.verbose {
        let pb = ProgressBar::new(total as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} Scanning [{bar:30.cyan/dim}] {pos}/{len} [{elapsed_precise}<{eta_precise}] {per_sec}")
            .unwrap()
            .progress_chars("█░░"));
        Some(pb)
    } else {
        None
    };
    let progress = progress.map(Arc::new);

    let mut handles = Vec::with_capacity(total);

    for &ip in hosts {
        for &port in ports {
            let sem = semaphore.clone();
            let conn_timeout = config.conn_timeout;
            let grab_banners = config.grab_banners;
            let retries = config.retries;
            let pb = progress.clone();

            // Rate limiting: sleep before spawning
            if let Some(interval) = rate_interval {
                if !interval.is_zero() {
                    sleep(interval).await;
                }
            }

            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                let (open, banner) = scan_port(ip, port, conn_timeout, grab_banners, retries).await;

                if let Some(ref pb) = pb {
                    pb.inc(1);
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

    if let Some(pb) = progress {
        pb.finish_and_clear();
    }

    results.sort_by(|a, b| a.ip.cmp(&b.ip).then(a.port.cmp(&b.port)));
    results
}
