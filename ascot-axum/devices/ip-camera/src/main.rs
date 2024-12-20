mod info;
mod screenshot;
mod stream;

use std::net::Ipv4Addr;
use std::sync::Arc;

// Ascot library.
use ascot_library::route::{Route, RouteHazards};

// Ascot axum.
use ascot_axum::actions::serial::SerialPayload;
use ascot_axum::actions::serial::{serial_stateful, serial_stateless};
use ascot_axum::actions::stream::stream_stateful;
use ascot_axum::actions::ActionError;
use ascot_axum::device::Device;
use ascot_axum::error::Error;
use ascot_axum::extract::{Path, State};
use ascot_axum::server::AscotServer;
use ascot_axum::service::ServiceConfig;

// Mutex
use async_lock::Mutex;

// Command line library
use clap::Parser;

// Nokhwa library
use nokhwa::{
    native_api_backend, query,
    utils::{ApiBackend, CameraIndex, RequestedFormatType},
};

use serde::Serialize;

// Tracing library.
use tracing::{error, info};
use tracing_subscriber::filter::LevelFilter;

use crate::info::{show_available_cameras, show_camera_info};
use crate::screenshot::{
    screenshot_absolute_framerate, screenshot_absolute_resolution, screenshot_closest,
    screenshot_exact, screenshot_highest_framerate, screenshot_highest_resolution,
    screenshot_random,
};
use crate::stream::show_camera_stream;

fn startup_error(error: &str) -> Error {
    Error::external(format!("{error} at server startup"))
}

fn startup_with_error(description: &str, error: impl std::error::Error) -> Error {
    Error::external(format!(
        r#"
            {description} at server startup
            Info: {error}
        "#
    ))
}

fn camera_error(description: &'static str, error: impl std::error::Error) -> ActionError {
    ActionError::internal_with_error(description, error)
}

fn thread_error<T: std::fmt::Display>(msg: &str, e: T) {
    error!("{msg}");
    error!("{e}");
}

#[derive(Clone)]
struct CameraConfig {
    index: CameraIndex,
    format_type: RequestedFormatType,
}

#[derive(Clone)]
struct InternalState {
    camera: Arc<Mutex<CameraConfig>>,
}

impl InternalState {
    fn new(camera: CameraConfig) -> Self {
        Self {
            camera: Arc::new(Mutex::new(camera)),
        }
    }
}

#[derive(Serialize)]
struct ChangeCameraResponse {
    message: String,
}

// Change camera.
async fn change_camera(
    State(state): State<InternalState>,
    Path(index): Path<CameraIndex>,
) -> Result<SerialPayload<ChangeCameraResponse>, ActionError> {
    let mut config = state.camera.lock().await;

    // Check whether the new index is equal to the previous one.
    if config.index == index {
        let message = format!("Camera index remained unchanged at {}", config.index);
        info!("{message}");
        return Ok(SerialPayload::new(ChangeCameraResponse { message }));
    }

    // Retrieve all cameras present on a system
    let cameras = query(ApiBackend::Auto).map_err(|e| camera_error("No cameras found", e))?;

    for camera in &cameras {
        if *camera.index() == index {
            config.index = index;
            break;
        }
    }

    let message = format!("Camera changed to index {}", config.index);
    info!("{message}");

    Ok(SerialPayload::new(ChangeCameraResponse { message }))
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
    #[arg(short = 't', long = "type", default_value = "General Device")]
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
    let camera_backend = native_api_backend().ok_or(startup_error("No camera backend found"))?;

    // Retrieve all cameras present on a system
    let cameras = query(camera_backend)
        .map_err(|e| startup_with_error("The backend cannot find any camera", e))?;

    // Retrieve first camera present in the system
    let first_camera = cameras
        .first()
        .ok_or(startup_error("No cameras found in the system"))?;

    // Camera configuration.
    let camera = CameraConfig {
        index: first_camera.index().clone(),
        format_type: RequestedFormatType::None,
    };

    // Route to view all available cameras.
    let view_cameras_route =
        RouteHazards::no_hazards(Route::get("/view-all").description("View all system cameras."));

    // Route to view camera info.
    let camera_info_route =
        RouteHazards::no_hazards(Route::get("/info").description("View current camera data."));

    // Route to change camera index.
    let change_camera_route =
        RouteHazards::no_hazards(Route::get("/change-camera").description("Change camera."));

    // Route to change stream format.
    let stream_format_random_route = RouteHazards::no_hazards(
        Route::post("/stream-format").description("Change stream format to random."),
    );

    // Route to view camera stream.
    let camera_stream_route =
        RouteHazards::no_hazards(Route::get("/stream").description("View camera stream."));

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
    let device = Device::with_state(InternalState::new(camera))
        .main_route("/camera")
        .add_action(serial_stateless(view_cameras_route, show_available_cameras))
        .add_action(serial_stateful(camera_info_route, show_camera_info))
        .add_action(serial_stateful(change_camera_route, change_camera))
        .add_action(stream_stateful(camera_stream_route, show_camera_stream))
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
