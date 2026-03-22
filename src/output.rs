use std::io::Write;
use std::time::Duration;
use colored::Colorize;
use serde::Serialize;
use crate::scanner::PortResult;
use crate::services;

#[derive(Serialize)]
struct JsonEntry {
    host: String,
    port: u16,
    state: String,
    service: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    banner: Option<String>,
}

/// Format results as a plain text table (no colors, for file output).
fn format_text_plain(results: &[PortResult], hosts_count: usize, ports_count: usize, elapsed: Duration) -> String {
    let total_scanned = hosts_count * ports_count;
    let mut out = String::new();

    out.push('\n');
    out.push_str(&format!(
        "Scan complete: {} ports scanned, {} open ({:.2}s)\n",
        total_scanned,
        results.len(),
        elapsed.as_secs_f64()
    ));
    out.push('\n');

    if results.is_empty() {
        out.push_str("No open ports found.\n");
        return out;
    }

    let has_banners = results.iter().any(|r| r.banner.is_some());

    if has_banners {
        out.push_str(&format!("{:<20} {:<8} {:<8} {:<18} {}\n", "HOST", "PORT", "STATE", "SERVICE", "BANNER"));
        out.push_str(&format!("{}\n", "-".repeat(80)));
        for r in results {
            let service = services::lookup(r.port).unwrap_or("-");
            let banner = r.banner.as_deref().unwrap_or("");
            out.push_str(&format!("{:<20} {:<8} {:<8} {:<18} {}\n", r.ip, r.port, "open", service, banner));
        }
    } else {
        out.push_str(&format!("{:<20} {:<8} {:<8} {}\n", "HOST", "PORT", "STATE", "SERVICE"));
        out.push_str(&format!("{}\n", "-".repeat(56)));
        for r in results {
            let service = services::lookup(r.port).unwrap_or("-");
            out.push_str(&format!("{:<20} {:<8} {:<8} {}\n", r.ip, r.port, "open", service));
        }
    }

    out
}

/// Print results as a colored table to stdout.
pub fn print_text(results: &[PortResult], hosts_count: usize, ports_count: usize, elapsed: Duration) {
    let total_scanned = hosts_count * ports_count;

    println!();
    println!(
        "Scan complete: {} ports scanned, {} open ({:.2}s)",
        total_scanned.to_string().bold(),
        results.len().to_string().green().bold(),
        elapsed.as_secs_f64()
    );
    println!();

    if results.is_empty() {
        println!("{}", "No open ports found.".dimmed());
        return;
    }

    let has_banners = results.iter().any(|r| r.banner.is_some());

    if has_banners {
        println!(
            "{:<20} {:<8} {:<8} {:<18} {}",
            "HOST".bold(), "PORT".bold(), "STATE".bold(), "SERVICE".bold(), "BANNER".bold()
        );
        println!("{}", "-".repeat(80).dimmed());
        for r in results {
            let service = services::lookup(r.port).unwrap_or("-");
            let banner = r.banner.as_deref().unwrap_or("");
            println!(
                "{:<20} {:<8} {:<8} {:<18} {}",
                r.ip.to_string().white(),
                r.port.to_string().cyan(),
                "open".green(),
                service.yellow(),
                banner.dimmed()
            );
        }
    } else {
        println!(
            "{:<20} {:<8} {:<8} {}",
            "HOST".bold(), "PORT".bold(), "STATE".bold(), "SERVICE".bold()
        );
        println!("{}", "-".repeat(56).dimmed());
        for r in results {
            let service = services::lookup(r.port).unwrap_or("-");
            println!(
                "{:<20} {:<8} {:<8} {}",
                r.ip.to_string().white(),
                r.port.to_string().cyan(),
                "open".green(),
                service.yellow()
            );
        }
    }
}

/// Format results as JSON string.
fn format_json(results: &[PortResult]) -> String {
    let entries: Vec<JsonEntry> = results
        .iter()
        .map(|r| JsonEntry {
            host: r.ip.to_string(),
            port: r.port,
            state: "open".to_string(),
            service: services::lookup(r.port).unwrap_or("unknown").to_string(),
            banner: r.banner.clone(),
        })
        .collect();

    serde_json::to_string_pretty(&entries).unwrap_or_else(|e| format!("Error serializing JSON: {}", e))
}

/// Format results as CSV string.
fn format_csv(results: &[PortResult]) -> String {
    let mut out = String::from("host,port,state,service,banner\n");
    for r in results {
        let service = services::lookup(r.port).unwrap_or("unknown");
        let banner = r.banner.as_deref().unwrap_or("").replace('"', "\"\"");
        out.push_str(&format!(
            "{},{},open,{},\"{}\"\n",
            r.ip, r.port, service, banner
        ));
    }
    out
}

/// Print results as JSON to stdout.
pub fn print_json(results: &[PortResult]) {
    println!("{}", format_json(results));
}

/// Save results to a text file (no colors).
pub fn save_text(results: &[PortResult], hosts_count: usize, ports_count: usize, elapsed: Duration, path: &str) -> Result<(), String> {
    let content = format_text_plain(results, hosts_count, ports_count, elapsed);
    write_file(path, &content)
}

/// Save results to a JSON file.
pub fn save_json(results: &[PortResult], path: &str) -> Result<(), String> {
    let content = format_json(results);
    write_file(path, &content)
}

/// Save results to a CSV file.
pub fn save_csv(results: &[PortResult], path: &str) -> Result<(), String> {
    let content = format_csv(results);
    write_file(path, &content)
}

/// Detect output format from file extension.
pub fn detect_format(path: &str) -> &'static str {
    if path.ends_with(".json") {
        "json"
    } else if path.ends_with(".csv") {
        "csv"
    } else {
        "text"
    }
}

fn write_file(path: &str, content: &str) -> Result<(), String> {
    let mut file = std::fs::File::create(path)
        .map_err(|e| format!("Failed to create file '{}': {}", path, e))?;
    file.write_all(content.as_bytes())
        .map_err(|e| format!("Failed to write to '{}': {}", path, e))?;
    Ok(())
}
