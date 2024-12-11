mod info;
mod screenshot;
mod stream;

use std::borrow::Cow;
use std::net::Ipv4Addr;
use std::sync::Arc;

// Ascot library.
use ascot_library::route::{Route, RouteHazards};

// Ascot axum.
use ascot_axum::actions::serial::{serial_stateful, serial_stateless};
use ascot_axum::actions::stream::stream_stateful;
use ascot_axum::actions::ActionError;
use ascot_axum::device::Device;
use ascot_axum::error::{Error, ErrorKind};
use ascot_axum::server::AscotServer;
use ascot_axum::service::ServiceConfig;

// Mutex
use async_lock::Mutex;

// Command line library
use clap::Parser;

// Nokhwa library
use nokhwa::{native_api_backend, query, utils::CameraIndex};

// Tracing library.
use tracing::info;
use tracing_subscriber::filter::LevelFilter;

use crate::info::{camera_info, view_available_cameras};

use crate::screenshot::{
    screenshot_absolute_framerate, screenshot_absolute_resolution, screenshot_closest,
    screenshot_exact, screenshot_highest_framerate, screenshot_highest_resolution,
    screenshot_random,
};

fn camera_error(error: impl AsRef<str>) -> ActionError {
    ActionError::internal(error.as_ref())
}

fn startup_error(error: impl Into<Cow<'static, str>>) -> Error {
    Error::new(ErrorKind::External, error)
}

#[derive(Clone)]
struct InternalState {
    camera: Arc<Mutex<CameraIndex>>,
}

impl InternalState {
    fn new(camera: CameraIndex) -> Self {
        Self {
            camera: Arc::new(Mutex::new(camera)),
        }
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Server address.
    ///
    /// Only an `Ipv4` address is accepted.
    #[arg(short, long, default_value_t = Ipv4Addr::UNSPECIFIED)]
    address: Ipv4Addr,

    /// Server host name.
    #[arg(short = 'n', long)]
    hostname: String,

    /// Server port.
    #[arg(short, long, default_value_t = 3000)]
    port: u16,

    /// Service domain.
    #[arg(short, long, default_value = "device")]
    domain: String,

    /// Service type.
    #[arg(short = 't', long = "type", default_value = "General Fridge")]
    service_type: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing subscriber.
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .init();

    // Command line parser.
    let cli = Cli::parse();

    // This initialization is necessary only on MacOS, but we are also going
    // to use this call to verify if everything went well.
    nokhwa::nokhwa_initialize(|granted| {
        if granted {
            info!("Nokhwa initialized correctly.");
        } else {
            info!("Nokhwa not initialized correctly. Exiting the process.");
            std::process::exit(1);
        }
    });

    // Retrieve native API camera backend
    let camera_backend =
        native_api_backend().ok_or(startup_error("No camera backend found at server startup"))?;

    // Retrieve all cameras present on a system
    let cameras = query(camera_backend).map_err(|e| {
        startup_error(format!(
            "The backend cannot find any camera at server startup: {e}"
        ))
    })?;

    // Retrieve first camera present in the system
    let first_camera = cameras.first().ok_or(startup_error(
        "No cameras found in the system at server startup",
    ))?;

    // Route to view all available cameras.
    let view_cameras_route =
        RouteHazards::no_hazards(Route::get("/view-all").description("View all system cameras."));

    // Route to view the camera info.
    let camera_info_route =
        RouteHazards::no_hazards(Route::get("/info").description("View current camera data."));

    // Route to take a screenshot with a random format.
    let screenshot_random_route = RouteHazards::no_hazards(
        Route::get("/screenshot-random").description("Screenshot with a random camera format."),
    );

    // Route to view screenshot with absolute resolution.
    let screenshot_absolute_resolution_route = RouteHazards::no_hazards(
        Route::get("/screenshot-absolute-resolution")
            .description("Screenshot from a camera with absolute resolution."),
    );

    // Route to view screenshot with absolute framerate.
    let screenshot_absolute_framerate_route = RouteHazards::no_hazards(
        Route::get("/screenshot-absolute-framerate")
            .description("Screenshot from a camera with absolute framerate."),
    );

    // Route to view screenshot with highest resolution.
    let screenshot_highest_resolution_route = RouteHazards::no_hazards(
        Route::post("/screenshot-highest-resolution")
            .description("Screenshot from a camera with highest resolution."),
    );

    // Route to view screenshot with highest framerate.
    let screenshot_highest_framerate_route = RouteHazards::no_hazards(
        Route::post("/screenshot-highest-framerate")
            .description("Screenshot from a camera with highest framerate."),
    );

    // Route to view screenshot with exact approach.
    let screenshot_exact_route = RouteHazards::no_hazards(
        Route::post("/screenshot-exact").description("Screenshot from a camera with exact type."),
    );

    // Route to view screenshot with closest type.
    let screenshot_closest_route = RouteHazards::no_hazards(
        Route::post("/screenshot-closest")
            .description("Screenshot from a camera with closest type."),
    );

    // A camera device which is going to be run on the server.
    let device = Device::with_state(InternalState::new(first_camera.index().clone()))
        .main_route("/camera")
        .add_action(serial_stateless(view_cameras_route, view_available_cameras))
        .add_action(serial_stateful(camera_info_route, camera_info))
        .add_action(stream_stateful(screenshot_random_route, screenshot_random))
        .add_action(stream_stateful(
            screenshot_absolute_resolution_route,
            screenshot_absolute_resolution,
        ))
        .add_action(stream_stateful(
            screenshot_absolute_framerate_route,
            screenshot_absolute_framerate,
        ))
        .add_action(stream_stateful(
            screenshot_highest_resolution_route,
            screenshot_highest_resolution,
        ))
        .add_action(stream_stateful(
            screenshot_highest_framerate_route,
            screenshot_highest_framerate,
        ))
        .add_action(stream_stateful(screenshot_exact_route, screenshot_exact))
        .add_action(stream_stateful(
            screenshot_closest_route,
            screenshot_closest,
        ));

    // Run a discovery service and the device on the server.
    AscotServer::new(device)
        .address(cli.address)
        .port(cli.port)
        .service(
            ServiceConfig::mdns_sd("camera")
                .hostname(&cli.hostname)
                .domain_name(&cli.domain)
                .service_type(&cli.service_type),
        )
        .run()
        .await
}
