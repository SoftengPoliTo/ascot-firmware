//! The firmware can be discovered in the local network, which also represents
//! the trusted network, through the `mDNS` protocol.

use core::net::Ipv4Addr;

use heapless::FnvIndexMap;

use crate::error::Result;
use crate::server::DEFAULT_SERVER_PORT;

// Maximum stack elements.
const MAXIMUM_ELEMENTS: usize = 16;

/// A service builder.
#[derive(Debug)]
pub struct ServiceBuilder {
    // Instance name.
    pub(crate) instance_name: &'static str,
    // Service port.
    pub(crate) port: u16,
    // Service properties.
    pub(crate) properties: FnvIndexMap<String, String, MAXIMUM_ELEMENTS>,
    // Add hostname
    pub(crate) hostname: &'static str,
    // Disable Ipv6.
    pub(crate) disable_ipv6: bool,
    // Disable docker network.
    pub(crate) disable_docker: bool,
}

impl ServiceBuilder {
    /// Creates a new [`ServiceBuilder`] for a `mDNS-SD` service.
    pub fn mdns_sd(instance_name: &'static str) -> Self {
        Self {
            instance_name,
            port: DEFAULT_SERVER_PORT,
            properties: FnvIndexMap::new(),
            hostname: instance_name,
            disable_ipv6: false,
            disable_docker: false,
        }
    }

    /// Sets a service property.
    pub fn property(mut self, property: (impl Into<String>, impl Into<String>)) -> Self {
        // If an equivalent key already exists in the map: the key remains and
        // retains in its place in the order.
        // Its corresponding value is updated with value and the older value
        // is returned inside.
        let _ = self.properties.insert(property.0.into(), property.1.into());
        self
    }

    /// Adds an host name to the service.
    pub fn host_name(mut self, hostname: &'static str) -> Self {
        self.hostname = hostname;
        self
    }

    /// Disables IPv6 addresses.
    pub fn ipv6(mut self) -> Self {
        self.disable_ipv6 = true;
        self
    }

    /// Disables docker bridge.
    pub fn docker(mut self) -> Self {
        self.disable_docker = true;
        self
    }

    // Sets service port.
    pub(crate) fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
}

// A new service.
pub(crate) struct Service;

impl Service {
    // Runs a service.
    #[inline]
    pub(crate) fn run(service_builder: ServiceBuilder, http_address: Ipv4Addr) -> Result<()> {
        #[cfg(feature = "mdns-sd-service")]
        crate::services::mdns_sd::run(service_builder, http_address)
    }
}
