//! The firmware can be discovered in the local network, which also represents
//! the trusted network, through the `mDNS` protocol.

use std::collections::HashMap;
use std::net::IpAddr;

use mdns_sd::{ServiceDaemon, ServiceInfo};

use thiserror::Error;
use tracing::debug;

use crate::server::DEFAULT_SERVER_PORT;

// Service type
//
// It constitutes part of the mDNS domain.
// This also allows the firmware to be detected during the mDNS discovery phase.
const SERVICE_TYPE: &str = "_ascot";

// DNS type.
//
// It defines the mDNS type. In this case, the firmware is an `Ascot Device`.
const DNS_TYPE: &str = "Ascot Device";

/// All `[ServiceBuilder]` errors.
#[derive(Debug, Error)]
pub enum ServiceBuilderError {
    /// A mDNS error.
    #[error("mdns error {0}")]
    Mdns(#[from] mdns_sd::Error),
    /// Network error.
    #[error("I/O error {0}")]
    Io(#[from] std::io::Error),
}

/// A specialized `Result` type.
pub type Result<T> = std::result::Result<T, ServiceBuilderError>;

/// A `mDNS` service builder.
#[derive(Debug)]
pub struct ServiceBuilder {
    /// Instance name.
    name: &'static str,
    /// Service port.
    pub(crate) port: u16,
    /// Service properties.
    properties: HashMap<String, String>,
}

impl ServiceBuilder {
    /// Creates a new `ServiceBuilder` instance.
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            port: DEFAULT_SERVER_PORT,
            properties: HashMap::new(),
        }
    }

    /// Sets a service port.
    ///
    /// It must be the same of the server.
    pub const fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sets a service property.
    pub fn property(mut self, property: (impl Into<String>, impl Into<String>)) -> Self {
        self.properties.insert(property.0.into(), property.1.into());
        self
    }

    /// Builds the service.
    pub fn build(&mut self) -> Result<()> {
        // Create a new mDNS service daemon
        let mdns = ServiceDaemon::new()?;

        // Retrieve the hostname associated with the machine on which the firmware
        // is running on
        let mut hostname = gethostname::gethostname().to_string_lossy().to_string();
        // Add the .local domain as hostname suffix when not present.
        //
        // .local is a special domain name for hostnames in local area networks
        // which can be resolved via the Multicast DNS name resolution protocol.
        if !hostname.ends_with(".local") {
            hostname.push_str(".local");
        }

        debug!("Hostname: {hostname}");

        // Retrieve all listening network IPs
        //
        // Do not exclude loopback interfaces in order to allow the communication
        // among the processes on the same machine for testing purposes.
        //
        // Only IPv4 addresses are considered.
        let listening_ips = if_addrs::get_if_addrs()?
            .iter()
            .filter(|iface| !iface.is_loopback())
            .filter_map(|iface| {
                let ip = iface.ip();
                match ip {
                    IpAddr::V4(_) => Some(ip),
                    _ => None,
                }
            })
            .collect::<Vec<IpAddr>>();

        debug!("IPs: {:?}", listening_ips);

        // Firmware DNS type
        self.properties.insert("type".into(), DNS_TYPE.into());

        // Define mDNS domain
        let domain = format!("{SERVICE_TYPE}._tcp.local.");

        let service = ServiceInfo::new(
            // Domain label and service type
            &domain,
            // Instance name
            self.name,
            // DNS hostname.
            //
            // For the same hostname in the same local network, the service resolves
            // in the same addresses. It is used for A (IPv4) and AAAA (IPv6)
            // records.
            &hostname,
            // Considered IP addresses which allow to reach out the service
            listening_ips.as_slice(),
            // Port on which the service listens to. It has to be same of the
            // server.
            self.port,
            // Service properties
            self.properties.to_owned(),
        )?;

        mdns.register(service)?;

        Ok(())
    }
}
