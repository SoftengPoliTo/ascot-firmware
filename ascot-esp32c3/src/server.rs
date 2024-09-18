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

// Server stack size.
const DEFAULT_STACK_SIZE: usize = 10240;

/// The `Ascot` server.
pub struct AscotServer {
    // HTTP address.
    http_address: Ipv4Addr,
    // Server port.
    port: u16,
    // Stack size
    stack_size: usize,
    // Device.
    device: Device,
}

impl AscotServer {
    /// Creates a new [`AscotServer`] instance.
    pub fn new(device: Device, http_address: Ipv4Addr) -> Self {
        Self {
            http_address,
            port: DEFAULT_SERVER_PORT,
            stack_size: DEFAULT_STACK_SIZE,
            device,
        }
    }

    /// Sets server port.
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sets server stack size.
    pub fn stack_size(mut self, stack_size: usize) -> Self {
        self.stack_size = stack_size;
        self
    }

    /// Runs a smart home device on the server.
    pub fn run(self) -> anyhow::Result<()> {
        let mut server = EspHttpServer::new(&Configuration {
            stack_size: self.stack_size,
            http_port: self.port,
            ..Default::default()
        })?;

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
