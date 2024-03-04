use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post, put},
    Router,
};
use thiserror::Error;
use tracing::info;

use crate::action::RestKind;
use crate::device::Device;
use crate::service::{ServiceBuilder, ServiceBuilderError};

// Default HTTP address.
//
// The entire local network is considered, so the Ipv4 unspecified address is
// used.
const DEFAULT_HTTP_ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::UNSPECIFIED);

// Default port.
pub(crate) const DEFAULT_SERVER_PORT: u16 = 3000;

/// All `[AscotServer]` errors.
#[derive(Debug, Error)]
pub enum AscotServerError {
    /// A service builder error.
    #[error("service builder error {0}")]
    Service(#[from] ServiceBuilderError),
    /// Network error.
    #[error("I/O error {0}")]
    Io(#[from] std::io::Error),
    /// Serialize error.
    #[error("Serialize error {0}")]
    Json(#[from] serde_json::Error),
}

/// A specialized `Result` type.
pub type Result<T> = std::result::Result<T, AscotServerError>;

// Default scheme is `http`.
const DEFAULT_SCHEME: &str = "http";

// Well-known URI.
// https://en.wikipedia.org/wiki/Well-known_URI
//
// Request to the server for well-known services or information are available
// at URLs consistent well-known locations across servers.
const WELL_KNOWN_URI: &str = "/.well-known/ascot";

/// The `Ascot` server.
#[derive(Debug)]
pub struct AscotServer {
    // HTTP address.
    http_address: IpAddr,
    // Server port.
    port: u16,
    // Scheme.
    scheme: &'static str,
    // Well-known URI.
    well_known_uri: &'static str,
    // An external service.
    service: Option<ServiceBuilder>,
    // Device.
    //
    // Only a single device per server.
    device: Device,
}

#[derive(Clone)]
struct AppState(Arc<dyn crate::action::Task + Send + Sync + 'static>);

impl IntoResponse for crate::action::TaskError {
    fn into_response(self) -> Response {
        let body = format!("Error: {}", self.0);
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

async fn handler(State(state): State<AppState>, request: Request) -> crate::action::Result<()> {
    info!("Performing the REST API ({})", request.method());
    state.0.task()
}

impl AscotServer {
    /// Creates a new `Ascot` server instance.
    pub fn new(device: Device) -> Self {
        Self {
            http_address: DEFAULT_HTTP_ADDRESS,
            port: DEFAULT_SERVER_PORT,
            scheme: DEFAULT_SCHEME,
            well_known_uri: WELL_KNOWN_URI,
            service: None,
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

    /// Sets a new service.
    ///
    /// The service port is automatically set to the same in use on the server.
    pub fn service(mut self, service: ServiceBuilder) -> Self {
        if service.port != self.port {
            self.service = Some(service.port(self.port))
        }
        self
    }

    /// Runs the server.
    pub async fn run(mut self) -> Result<()> {
        // Initialize tracing subscriber.
        tracing_subscriber::fmt::init();

        // Run service.
        if let Some(ref mut service) = self.service {
            service.build()?;
        } else {
            ServiceBuilder::new("Server")
                .port(self.port)
                .property(("scheme", self.scheme))
                .property(("path", self.well_known_uri))
                .build()?;
        }

        // Create application.
        let router = self.build_app()?;

        // Create a new TCP socket which responds to the specified HTTP address
        // and port.
        let listener =
            tokio::net::TcpListener::bind(format!("{}:{}", self.http_address, self.port)).await?;

        // Print server start message
        info!(
            "Starting the Ascot server at {} (port {})…",
            self.http_address, self.port
        );

        // Start the server
        axum::serve(listener, router).await?;

        Ok(())
    }

    fn build_app(&self) -> Result<Router> {
        // Serialize device information returning a json format.
        let device_info = self.device.json()?;

        // Create a new router.
        //
        //- Save device info as a json format which is returned when a query to
        //  the server root is requested.
        //- Redirect well-known URI to server root.
        let mut router = Router::new()
            .route(
                "/",
                axum::routing::get(move || async { axum::Json(device_info) }),
            )
            .route(
                self.well_known_uri,
                axum::routing::get(move || async { Redirect::to("/") }),
            );

        for action in self.device.actions() {
            let state = AppState(action.task.clone());
            let route = format!("{}/{}", self.device.main_route, action.route);
            router = match action.rest_kind {
                RestKind::Get => router.route(&route, get(handler).with_state(state)),
                RestKind::Put => router.route(&route, put(handler).with_state(state)),
                RestKind::Post => router.route(&route, post(handler).with_state(state)),
            };
        }
        Ok(router)
    }
}
