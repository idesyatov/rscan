mod scanner;
mod network;
mod output;

use clap::Parser;
use std::time::Duration;

/// rscan — CLI port scanner
#[derive(Parser, Debug)]
#[command(name = "rscan", version, about = "Fast CLI port scanner")]
struct Cli {
    /// Target IP address or CIDR subnet (e.g. 192.168.1.1, 10.0.0.0/24)
    target: String,

    /// Ports to scan: single (80), list (22,80,443), or range (1-1024)
    #[arg(short, long, default_value = "1-1024")]
    ports: String,

    /// Connection timeout in milliseconds
    #[arg(short, long, default_value_t = 1000)]
    timeout: u64,

    /// Max concurrent connections
    #[arg(short = 'j', long = "threads", default_value_t = 100)]
    threads: usize,

    /// Output results as JSON
    #[arg(long)]
    json: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Parse target
    let hosts = match network::parse_target(&cli.target) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    // Parse ports
    let ports = match network::parse_ports(&cli.ports) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    if cli.verbose {
        eprintln!(
            "Scanning {} host(s), {} port(s), timeout {}ms, {} threads",
            hosts.len(),
            ports.len(),
            cli.timeout,
            cli.threads
        );
    }

    // Scan
    let timeout = Duration::from_millis(cli.timeout);
    let results = scanner::scan(&hosts, &ports, timeout, cli.threads, cli.verbose).await;

    // Output
    if cli.json {
        output::print_json(&results);
    } else {
        output::print_text(&results, hosts.len(), ports.len());
    }
}
