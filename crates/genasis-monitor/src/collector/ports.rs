//! TCP port probe — detect running dev servers per agent role.

use std::collections::HashMap;
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

/// Probe a set of ports and return which are listening.
pub fn probe_ports(ports: &HashMap<String, u16>) -> HashMap<String, bool> {
    let mut results = HashMap::new();
    let timeout = Duration::from_secs(1);

    for (role, port) in ports {
        let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
        let is_up = TcpStream::connect_timeout(&addr, timeout).is_ok();
        results.insert(role.clone(), is_up);
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_nonexistent_port_returns_false() {
        let mut ports = HashMap::new();
        ports.insert("test".into(), 59999); // unlikely to be in use
        let results = probe_ports(&ports);
        assert_eq!(results.get("test"), Some(&false));
    }
}
