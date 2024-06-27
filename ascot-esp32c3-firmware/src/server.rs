use core::fmt::Debug;
use core::net::{IpAddr, Ipv4Addr};

use std::thread::sleep;
use std::time::Duration;

use ascot_library::device::DeviceSerializer;

use embedded_svc::{
    http::{Headers, Method},
    io::{Read, Write},
};

use esp_idf_svc::http::server::{Configuration, Connection, EspHttpServer, FnHandler, Request};

use serde::Deserialize;

use crate::device::Device;

// Default HTTP address.
//
// The entire local network is considered, so the Ipv4 unspecified address is
// used.
const DEFAULT_HTTP_ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::UNSPECIFIED);

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

// Max payload length
const MAX_LEN: usize = 128;

// Stack size needed to parse a JSON file
const STACK_SIZE: usize = 10240;

static INDEX_HTML: &str = include_str!("../http_server_page.html");

/// The `Ascot` server.
pub struct AscotServer<C, E, F>
where
    C: Connection,
    F: Fn(Request<&mut C>) -> Result<(), E> + Send,
    E: Debug,
{
    // HTTP address.
    http_address: IpAddr,
    // Server port.
    port: u16,
    // Scheme.
    scheme: &'static str,
    // Well-known URI.
    well_known_uri: &'static str,
    // Server configuration.
    configuration: Configuration,
    // Device.
    device: Device<C, E, F>,
}

impl<C, E, F> AscotServer<C, E, F>
where
    C: Connection,
    F: Fn(Request<&mut C>) -> Result<(), E> + Send,
    E: Debug,
{
    /// Creates a new [`AscotServer`] instance.
    pub fn new(device: Device<C, E, F>) -> Self {
        let configuration = Configuration {
            stack_size: STACK_SIZE,
            ..Default::default()
        };

        Self {
            http_address: DEFAULT_HTTP_ADDRESS,
            port: DEFAULT_SERVER_PORT,
            scheme: DEFAULT_SCHEME,
            well_known_uri: WELL_KNOWN_URI,
            configuration,
            device,
        }
    }

    /// Sets a new HTTP address.
    pub fn http_address(mut self, http_address: IpAddr) -> Self {
        self.http_address = http_address;
        self
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

    /*/// Runs a service.
    pub fn run_service(self, service: ServiceBuilder) -> Result<Self> {
        // Add server properties.
        let service = service
            .port(self.port)
            .property(("scheme", self.scheme))
            .property(("path", self.well_known_uri));

        // Run service.
        Service::run(service)?;

        Ok(self)
    }*/

    /// Runs a smart home device on the server.
    pub async fn run(self) -> anyhow::Result<()> {
        let mut server = EspHttpServer::new(&self.configuration)?;

        server.fn_handler::<anyhow::Error, _>("/", Method::Get, |req| {
            req.into_ok_response()?
                .write_all(INDEX_HTML.as_bytes())
                .map(|_| ())
                .map_err(|e| e.into())
        })?;

        loop {
            sleep(Duration::from_millis(1000));
        }
    }
}

pub(crate) fn run_server() -> anyhow::Result<()> {
    let server_configuration = esp_idf_svc::http::server::Configuration {
        stack_size: STACK_SIZE,
        ..Default::default()
    };

    let mut server = EspHttpServer::new(&server_configuration)?;

    server.fn_handler("/", Method::Get, |req| {
        req.into_ok_response()?
            .write_all(INDEX_HTML.as_bytes())
            .map(|_| ())
    })?;

    server.fn_handler::<anyhow::Error, _>("/put", Method::Put, |mut req| {
        #[derive(Deserialize)]
        struct FormData<'a> {
            first_name: &'a str,
            age: u32,
            birthplace: &'a str,
        }

        let len = req.content_len().unwrap_or(0) as usize;

        if len > MAX_LEN {
            req.into_status_response(413)?
                .write_all("Request too big".as_bytes())?;
            return Ok(());
        }

        let mut buf = vec![0; len];
        req.read_exact(&mut buf)?;
        let mut resp = req.into_ok_response()?;

        if let Ok(form) = serde_json::from_slice::<FormData>(&buf) {
            write!(
                resp,
                "Hello, {}-year-old {} from {}!",
                form.age, form.first_name, form.birthplace
            )?;
        } else {
            resp.write_all("JSON error".as_bytes())?;
        }

        Ok(())
    })?;

    loop {
        sleep(Duration::from_millis(1000));
    }
}
