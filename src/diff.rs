use crate::scanner::PortResult;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Deserialize)]
struct JsonEntry {
    host: String,
    port: u16,
    #[allow(dead_code)]
    state: String,
}

/// Load previous scan results from a JSON file.
pub fn load_baseline(path: &str) -> Result<HashSet<(String, u16)>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read baseline '{}': {}", path, e))?;
    let entries: Vec<JsonEntry> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse baseline JSON: {}", e))?;
    Ok(entries.into_iter().map(|e| (e.host, e.port)).collect())
}

/// Compare current results with baseline.
/// Returns (new_open, now_closed, unchanged_count)
pub fn compare<'a>(current: &'a [PortResult], baseline: &HashSet<(String, u16)>) -> (Vec<&'a PortResult>, Vec<(String, u16)>, usize) {
    let current_set: HashSet<(String, u16)> = current.iter()
        .map(|r| (r.ip.to_string(), r.port))
        .collect();

    let new_open: Vec<&PortResult> = current.iter()
        .filter(|r| !baseline.contains(&(r.ip.to_string(), r.port)))
        .collect();

    let now_closed: Vec<(String, u16)> = baseline.iter()
        .filter(|entry| !current_set.contains(entry))
        .cloned()
        .collect();

    let unchanged = current.len() - new_open.len();

    (new_open, now_closed, unchanged)
}
