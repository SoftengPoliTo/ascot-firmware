mod screenshot;

use core::net::Ipv4Addr;

use std::sync::Arc;

// Ascot library.
use ascot_library::actions::ActionErrorKind;
use ascot_library::hazards::Hazard;
use ascot_library::input::Input;
use ascot_library::route::{Route, RouteHazards};

// Ascot axum.
use ascot_axum::actions::serial::{serial_stateless, SerialPayload};
use ascot_axum::actions::stream::stream_stateless;
use ascot_axum::actions::ActionError;
use ascot_axum::device::Device;
use ascot_axum::error::Error;
use ascot_axum::extract::{Json, Path};
use ascot_axum::server::AscotServer;
use ascot_axum::service::ServiceConfig;

// Command line library
use clap::Parser;

// Nokhwa library
use nokhwa::{
    native_api_backend,
    pixel_format::{RgbAFormat, RgbFormat},
    query,
    utils::{
        frame_formats, yuyv422_predicted_size, ApiBackend, CameraFormat, CameraIndex, CameraInfo,
        FrameFormat, RequestedFormat, RequestedFormatType, Resolution,
    },
    Buffer, Camera, NokhwaError,
};

// Serde library.
use serde::{Deserialize, Serialize};

// Tracing library.
use tracing::info;
use tracing_subscriber::filter::LevelFilter;

use crate::screenshot::{
    screenshot_absolute_framerate, screenshot_absolute_resolution, screenshot_closest,
    screenshot_exact, screenshot_highest_framerate, screenshot_highest_resolution, screenshot_none,
};

fn camera_error(error: String) -> ActionError {
    ActionError::from_str(ActionErrorKind::Internal, &error)
}

#[derive(Serialize)]
struct ViewCamerasResponse {
    #[serde(rename = "number-of-cameras")]
    number_of_cameras: u8,
    cameras: Vec<CameraInfo>,
}

async fn view_available_cameras() -> Result<SerialPayload<ViewCamerasResponse>, ActionError> {
    // Retrieve camera backend
    let camera_backend = native_api_backend().ok_or(ActionError::from_str(
        ActionErrorKind::Internal,
        "No camera backend found",
    ))?;

    // Retrieve all cameras present on a system
    let cameras =
        query(camera_backend).map_err(|e| camera_error(format!("No cameras found: {e}")))?;

    // Number of cameras
    let camera_len = cameras.len() as u8;

    info!("There are {} available cameras.", camera_len);

    for camera in &cameras {
        info!("{camera}");
    }

    Ok(SerialPayload::new(ViewCamerasResponse {
        number_of_cameras: camera_len,
        cameras,
    }))
}

#[derive(Serialize)]
struct CameraDataResponse {
    camera_index: u32,
    controls: Vec<String>,
    frame_formats: Vec<CameraFrameFormat>,
}

#[derive(Serialize)]
struct CameraFrameFormat {
    frame_format: FrameFormat,
    format_data: Vec<FormatData>,
}

#[derive(Serialize)]
struct FormatData {
    resolution: Resolution,
    fps: String,
}

async fn camera_data(
    Path(camera_index): Path<u32>,
) -> Result<SerialPayload<CameraDataResponse>, ActionError> {
    // Create camera
    let mut camera = Camera::new(
        CameraIndex::Index(camera_index),
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::None),
    )
    .map_err(|e| camera_error(format!("Error in retrieving camera {camera_index}: {e}")))?;

    // Get controls for a camera
    let controls = camera.camera_controls().map_err(|e| {
        camera_error(format!(
            "Error in retrieving controls for camera {camera_index}: {e}"
        ))
    })?;

    info!("Controls for camera {camera_index}");

    let controls = controls
        .into_iter()
        .map(|control| control.to_string())
        .inspect(|control| info!("{control}"))
        .collect::<Vec<String>>();

    // Iterate over frame formats.
    let frame_formats = frame_formats()
        .into_iter()
        .filter_map(|frame_format| {
            let frame_format = *frame_format;

            // Among the frame formats, save the ones compatible with the camera
            if let Ok(compatible) = camera.compatible_list_by_resolution(frame_format) {
                info!("{frame_format}:");

                let mut formats = compatible
                    .into_iter()
                    .map(|(resolution, fps)| (resolution, fps))
                    .collect::<Vec<(Resolution, Vec<u32>)>>();

                // Sort formats by name
                formats.sort_by(|a, b| a.0.cmp(&b.0));

                // Show sorted formats.
                let format_data = formats
                    .into_iter()
                    .map(|(resolution, fps)| {
                        let fps = format!("{fps:?}");
                        info!(" - {resolution}: {fps}");
                        FormatData { resolution, fps }
                    })
                    .collect::<Vec<FormatData>>();

                Some(CameraFrameFormat {
                    frame_format,
                    format_data,
                })
            } else {
                None
            }
        })
        .collect::<Vec<CameraFrameFormat>>();

    Ok(SerialPayload::new(CameraDataResponse {
        camera_index,
        controls,
        frame_formats,
    }))
}

/*fn camera_stream(
    index: &str,
    frame_format_type: &str,
    frame_format_option: Option<&str>,
    display_frame: bool,
) {
    // Camera index.
    let index = CameraIndex::String(index.into());

    // Camera frame format.
    let requested_format = make_requested_format(frame_format_type, frame_format_option).unwrap();

    /*if display_frame {
        // Sender and receiver channels.
        let (sender, receiver) = flume::unbounded();
        let (sender, receiver) = (Arc::new(sender), Arc::new(receiver));
        let sender_clone = sender.clone();

        // Create a thread for sending the camera frame.
        let mut camera = CallbackCamera::new(index, requested_format, move |buf| {
            sender_clone.send(buf).expect("Error sending frame!!!!");
        })
        .unwrap();

        // Retrieve camera info.
        let camera_info = camera.info().clone();
        // Retrieve camera format.
        let format = camera.camera_format().unwrap();

        // Open camera stream.
        camera.open_stream().unwrap();
    } else {
        // Define a thread which captures a frame
        let mut cb = CallbackCamera::new(index, requested_format, |buf| {
            info!("Captured frame of size {}", buf.buffer().len());
        })
        .unwrap();

        // Open a camera stream
        cb.open_stream().unwrap();
        // Run the stream endlessly
        loop {}
    }*/
}*/

async fn show_cameras_info(//Extension(state): Extension<CameraState>,
) {
    info!("Start");
    // this will go out of scope either when this function returns (which it never does)
    // or hyper drops the future because the client closed the connection
    let _guard = Guard;

    // Retrieves camera images.
    let mut send_task = tokio::spawn(async move { loop {} });

    // Checks browser connection. This task waits forever.
    let mut recv_task = tokio::spawn(async move { std::future::pending::<()>().await });

    // If any one of the tasks run to completion, we abort the other.
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    };
}

struct Guard;

impl Drop for Guard {
    fn drop(&mut self) {
        println!("Dropping `Guard` when a connection is closed!")
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

    // This initialization is necessary only on MacOS, but we are also going
    // to use this call to verify if everything went well.
    nokhwa::nokhwa_initialize(|granted| {
        if granted {
            info!("Nokhwa initalized correctly.");
        } else {
            info!("Nokhwa not initialized correctly. Exiting the process.");
            std::process::exit(1);
        }
    });

    // Command line parser.
    let cli = Cli::parse();

    // Route to view all available cameras.
    let view_cameras_route =
        RouteHazards::no_hazards(Route::get("/view-all").description("View all cameras."));

    // Route to view the data of a camera.
    let camera_data_route =
        RouteHazards::no_hazards(Route::get("/:camera_index").description("View camera data."));

    // Route to view screenshot with no format.
    let screenshot_none_route = RouteHazards::no_hazards(
        Route::post("/screenshot-none").description("Screenshot from a camera with no format."),
    );

    // Route to view screenshot with absolute resolution.
    let screenshot_absolute_resolution_route = RouteHazards::no_hazards(
        Route::post("/screenshot-absolute-resolution")
            .description("Screenshot from a camera with absolute resolution."),
    );

    // Route to view screenshot with absolute framerate.
    let screenshot_absolute_framerate_route = RouteHazards::no_hazards(
        Route::post("/screenshot-absolute-framerate")
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
    let device = Device::new()
        .main_route("/camera")
        .add_action(serial_stateless(view_cameras_route, view_available_cameras))
        .add_action(serial_stateless(camera_data_route, camera_data))
        .add_action(stream_stateless(screenshot_none_route, screenshot_none))
        .add_action(stream_stateless(
            screenshot_absolute_resolution_route,
            screenshot_absolute_resolution,
        ))
        .add_action(stream_stateless(
            screenshot_absolute_framerate_route,
            screenshot_absolute_framerate,
        ))
        .add_action(stream_stateless(
            screenshot_highest_resolution_route,
            screenshot_highest_resolution,
        ))
        .add_action(stream_stateless(
            screenshot_highest_framerate_route,
            screenshot_highest_framerate,
        ))
        .add_action(stream_stateless(screenshot_exact_route, screenshot_exact))
        .add_action(stream_stateless(
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
