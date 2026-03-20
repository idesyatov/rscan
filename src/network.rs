use std::net::Ipv4Addr;

/// Parse target string into a list of IPv4 addresses.
/// Supports single IP (192.168.1.1) and CIDR notation (192.168.1.0/24).
pub fn parse_target(target: &str) -> Result<Vec<Ipv4Addr>, String> {
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
            // Exclude network and broadcast addresses
            let mut addrs = Vec::new();
            for addr in (network + 1)..broadcast {
                addrs.push(Ipv4Addr::from(addr));
            }
            Ok(addrs)
        }
    } else {
        let ip: Ipv4Addr = target
            .parse()
            .map_err(|_| format!("Invalid target: '{}'. Expected IP or CIDR (e.g. 192.168.1.1, 10.0.0.0/24)", target))?;
        Ok(vec![ip])
    }
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
        assert_eq!(addrs[0], Ipv4Addr::new(10, 0, 0, 1));
        assert_eq!(addrs[1], Ipv4Addr::new(10, 0, 0, 2));
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
