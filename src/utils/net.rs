use local_ip_address::list_afinet_netifas;
use std::net::IpAddr;

/// Detects the machine's private local IP address (e.g., 192.168.x.x).
/// It prioritizes non-loopback IPv4 addresses.
pub fn get_local_ip() -> String {
    if let Ok(interfaces) = list_afinet_netifas() {
        for (_name, ip) in interfaces {
            if let IpAddr::V4(ipv4) = ip {
                if !ipv4.is_loopback() {
                    return ipv4.to_string();
                }
            }
        }
    }
    // Fallback if no network is found
    "127.0.0.1".to_string()
}

/// Constructs the canonical connection URL.
pub fn build_connection_url(
    tls: bool,
    ip: &str,
    port: u16,
    username: Option<&str>,
    password: Option<&str>,
    include_credentials: bool,
) -> String {
    let protocol = if tls { "https" } else { "http" };
    
    let auth_part = if include_credentials {
        match (username, password) {
            (Some(u), Some(p)) => format!("{}:{}@", u, p),
            _ => String::new(),
        }
    } else {
        String::new()
    };

    format!("{}://{}{}:{}", protocol, auth_part, ip, port)
}
