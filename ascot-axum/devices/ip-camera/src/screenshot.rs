// Ascot axum.
use ascot_axum::device::{DeviceError, DevicePayload};
use ascot_axum::error::Error;
use ascot_axum::extract::Json;

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
use serde::Deserialize;

// Tracing library.
use tracing::info;

use crate::camera_error;

/*fn make_requested_format(
    frame_format_type: RequestedFormatType,
    frame_format_option: Option,
) -> Option<RequestedFormat<'static>> {
    match frame_format_type {
        "AboluteHighestResolution" => Some(RequestedFormat::new::<RgbFormat>(
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
}*/

fn run_camera_screenshot(
    camera_index: u32,
    format: RequestedFormat,
) -> Result<DevicePayload, DeviceError> {
    // Create camera
    let mut camera = Camera::new(CameraIndex::Index(camera_index), format)
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

#[derive(Deserialize)]
pub(crate) struct CameraInputIndex {
    camera_index: u32,
}

pub(crate) async fn screenshot_none(
    Json(input): Json<CameraInputIndex>,
) -> Result<DevicePayload, DeviceError> {
    run_camera_screenshot(
        input.camera_index,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::None),
    )
}

pub(crate) async fn screenshot_absolute_resolution(
    Json(input): Json<CameraInputIndex>,
) -> Result<DevicePayload, DeviceError> {
    run_camera_screenshot(
        input.camera_index,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution),
    )
}

pub(crate) async fn screenshot_absolute_framerate(
    Json(input): Json<CameraInputIndex>,
) -> Result<DevicePayload, DeviceError> {
    run_camera_screenshot(
        input.camera_index,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate),
    )
}
