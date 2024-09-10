use core::fmt::Debug;
use core::net::{Ipv4Addr, Ipv6Addr};

use std::thread::sleep;
use std::time::Duration;

use edge_mdns::buf::{BufferAccess, VecBufAccess};
use edge_mdns::domain::base::Ttl;
use edge_mdns::io::{self, MdnsIoError, DEFAULT_SOCKET};
use edge_mdns::{host::Host, HostAnswersMdnsHandler};
use edge_nal::{UdpBind, UdpSplit};
use edge_nal_std::Stack;

use embedded_svc::http::Method;

use esp_idf_svc::http::server::{Configuration, EspHttpConnection, EspHttpServer, Request};

use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::signal::Signal;

use futures_lite::future::block_on;

use rand::{thread_rng, RngCore};

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

// Stack size needed to parse a JSON file
const STACK_SIZE: usize = 10240;

// mdns-sd service.
const SERVICE_TYPE: &str = "_ascot";

/// The `Ascot` server.
pub struct AscotServer<E, F>
where
    F: for<'r> Fn(Request<&mut EspHttpConnection<'r>>) -> Result<(), E> + Send + 'static,
    E: Debug,
{
    // HTTP address.
    http_address: Ipv4Addr,
    // Server port.
    port: u16,
    // Scheme.
    scheme: &'static str,
    // Well-known URI.
    well_known_uri: &'static str,
    // Server configuration.
    configuration: Configuration,
    // Device.
    device: Device<E, F>,
}

impl<E, F> AscotServer<E, F>
where
    F: for<'r> Fn(Request<&mut EspHttpConnection<'r>>) -> Result<(), E> + Send + 'static,
    E: Debug,
{
    /// Creates a new [`AscotServer`] instance.
    pub fn new(device: Device<E, F>, service_address: Ipv4Addr) -> Self {
        let configuration = Configuration {
            stack_size: STACK_SIZE,
            ..Default::default()
        };

        Self {
            http_address: service_address,
            port: DEFAULT_SERVER_PORT,
            scheme: DEFAULT_SCHEME,
            well_known_uri: WELL_KNOWN_URI,
            configuration,
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
        let mut server = EspHttpServer::new(&self.configuration)?;

        for route in self.device.routes_data.into_iter() {
            server.fn_handler("/", Method::Get, route.handler)?;
        }

        let stack = Stack::new();

        let (recv_buf, send_buf) = (
            VecBufAccess::<NoopRawMutex, 1500>::new(),
            VecBufAccess::<NoopRawMutex, 1500>::new(),
        );

        // Block the main thread to initialize the service.
        // If everything goes well, keep going with the server execution.
        block_on(Self::run_service::<Stack, _, _>(
            &stack,
            &recv_buf,
            &send_buf,
            SERVICE_TYPE,
            self.http_address,
        ))
        .map_err(|e| anyhow::anyhow!("Error running the mdns-sd service: {}", e))?;

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
        // About to run an mDNS responder for our PC.
        // It will be addressable using _ascot.local, so try to `ping _ascot.local`.

        let mut socket =
            io::bind(stack, DEFAULT_SOCKET, Some(Ipv4Addr::UNSPECIFIED), Some(0)).await?;

        let (recv, send) = socket.split();

        let host = Host {
            hostname: our_name,
            ipv4: our_ip,
            ipv6: Ipv6Addr::UNSPECIFIED,
            ttl: Ttl::from_secs(60),
        };

        // A way to notify the mDNS responder that the data in `Host` had changed
        // We don't use it in this example, because the data is hard-coded
        let signal = Signal::new();

        let mdns = io::Mdns::<NoopRawMutex, _, _, _, _>::new(
            Some(Ipv4Addr::UNSPECIFIED),
            Some(0),
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
