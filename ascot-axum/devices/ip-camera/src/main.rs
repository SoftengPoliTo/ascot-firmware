use core::net::Ipv4Addr;

use std::sync::Arc;

// Ascot library.
use ascot_library::device::{DeviceErrorKind, DeviceKind};
use ascot_library::hazards::Hazard;
use ascot_library::input::Input;
use ascot_library::route::Route;

// Ascot axum.
use ascot_axum::device::{Device, DeviceAction, DeviceError, DevicePayload};
use ascot_axum::error::Error;
use ascot_axum::extract::{Extension, Json, Path};
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

fn make_requested_format(
    frame_format_type: &str,
    frame_format_option: Option<&str>,
) -> Option<RequestedFormat<'static>> {
    match frame_format_type {
        "AbsoluteHighestResolution" => Some(RequestedFormat::new::<RgbFormat>(
            RequestedFormatType::AbsoluteHighestResolution,
        )),
        "AbsoluteHighestFrameRate" => Some(RequestedFormat::new::<RgbFormat>(
            RequestedFormatType::AbsoluteHighestFrameRate,
        )),
        "HighestResolution" => {
            if let Some(fmtv) = frame_format_option {
                let values = fmtv.split(",").collect::<Vec<&str>>();
                let x = values[0].parse::<u32>().unwrap();
                let y = values[1].parse::<u32>().unwrap();
                let resolution = Resolution::new(x, y);

                Some(RequestedFormat::new::<RgbFormat>(
                    RequestedFormatType::HighestResolution(resolution),
                ))
            } else {
                None
            }
        }
        "HighestFrameRate" => {
            if let Some(fmtv) = frame_format_option {
                let fps = fmtv.parse::<u32>().unwrap();

                Some(RequestedFormat::new::<RgbFormat>(
                    RequestedFormatType::HighestFrameRate(fps),
                ))
            } else {
                None
            }
        }
        "Exact" => {
            if let Some(fmtv) = frame_format_option {
                let values = fmtv.split(",").collect::<Vec<&str>>();
                let x = values[0].parse::<u32>().unwrap();
                let y = values[1].parse::<u32>().unwrap();
                let fps = values[2].parse::<u32>().unwrap();
                let fourcc = values[3].parse::<FrameFormat>().unwrap();

                let resolution = Resolution::new(x, y);
                let camera_format = CameraFormat::new(resolution, fourcc, fps);
                Some(RequestedFormat::new::<RgbFormat>(
                    RequestedFormatType::Exact(camera_format),
                ))
            } else {
                None
            }
        }
        "Closest" => {
            if let Some(fmtv) = frame_format_option {
                let values = fmtv.split(",").collect::<Vec<&str>>();
                let x = values[0].parse::<u32>().unwrap();
                let y = values[1].parse::<u32>().unwrap();
                let fps = values[2].parse::<u32>().unwrap();
                let fourcc = values[3].parse::<FrameFormat>().unwrap();

                let resolution = Resolution::new(x, y);
                let camera_format = CameraFormat::new(resolution, fourcc, fps);
                Some(RequestedFormat::new::<RgbFormat>(
                    RequestedFormatType::Closest(camera_format),
                ))
            } else {
                None
            }
        }
        // Random camera format
        "None" | _ => Some(RequestedFormat::new::<RgbFormat>(RequestedFormatType::None)),
    }
}

#[inline(always)]
fn camera_error(error: String) -> DeviceError {
    DeviceError::from_string(DeviceErrorKind::Wrong, error)
}

#[derive(Serialize)]
struct ViewCamerasResponse {
    #[serde(rename = "number-of-cameras")]
    number_of_cameras: u8,
    cameras: Vec<CameraInfo>,
}

async fn view_available_cameras() -> Result<DevicePayload, DeviceError> {
    // Retrieve camera backend
    let camera_backend = native_api_backend().ok_or(DeviceError::from_str(
        DeviceErrorKind::Wrong,
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

    DevicePayload::new(ViewCamerasResponse {
        number_of_cameras: camera_len,
        cameras,
    })
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

async fn camera_data(Path(camera_index): Path<u32>) -> Result<DevicePayload, DeviceError> {
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

    DevicePayload::new(CameraDataResponse {
        camera_index,
        controls,
        frame_formats,
    })
}

#[derive(Deserialize)]
struct CameraInputs {
    camera_index: u32,
    frame_format_type: String,
    frame_format_option: Option<String>,
}

async fn camera_screenshot(Json(inputs): Json<CameraInputs>) -> Result<DevicePayload, DeviceError> {
    // Camera frame format.
    //let requested_format =
    //    make_requested_format(inputs.frame_format_type, inputs.frame_format_option).unwrap();

    let camera_index = inputs.camera_index;

    // Create camera
    let mut camera = Camera::new(
        CameraIndex::Index(camera_index),
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::None),
    )
    .map_err(|e| camera_error(format!("Error in retrieving camera {camera_index}: {e}")))?;

    // Open camera stream.
    camera
        .open_stream()
        .map_err(|e| camera_error(format!("Error in opening a stream for {camera_index}: {e}")))?;

    // Retrieve camera frame.
    let frame = camera
        .frame()
        .map_err(|e| camera_error(format!("Error in getting a frame for {camera_index}: {e}")))?;

    // Stop camera stream.
    camera.stop_stream().map_err(|e| {
        camera_error(format!(
            "Error in stopping a stream for {camera_index}: {e}"
        ))
    })?;

    info!("Capture camera screenshot of size {}", frame.buffer().len());

    // Decode a frame
    let decoded = frame
        .decode_image::<RgbFormat>()
        .map_err(|e| camera_error(format!("Error in decoding a frame for {camera_index}: {e}")))?;
    info!("Decoded frame of size {}", decoded.len());

    Ok(DevicePayload::empty())
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

    // Action to view all available cameras.
    let view_cameras_action = DeviceAction::no_hazards(
        Route::get("/view-all").description("View all cameras."),
        view_available_cameras,
    );

    // Action to view the data of a camera.
    let camera_data_action = DeviceAction::no_hazards(
        Route::get("/:camera_index").description("View camera data."),
        camera_data,
    );

    // Action to show screenshot from a camera.
    let camera_screenshot_action = DeviceAction::no_hazards(
        Route::post("/screenshot").description("Screenshot from a camera."),
        camera_screenshot,
    );

    // A camera device which is going to be run on the server.
    let device = Device::<()>::unknown()
        .main_route("/camera")
        .add_action(view_cameras_action)
        .add_action(camera_data_action)
        .add_action(camera_screenshot_action);

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
