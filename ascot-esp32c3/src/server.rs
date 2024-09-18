use core::net::Ipv4Addr;

use std::thread::sleep;
use std::time::Duration;

use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_svc::http::Method;

use ascot_library::route::RestKind;

use crate::device::Device;
use crate::service::MdnsSdService;

// Default port.
const DEFAULT_SERVER_PORT: u16 = 3000;

// Default scheme is `http`.
const DEFAULT_SCHEME: &str = "http";

// Well-known URI.
// https://en.wikipedia.org/wiki/Well-known_URI
//
// Request to the server for well-known services or information are available
// at URLs consistent well-known locations across servers.
const WELL_KNOWN_URI: &str = "/.well-known/ascot";

/// The `Ascot` server.
pub struct AscotServer {
    // HTTP address.
    http_address: Ipv4Addr,
    // Server port.
    port: u16,
    // Scheme.
    scheme: &'static str,
    // Well-known URI.
    well_known_uri: &'static str,
    // Device.
    device: Device,
}

impl AscotServer {
    /// Creates a new [`AscotServer`] instance.
    pub fn new(device: Device, service_address: Ipv4Addr) -> Self {
        Self {
            http_address: service_address,
            port: DEFAULT_SERVER_PORT,
            scheme: DEFAULT_SCHEME,
            well_known_uri: WELL_KNOWN_URI,
            device,
        }
    }

    /// Sets server port.
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sets server scheme.
    pub fn scheme(mut self, scheme: &'static str) -> Self {
        self.scheme = scheme;
        self
    }

    /// Sets well-known URI.
    pub fn well_known_uri(mut self, well_known_uri: &'static str) -> Self {
        self.well_known_uri = well_known_uri;
        self
    }

    /// Runs a smart home device on the server.
    pub fn run(self) -> anyhow::Result<()> {
        let mut server = EspHttpServer::new(&Configuration::default())?;

        for route in self.device.routes_data {
            let method = match route.route_hazards.route.kind() {
                RestKind::Get => Method::Get,
                RestKind::Post => Method::Post,
                RestKind::Put => Method::Put,
            };
            server.fn_handler(route.route_hazards.route.route(), method, route.handler)?;
        }

        // Run service
        MdnsSdService::new().run(self.http_address)?;

        // Run the server endlessly.
        loop {
            // Sleep for one second and then continue the execution.
            sleep(Duration::from_millis(1000));
        }
    }
}
