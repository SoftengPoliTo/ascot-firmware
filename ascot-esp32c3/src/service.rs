use core::net::{Ipv4Addr, Ipv6Addr};

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

use log::info;

use rand::{thread_rng, RngCore};

// Service name
const SERVICE_NAME: &str = "ascot";

pub(crate) struct MdnsSdService {
    stack: Stack,
    send_buf: VecBufAccess<NoopRawMutex, 1500>,
    recv_buf: VecBufAccess<NoopRawMutex, 1500>,
}

impl MdnsSdService {
    pub(crate) fn new() -> Self {
        // Create stack
        let stack = Stack::new();

        // Create sender and receiver buffers for mdns-sd protocol.
        let (recv_buf, send_buf) = (VecBufAccess::new(), VecBufAccess::new());

        Self {
            stack,
            send_buf,
            recv_buf,
        }
    }

    pub(crate) fn run(self, ip: Ipv4Addr) -> anyhow::Result<()> {
        // Run mdns-sd service
        block_on(Self::mdns_sd_service::<Stack, _, _>(
            &self.stack,
            &self.recv_buf,
            &self.send_buf,
            SERVICE_NAME,
            ip,
        ))
        .map_err(|e| anyhow!("Error running mdns-sd service: {}", e))
    }

    async fn mdns_sd_service<T, RB, SB>(
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
        info!(
            "About to run an mDNS responder for our PC. \
             It will be addressable using {SERVICE_NAME}.local, \
             so try to `ping {SERVICE_NAME}.local`."
        );

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
