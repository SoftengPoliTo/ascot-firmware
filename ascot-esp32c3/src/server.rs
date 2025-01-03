use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::Write;

use ascot_library::device::DeviceSerializer;
use ascot_library::route::RestKind;

use crate::device::Device;
use crate::error::Result;
use crate::service::{InternalService, ServiceConfig};

// Default port.
const DEFAULT_SERVER_PORT: u16 = 3000;

// Server stack size.
const DEFAULT_STACK_SIZE: usize = 10240;

/// The `Ascot` server.
pub struct AscotServer {
    // Server port.
    port: u16,
    // Stack size
    stack_size: usize,
    // Device.
    device: Device,
    // Service configuration.
    service_config: Option<ServiceConfig>,
}

impl AscotServer {
    /// Creates a new [`AscotServer`] instance.
    pub const fn new(device: Device) -> Self {
        Self {
            port: DEFAULT_SERVER_PORT,
            stack_size: DEFAULT_STACK_SIZE,
            device,
            service_config: None,
        }
    }

    /// Sets server port.
    pub const fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sets server stack size.
    pub const fn stack_size(mut self, stack_size: usize) -> Self {
        self.stack_size = stack_size;
        self
    }

    /// Sets a server discovery service configuration.
    pub const fn service(mut self, service_config: ServiceConfig) -> Self {
        self.service_config = Some(service_config);
        self
    }

    /// Runs the server.
    pub fn run(self) -> Result<()> {
        let mut server = EspHttpServer::new(&Configuration {
            stack_size: self.stack_size,
            http_port: self.port,
            ..Default::default()
        })?;

        // Format the device description as a pretty string.
        let device_description = serde_json::to_string_pretty(&self.device.serialize_data())?;

        for route in self.device.routes_data {
            let method = match route.route.kind() {
                RestKind::Get => Method::Get,
                RestKind::Post => Method::Post,
                RestKind::Put => Method::Put,
                RestKind::Delete => Method::Delete,
            };
            if let Some(body) = route.body {
                server.fn_handler(
                    &format!("{}{}", self.device.main_route, route.route.route()),
                    method,
                    move |req| {
                        // Run body.
                        body()?;

                        // Write response.
                        (route.response)(req)?.write_all(route.content.as_bytes())
                    },
                )?;
            } else {
                server.fn_handler(route.route.route(), method, move |req| {
                    // Write only response.
                    (route.response)(req)?.write_all(route.content.as_bytes())
                })?;
            }
        }

        // Add main route
        server.fn_handler(self.device.main_route, Method::Get, move |req| {
            req.into_response(200, Some("OK"), &[("Content-Type", "application/json")])?
                .write_all(device_description.as_bytes())
        })?;

        if let Some(service_config) = self.service_config {
            // Run service
            InternalService::run(service_config)
        } else {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1))
            }
        }
    }
}
