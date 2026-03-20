/// Lookup well-known service name by port number.
pub fn lookup(port: u16) -> Option<&'static str> {
    match port {
        1 => Some("tcpmux"),
        7 => Some("echo"),
        9 => Some("discard"),
        11 => Some("systat"),
        13 => Some("daytime"),
        20 => Some("ftp-data"),
        21 => Some("ftp"),
        22 => Some("ssh"),
        23 => Some("telnet"),
        25 => Some("smtp"),
        37 => Some("time"),
        43 => Some("whois"),
        49 => Some("tacacs"),
        53 => Some("dns"),
        67 => Some("dhcp-server"),
        68 => Some("dhcp-client"),
        69 => Some("tftp"),
        70 => Some("gopher"),
        79 => Some("finger"),
        80 => Some("http"),
        88 => Some("kerberos"),
        102 => Some("iso-tsap"),
        110 => Some("pop3"),
        111 => Some("rpcbind"),
        113 => Some("ident"),
        119 => Some("nntp"),
        123 => Some("ntp"),
        135 => Some("msrpc"),
        137 => Some("netbios-ns"),
        138 => Some("netbios-dgm"),
        139 => Some("netbios-ssn"),
        143 => Some("imap"),
        161 => Some("snmp"),
        162 => Some("snmp-trap"),
        179 => Some("bgp"),
        194 => Some("irc"),
        389 => Some("ldap"),
        443 => Some("https"),
        445 => Some("microsoft-ds"),
        464 => Some("kpasswd"),
        465 => Some("smtps"),
        500 => Some("isakmp"),
        514 => Some("syslog"),
        515 => Some("printer"),
        520 => Some("rip"),
        523 => Some("ibm-db2"),
        530 => Some("rpc"),
        543 => Some("klogin"),
        544 => Some("kshell"),
        548 => Some("afp"),
        554 => Some("rtsp"),
        587 => Some("submission"),
        593 => Some("http-rpc"),
        631 => Some("ipp"),
        636 => Some("ldaps"),
        873 => Some("rsync"),
        902 => Some("vmware"),
        993 => Some("imaps"),
        995 => Some("pop3s"),
        1080 => Some("socks"),
        1099 => Some("rmiregistry"),
        1433 => Some("mssql"),
        1434 => Some("mssql-m"),
        1521 => Some("oracle"),
        1723 => Some("pptp"),
        1883 => Some("mqtt"),
        2049 => Some("nfs"),
        2082 => Some("cpanel"),
        2083 => Some("cpanel-ssl"),
        2181 => Some("zookeeper"),
        2222 => Some("ssh-alt"),
        2375 => Some("docker"),
        2376 => Some("docker-ssl"),
        3000 => Some("grafana"),
        3306 => Some("mysql"),
        3389 => Some("rdp"),
        3690 => Some("svn"),
        4369 => Some("epmd"),
        4443 => Some("https-alt"),
        5000 => Some("upnp"),
        5432 => Some("postgresql"),
        5672 => Some("amqp"),
        5900 => Some("vnc"),
        5984 => Some("couchdb"),
        6379 => Some("redis"),
        6443 => Some("kubernetes"),
        6660..=6669 => Some("irc"),
        7001 => Some("weblogic"),
        8000 => Some("http-alt"),
        8080 => Some("http-proxy"),
        8443 => Some("https-alt"),
        8888 => Some("http-alt"),
        9090 => Some("prometheus"),
        9092 => Some("kafka"),
        9200 => Some("elasticsearch"),
        9300 => Some("elasticsearch"),
        9418 => Some("git"),
        10000 => Some("webmin"),
        11211 => Some("memcached"),
        15672 => Some("rabbitmq-mgmt"),
        27017 => Some("mongodb"),
        27018 => Some("mongodb"),
        28017 => Some("mongodb-web"),
        50000 => Some("db2"),
        _ => None,
    }
}

/// Return the top N most commonly scanned ports (by frequency in real-world use).
/// Based on nmap's port frequency data.
pub fn top_ports(n: usize) -> Vec<u16> {
    let ports: Vec<u16> = vec![
        80, 23, 443, 21, 22, 25, 3389, 110, 445, 139,
        143, 53, 135, 3306, 8080, 1723, 111, 995, 993, 5900,
        1025, 587, 8888, 199, 1720, 465, 548, 113, 81, 6001,
        10000, 514, 5060, 179, 1026, 2000, 8443, 8000, 32768, 554,
        26, 1433, 49152, 2001, 515, 8008, 49154, 1027, 5666, 646,
        5000, 5631, 631, 49153, 8081, 2049, 88, 79, 5800, 106,
        2121, 1110, 49155, 6000, 513, 990, 5357, 427, 49156, 543,
        544, 5101, 144, 7, 389, 8009, 3128, 444, 9999, 5009,
        7070, 5190, 3000, 5432, 1900, 3986, 13, 1029, 9, 5051,
        6646, 49157, 1028, 873, 1755, 2717, 4899, 9100, 119, 37,
    ];
    ports.into_iter().take(n).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_known_ports() {
        assert_eq!(lookup(22), Some("ssh"));
        assert_eq!(lookup(80), Some("http"));
        assert_eq!(lookup(443), Some("https"));
        assert_eq!(lookup(3306), Some("mysql"));
        assert_eq!(lookup(6379), Some("redis"));
    }

    #[test]
    fn test_lookup_unknown_port() {
        assert_eq!(lookup(12345), None);
    }

    #[test]
    fn test_lookup_irc_range() {
        assert_eq!(lookup(6660), Some("irc"));
        assert_eq!(lookup(6667), Some("irc"));
        assert_eq!(lookup(6669), Some("irc"));
    }

    #[test]
    fn test_top_ports() {
        let top10 = top_ports(10);
        assert_eq!(top10.len(), 10);
        assert!(top10.contains(&80));
        assert!(top10.contains(&443));
        assert!(top10.contains(&22));
    }

    #[test]
    fn test_top_ports_overflow() {
        let all = top_ports(1000);
        assert_eq!(all.len(), 100); // max 100 in our list
    }
}
