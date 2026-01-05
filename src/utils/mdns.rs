use mdns_sd::{ServiceDaemon, ServiceInfo};
use std::collections::HashMap;
use tokio::task::JoinHandle;

pub struct MdnsGuard(JoinHandle<()>);

impl Drop for MdnsGuard {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// Registers the HTTP service via mDNS in a background task.
/// Returns a guard that aborts the service when dropped.
pub fn register_service(
    port: u16,
    instance_name: &str,
    tls: bool,
) -> MdnsGuard {
    // Clone values to move into the async block
    let port = port;
    let name = instance_name.to_string();
    
    // Create the daemon
    let handle = tokio::spawn(async move {
        match ServiceDaemon::new() {
            Ok(mdns) => {
                let service_type = "_http._tcp.local.";
                let host_ipv4 = ""; // Let the library detect the IP
                let host_name = format!("{}.local.", name);
                
                let mut properties = HashMap::new();
                properties.insert("version".to_string(), "0.1.0".to_string());
                properties.insert("tls".to_string(), tls.to_string());

                match ServiceInfo::new(
                    service_type,
                    &name,
                    host_name.as_str(),
                    host_ipv4,
                    port,
                    properties,
                ) {
                    Ok(service_info) => {
                        if let Err(e) = mdns.register(service_info) {
                            tracing::error!("Failed to register mDNS service: {}", e);
                        } else {
                            tracing::info!("mDNS service registered: {}.{}", name, service_type);
                        }
                    },
                    Err(e) => {
                        tracing::error!("Invalid mDNS service info: {}", e);
                    }
                }

                // Keep the task alive to maintain the responder.
                // The daemon runs in its own threads but this task owns the daemon handle.
                // If we drop `mdns`, it might shut down depending on implementation (mdns-sd usually runs in background but we hold the controller).
                // We'll just wait indefinitely until this task is cancelled.
                std::future::pending::<()>().await;
            },
            Err(e) => {
                tracing::error!("Failed to create mDNS daemon: {}", e);
            }
        }
    });
    
    MdnsGuard(handle)
}
