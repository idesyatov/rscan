use serde::Serialize;
use crate::scanner::PortResult;

#[derive(Serialize)]
struct JsonEntry {
    host: String,
    port: u16,
    state: String,
}

/// Print results as a human-readable table.
pub fn print_text(results: &[PortResult], hosts_count: usize, ports_count: usize) {
    let total_scanned = hosts_count * ports_count;

    println!();
    println!("Scan complete: {} ports scanned, {} open", total_scanned, results.len());
    println!();

    if results.is_empty() {
        println!("No open ports found.");
        return;
    }

    println!("{:<20} {:<10} {:<10}", "HOST", "PORT", "STATE");
    println!("{}", "-".repeat(40));
    for r in results {
        println!("{:<20} {:<10} {:<10}", r.ip, r.port, "open");
    }
}

/// Print results as JSON.
pub fn print_json(results: &[PortResult]) {
    let entries: Vec<JsonEntry> = results
        .iter()
        .map(|r| JsonEntry {
            host: r.ip.to_string(),
            port: r.port,
            state: "open".to_string(),
        })
        .collect();

    match serde_json::to_string_pretty(&entries) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error serializing JSON: {}", e),
    }
}
