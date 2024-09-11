use core::net::{Ipv4Addr, Ipv6Addr};

use std::thread::sleep;
use std::time::Duration;

use anyhow::anyhow;

use edge_mdns::buf::{BufferAccess, VecBufAccess};
use edge_mdns::domain::base::Ttl;
use edge_mdns::io::{self, MdnsIoError, DEFAULT_SOCKET};
use edge_mdns::{host::Host, HostAnswersMdnsHandler};
use edge_nal::{UdpBind, UdpSplit};
use edge_nal_std::Stack;

use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::signal::Signal;

use esp_idf_svc::hal::task::block_on;
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_svc::http::Method;

use log::info;

use rand::{thread_rng, RngCore};

use ascot_library::route::RestKind;

use crate::device::Device;

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

// Service name
const SERVICE_NAME: &str = "ascot";

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

        // Create stack
        let stack = Stack::new();

        // Create sender and receiver buffers for mdns-sd protocol.
        let (recv_buf, send_buf) = (
            VecBufAccess::<NoopRawMutex, 1500>::new(),
            VecBufAccess::<NoopRawMutex, 1500>::new(),
        );

        // Run mdns-sd service
        block_on(Self::run_service::<Stack, _, _>(
            &stack,
            &recv_buf,
            &send_buf,
            SERVICE_NAME,
            self.http_address,
        ))
        .map_err(|e| anyhow!("Error running mdns-sd service: {}", e))?;

        // Run the server endlessly.
        loop {
            // Sleep for one second and then continue the execution.
            sleep(Duration::from_millis(1000));
        }
    }

    async fn run_service<T, RB, SB>(
        stack: &T,
        recv_buf: RB,
        send_buf: SB,
        our_name: &str,
        our_ip: Ipv4Addr,
    ) -> Result<(), MdnsIoError<T::Error>>
    where
        T: UdpBind,
        RB: BufferAccess<[u8]>,
        SB: BufferAccess<[u8]>,
    {
        info!("About to run an mDNS responder for our PC. It will be addressable using {SERVICE_NAME}.local, so try to `ping {SERVICE_NAME}.local`.");

        // No ipv6 up and running.
        // To have it running, we need to get at least a link-local ipv6 addr
        // first, using an `esp-idf-sys` API call once the wifi is up and running:
        // `esp_idf_svc::sys::esp_netif_create_ip6_linklocal`.
        // Moreover, we can't just pass "0" for the interface.
        // We need to pass `wifi.sta_netif().index()`
        // Sometimes, "0" does work on PCs, but not with ESP-IDF.
        // This API is very picky about having a correct ipv6-capable
        // interface rather than just "all" (= 0).
        let mut socket = io::bind(stack, DEFAULT_SOCKET, Some(Ipv4Addr::UNSPECIFIED), None).await?;

        let (recv, send) = socket.split();

        let host = Host {
            hostname: our_name,
            ipv4: our_ip,
            ipv6: Ipv6Addr::UNSPECIFIED,
            ttl: Ttl::from_secs(60),
        };

        // A way to notify the mDNS responder that the data in `Host` had changed
        // Not necessary for this example, because the data is hard-coded
        let signal = Signal::new();

        let mdns = io::Mdns::<NoopRawMutex, _, _, _, _>::new(
            Some(Ipv4Addr::UNSPECIFIED),
            // No ipv6 up and running.
            None,
            recv,
            send,
            recv_buf,
            send_buf,
            |buf| thread_rng().fill_bytes(buf),
            &signal,
        );

        mdns.run(HostAnswersMdnsHandler::new(&host)).await
    }
}
