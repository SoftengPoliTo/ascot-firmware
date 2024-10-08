use core::net::{IpAddr, Ipv4Addr};

use std::collections::HashMap;

use mdns_sd::{IfKind, ServiceDaemon, ServiceInfo};

use tracing::info;

use crate::error::{Error, ErrorKind};
use crate::service::ServiceBuilder;

// Service type
//
// It constitutes part of the mDNS domain.
// This also allows the firmware to be detected during the mDNS discovery phase.
const SERVICE_TYPE: &str = "_ascot";

// DNS type.
//
// It defines the mDNS type. In this case, the firmware is an `Ascot Device`.
const DNS_TYPE: &str = "Ascot Device";

impl From<mdns_sd::Error> for Error {
    fn from(e: mdns_sd::Error) -> Self {
        Self::new(ErrorKind::Service, e.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::new(ErrorKind::NotFoundAddress, e.to_string())
    }
}

pub(crate) fn run(
    service: ServiceBuilder,
    http_address: Ipv4Addr,
) -> std::result::Result<(), Error> {
    // Create a new mDNS service daemon
    let mdns = ServiceDaemon::new()?;

    // Disable every IpV6 interface.
    if service.disable_ipv6 {
        mdns.disable_interface(IfKind::IPv6)?;
    }

    // Disable docker0 bridge.
    if service.disable_docker {
        mdns.disable_interface("docker0")?;
    }

    // Add the .local domain as hostname suffix when not present.
    //
    // .local is a special domain name for hostnames in local area networks
    // which can be resolved via the Multicast DNS name resolution protocol.
    let hostname = if !service.hostname.ends_with(".local.") {
        &format!("{}.local.", service.hostname)
    } else {
        service.hostname
    };

    // Allocates properties on heap
    let mut properties = service
        .properties
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect::<HashMap<_, _>>();

    // Firmware DNS type
    properties.insert("type".into(), DNS_TYPE.into());

    // Define mDNS domain
    let domain = format!("{SERVICE_TYPE}._tcp.local.");

    info!("Service instance name: {}", service.instance_name);
    info!("Service domain: {domain}");
    info!("Service port: {}", service.port);
    info!(
        "Device reachable at this hostname: {}:{}",
        &hostname[0..hostname.len() - 1],
        service.port
    );

    let service = ServiceInfo::new(
        // Domain label and service type
        &domain,
        // Service instance name
        service.instance_name,
        // DNS hostname.
        //
        // For the same hostname in the same local network, the service resolves
        // in the same addresses. It is used for A (IPv4) and AAAA (IPv6)
        // records.
        hostname,
        // Considered IP address which allow to reach out the service.
        IpAddr::V4(http_address),
        // Port on which the service listens to. It has to be same of the
        // server.
        service.port,
        // Service properties
        properties,
    )?;

    mdns.register(service)?;

    Ok(())
}
