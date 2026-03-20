use std::net::{Ipv4Addr, ToSocketAddrs};

/// Parse multiple target strings into a deduplicated list of IPv4 addresses.
/// Each target can be: single IP, CIDR notation, or hostname.
pub fn parse_targets(targets: &[String]) -> Result<Vec<Ipv4Addr>, String> {
    let mut all_hosts = Vec::new();

    for target in targets {
        let hosts = parse_target(target)?;
        all_hosts.extend(hosts);
    }

    // Deduplicate while preserving order
    let mut seen = std::collections::HashSet::new();
    all_hosts.retain(|ip| seen.insert(*ip));

    if all_hosts.is_empty() {
        return Err("No valid hosts to scan".to_string());
    }

    Ok(all_hosts)
}

/// Parse a single target string into a list of IPv4 addresses.
/// Supports: single IP (192.168.1.1), CIDR notation (192.168.1.0/24), or hostname (google.com).
pub fn parse_target(target: &str) -> Result<Vec<Ipv4Addr>, String> {
    // CIDR notation
    if let Some((ip_str, prefix_str)) = target.split_once('/') {
        let ip: Ipv4Addr = ip_str
            .parse()
            .map_err(|_| format!("Invalid IP address: {}", ip_str))?;
        let prefix: u32 = prefix_str
            .parse()
            .map_err(|_| format!("Invalid CIDR prefix: {}", prefix_str))?;

        if prefix > 32 {
            return Err(format!("CIDR prefix must be 0-32, got: {}", prefix));
        }

        let ip_u32 = u32::from(ip);
        let mask = if prefix == 0 { 0 } else { !((1u32 << (32 - prefix)) - 1) };
        let network = ip_u32 & mask;
        let broadcast = network | !mask;

        if prefix >= 31 {
            let mut addrs = Vec::new();
            for addr in network..=broadcast {
                addrs.push(Ipv4Addr::from(addr));
            }
            Ok(addrs)
        } else {
            let mut addrs = Vec::new();
            for addr in (network + 1)..broadcast {
                addrs.push(Ipv4Addr::from(addr));
            }
            Ok(addrs)
        }
    }
    // Try as IP address first
    else if let Ok(ip) = target.parse::<Ipv4Addr>() {
        Ok(vec![ip])
    }
    // Try as hostname (DNS resolve)
    else {
        resolve_hostname(target)
    }
}

/// Resolve a hostname to a list of IPv4 addresses.
fn resolve_hostname(hostname: &str) -> Result<Vec<Ipv4Addr>, String> {
    let addr_str = format!("{}:0", hostname);
    let addrs: Vec<Ipv4Addr> = addr_str
        .to_socket_addrs()
        .map_err(|e| format!("Failed to resolve '{}': {}", hostname, e))?
        .filter_map(|sa| match sa.ip() {
            std::net::IpAddr::V4(ip) => Some(ip),
            _ => None,
        })
        .collect();

    if addrs.is_empty() {
        return Err(format!("No IPv4 addresses found for '{}'", hostname));
    }

    Ok(addrs)
}

/// Parse exclude list and remove those IPs from hosts.
pub fn apply_excludes(hosts: &mut Vec<Ipv4Addr>, excludes: &str) -> Result<(), String> {
    let mut excluded = std::collections::HashSet::new();

    for part in excludes.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        // Support CIDR in excludes too
        let addrs = parse_target(part)?;
        for addr in addrs {
            excluded.insert(addr);
        }
    }

    hosts.retain(|ip| !excluded.contains(ip));
    Ok(())
}

/// Parse port specification into a list of port numbers.
/// Supports: single (80), list (22,80,443), range (1-1024), or mixed (22,80,100-200).
pub fn parse_ports(ports_str: &str) -> Result<Vec<u16>, String> {
    let mut ports = Vec::new();

    for part in ports_str.split(',') {
        let part = part.trim();
        if let Some((start_str, end_str)) = part.split_once('-') {
            let start: u16 = start_str
                .trim()
                .parse()
                .map_err(|_| format!("Invalid port number: '{}'", start_str.trim()))?;
            let end: u16 = end_str
                .trim()
                .parse()
                .map_err(|_| format!("Invalid port number: '{}'", end_str.trim()))?;

            if start > end {
                return Err(format!("Invalid port range: {}-{} (start > end)", start, end));
            }
            if start == 0 {
                return Err("Port number must be 1-65535".to_string());
            }

            for port in start..=end {
                ports.push(port);
            }
        } else {
            let port: u16 = part
                .parse()
                .map_err(|_| format!("Invalid port number: '{}'", part))?;
            if port == 0 {
                return Err("Port number must be 1-65535".to_string());
            }
            ports.push(port);
        }
    }

    if ports.is_empty() {
        return Err("No ports specified".to_string());
    }

    ports.sort();
    ports.dedup();
    Ok(ports)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_ip() {
        let addrs = parse_target("192.168.1.1").unwrap();
        assert_eq!(addrs, vec![Ipv4Addr::new(192, 168, 1, 1)]);
    }

    #[test]
    fn test_parse_cidr_24() {
        let addrs = parse_target("192.168.1.0/24").unwrap();
        assert_eq!(addrs.len(), 254);
        assert_eq!(addrs[0], Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(addrs[253], Ipv4Addr::new(192, 168, 1, 254));
    }

    #[test]
    fn test_parse_cidr_30() {
        let addrs = parse_target("10.0.0.0/30").unwrap();
        assert_eq!(addrs.len(), 2);
    }

    #[test]
    fn test_parse_cidr_32() {
        let addrs = parse_target("10.0.0.5/32").unwrap();
        assert_eq!(addrs, vec![Ipv4Addr::new(10, 0, 0, 5)]);
    }

    #[test]
    fn test_parse_invalid_ip() {
        assert!(parse_target("999.999.999.999").is_err());
    }

    #[test]
    fn test_parse_invalid_cidr() {
        assert!(parse_target("192.168.1.0/33").is_err());
    }

    #[test]
    fn test_parse_hostname_localhost() {
        let addrs = parse_target("localhost").unwrap();
        assert!(addrs.contains(&Ipv4Addr::new(127, 0, 0, 1)));
    }

    #[test]
    fn test_parse_invalid_hostname() {
        assert!(parse_target("this.host.does.not.exist.invalid").is_err());
    }

    #[test]
    fn test_parse_multiple_targets() {
        let targets = vec!["192.168.1.1".to_string(), "10.0.0.1".to_string()];
        let addrs = parse_targets(&targets).unwrap();
        assert_eq!(addrs.len(), 2);
        assert!(addrs.contains(&Ipv4Addr::new(192, 168, 1, 1)));
        assert!(addrs.contains(&Ipv4Addr::new(10, 0, 0, 1)));
    }

    #[test]
    fn test_parse_multiple_targets_dedup() {
        let targets = vec!["192.168.1.1".to_string(), "192.168.1.1".to_string()];
        let addrs = parse_targets(&targets).unwrap();
        assert_eq!(addrs.len(), 1);
    }

    #[test]
    fn test_exclude_hosts() {
        let mut hosts = vec![
            Ipv4Addr::new(192, 168, 1, 1),
            Ipv4Addr::new(192, 168, 1, 2),
            Ipv4Addr::new(192, 168, 1, 3),
        ];
        apply_excludes(&mut hosts, "192.168.1.2").unwrap();
        assert_eq!(hosts.len(), 2);
        assert!(!hosts.contains(&Ipv4Addr::new(192, 168, 1, 2)));
    }

    #[test]
    fn test_exclude_cidr() {
        let mut hosts: Vec<Ipv4Addr> = (1..=10).map(|i| Ipv4Addr::new(10, 0, 0, i)).collect();
        apply_excludes(&mut hosts, "10.0.0.0/30").unwrap();
        // /30 excludes .1 and .2 (network .0 and broadcast .3 weren't in list)
        assert_eq!(hosts.len(), 8);
    }

    #[test]
    fn test_parse_single_port() {
        assert_eq!(parse_ports("80").unwrap(), vec![80]);
    }

    #[test]
    fn test_parse_port_list() {
        assert_eq!(parse_ports("22,80,443").unwrap(), vec![22, 80, 443]);
    }

    #[test]
    fn test_parse_port_range() {
        let ports = parse_ports("1-5").unwrap();
        assert_eq!(ports, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_parse_port_mixed() {
        let ports = parse_ports("22,80,100-103").unwrap();
        assert_eq!(ports, vec![22, 80, 100, 101, 102, 103]);
    }

    #[test]
    fn test_parse_port_dedup() {
        let ports = parse_ports("80,80,80").unwrap();
        assert_eq!(ports, vec![80]);
    }

    #[test]
    fn test_parse_port_zero() {
        assert!(parse_ports("0").is_err());
    }

    #[test]
    fn test_parse_port_invalid_range() {
        assert!(parse_ports("100-50").is_err());
    }
}
