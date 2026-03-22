mod scanner;
mod network;
mod output;
mod services;

use clap::Parser;
use std::time::{Duration, Instant};

/// rscan — CLI port scanner
#[derive(Parser, Debug)]
#[command(name = "rscan", version, about = "Fast CLI port scanner")]
struct Cli {
    /// Targets: IP addresses, CIDR subnets, or hostnames (multiple allowed)
    #[arg(required_unless_present = "target_file", help_heading = "Target")]
    targets: Vec<String>,

    /// Read targets from file (one per line; IPs, CIDRs, hostnames; # for comments)
    #[arg(short = 'i', long = "target-file", alias = "iL", help_heading = "Target")]
    target_file: Option<String>,

    /// Exclude hosts (comma-separated IPs or CIDRs)
    #[arg(long, help_heading = "Target")]
    exclude: Option<String>,

    /// Ports to scan: single (80), list (22,80,443), or range (1-1024)
    #[arg(short, long, default_value = "1-1024", help_heading = "Ports")]
    ports: String,

    /// Scan top N most common ports (overrides -p)
    #[arg(long, help_heading = "Ports")]
    top: Option<usize>,

    /// Connection timeout in milliseconds
    #[arg(short, long, default_value_t = 1000, help_heading = "Scan")]
    timeout: u64,

    /// Max concurrent connections
    #[arg(short = 'j', long = "threads", default_value_t = 100, help_heading = "Scan")]
    threads: usize,

    /// Grab service banners from open ports
    #[arg(short, long, help_heading = "Scan")]
    banner: bool,

    /// Ping check: discover alive hosts before scanning
    #[arg(long, help_heading = "Scan")]
    ping: bool,

    /// Retry count for timed-out ports
    #[arg(long, default_value_t = 0, help_heading = "Scan")]
    retry: u32,

    /// Max connections per second (0 = unlimited)
    #[arg(long, help_heading = "Scan")]
    rate: Option<u32>,

    /// Scan profile: fast, full, or stealth
    #[arg(long, help_heading = "Profile")]
    profile: Option<String>,

    /// [hidden] Alias for --profile fast
    #[arg(long, hide = true)]
    fast: bool,

    /// [hidden] Alias for --profile full
    #[arg(long, hide = true)]
    full: bool,

    /// Output results as JSON
    #[arg(long, help_heading = "Output")]
    json: bool,

    /// Save results to file (format by extension: .txt, .json, .csv)
    #[arg(short, long, help_heading = "Output")]
    output: Vec<String>,

    /// Verbose output
    #[arg(short, long, help_heading = "Output")]
    verbose: bool,
}

#[tokio::main]
async fn main() {
    let mut cli = Cli::parse();

    // Hidden aliases for backward compatibility
    if cli.fast { cli.profile = Some("fast".to_string()); }
    if cli.full { cli.profile = Some("full".to_string()); }

    // Apply profiles
    match cli.profile.as_deref() {
        Some("fast") => {
            if cli.top.is_none() { cli.top = Some(100); }
            if cli.timeout == 1000 { cli.timeout = 200; }
            if cli.threads == 100 { cli.threads = 200; }
        }
        Some("full") => {
            cli.ports = "1-65535".to_string();
            cli.top = None;
            if cli.timeout == 1000 { cli.timeout = 2000; }
        }
        Some("stealth") => {
            if cli.top.is_none() { cli.top = Some(20); }
            if cli.timeout == 1000 { cli.timeout = 3000; }
            if cli.threads == 100 { cli.threads = 10; }
            if cli.rate.is_none() { cli.rate = Some(10); }
        }
        Some(unknown) => {
            eprintln!("Error: unknown profile '{}'. Available: fast, full, stealth", unknown);
            std::process::exit(1);
        }
        None => {}
    }

    // Collect targets: from CLI args + from file
    let mut all_targets = cli.targets.clone();

    if let Some(ref path) = cli.target_file {
        match network::load_targets_from_file(path) {
            Ok(file_targets) => {
                if cli.verbose {
                    eprintln!("Loaded {} target(s) from {}", file_targets.len(), path);
                }
                all_targets.extend(file_targets);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }

    if all_targets.is_empty() {
        eprintln!("Error: no targets specified (use positional args or --target-file)");
        std::process::exit(1);
    }

    // Parse targets
    let mut hosts = match network::parse_targets(&all_targets) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    // Apply excludes
    if let Some(ref excludes) = cli.exclude {
        if let Err(e) = network::apply_excludes(&mut hosts, excludes) {
            eprintln!("Error in --exclude: {}", e);
            std::process::exit(1);
        }
        if hosts.is_empty() {
            eprintln!("Error: all hosts were excluded");
            std::process::exit(1);
        }
    }

    // Parse ports (--top overrides --ports)
    let ports = if let Some(n) = cli.top {
        if n == 0 {
            eprintln!("Error: --top must be at least 1");
            std::process::exit(1);
        }
        services::top_ports(n)
    } else {
        match network::parse_ports(&cli.ports) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    };

    // Verbose: show resolved info
    if cli.verbose {
        for target in &all_targets {
            if target.parse::<std::net::Ipv4Addr>().is_err() && !target.contains('/') {
                if let Ok(resolved) = network::parse_target(target) {
                    eprintln!("Resolved {} → {}", target, resolved.iter().map(|h| h.to_string()).collect::<Vec<_>>().join(", "));
                }
            }
        }
        let mut flags = Vec::new();
        if cli.banner { flags.push("banner"); }
        if cli.ping { flags.push("ping"); }
        if cli.retry > 0 { flags.push("retry"); }
        if cli.rate.is_some() { flags.push("rate-limit"); }
        if let Some(ref profile) = cli.profile {
            flags.push(match profile.as_str() {
                "fast" => "FAST",
                "full" => "FULL",
                "stealth" => "STEALTH",
                _ => "CUSTOM",
            });
        }

        eprintln!(
            "Scanning {} host(s), {} port(s), timeout {}ms, {} threads{}",
            hosts.len(),
            ports.len(),
            cli.timeout,
            cli.threads,
            if flags.is_empty() { String::new() } else { format!(" [{}]", flags.join(", ")) }
        );
    }

    // Host discovery (ping check)
    if cli.ping {
        let ping_timeout = Duration::from_millis(cli.timeout.min(500));
        if cli.verbose {
            eprintln!("Running host discovery...");
        }
        hosts = scanner::discover_hosts(&hosts, ping_timeout, cli.threads, cli.verbose).await;
        if hosts.is_empty() {
            eprintln!("No alive hosts found.");
            std::process::exit(0);
        }
    }

    // Build scan config
    let config = scanner::ScanConfig {
        conn_timeout: Duration::from_millis(cli.timeout),
        concurrency: cli.threads,
        verbose: cli.verbose,
        grab_banners: cli.banner,
        retries: cli.retry,
        rate_limit: cli.rate,
    };

    // Scan
    let start = Instant::now();
    let results = scanner::scan(&hosts, &ports, &config).await;
    let elapsed = start.elapsed();

    // Output to stdout
    if cli.json {
        output::print_json(&results);
    } else {
        output::print_text(&results, hosts.len(), ports.len(), elapsed);
    }

    // Save to files (format detected by extension)
    for path in &cli.output {
        let result = match output::detect_format(path) {
            "json" => output::save_json(&results, path),
            "csv" => output::save_csv(&results, path),
            _ => output::save_text(&results, hosts.len(), ports.len(), elapsed, path),
        };
        match result {
            Ok(()) => eprintln!("Results saved to {}", path),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
