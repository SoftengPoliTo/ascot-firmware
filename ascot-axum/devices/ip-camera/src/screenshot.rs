// Ascot axum.
use ascot_axum::actions::stream::StreamPayload;
use ascot_axum::actions::ActionError;

use ascot_axum::extract::Json;
use ascot_axum::header;

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

fn run_camera_screenshot(
    camera_index: u32,
    format: RequestedFormat,
) -> Result<StreamPayload, ActionError> {
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

    let headers = [
        (header::CONTENT_TYPE, "image/png"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"screenshot.png\"",
        ),
    ];

    Ok(StreamPayload::new(headers, decoded.into_vec()))
}

#[derive(Deserialize)]
pub(crate) struct CameraInputIndex {
    camera_index: u32,
}

pub(crate) async fn screenshot_none(
    Json(input): Json<CameraInputIndex>,
) -> Result<StreamPayload, ActionError> {
    run_camera_screenshot(
        input.camera_index,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::None),
    )
}

pub(crate) async fn screenshot_absolute_resolution(
    Json(input): Json<CameraInputIndex>,
) -> Result<StreamPayload, ActionError> {
    run_camera_screenshot(
        input.camera_index,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution),
    )
}

pub(crate) async fn screenshot_absolute_framerate(
    Json(input): Json<CameraInputIndex>,
) -> Result<StreamPayload, ActionError> {
    run_camera_screenshot(
        input.camera_index,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate),
    )
}

#[derive(Deserialize)]
pub(crate) struct CameraResolution {
    camera_index: u32,
    x: u32,
    y: u32,
}

pub(crate) async fn screenshot_highest_resolution(
    Json(inputs): Json<CameraResolution>,
) -> Result<StreamPayload, ActionError> {
    let resolution = Resolution::new(inputs.x, inputs.y);

    run_camera_screenshot(
        inputs.camera_index,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::HighestResolution(resolution)),
    )
}

#[derive(Deserialize)]
pub(crate) struct CameraFramerate {
    camera_index: u32,
    fps: u32,
}

pub(crate) async fn screenshot_highest_framerate(
    Json(inputs): Json<CameraFramerate>,
) -> Result<StreamPayload, ActionError> {
    run_camera_screenshot(
        inputs.camera_index,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::HighestFrameRate(inputs.fps)),
    )
}

#[derive(Deserialize)]
pub(crate) struct CameraInputs {
    camera_index: u32,
    x: u32,
    y: u32,
    fps: u32,
    fourcc: String,
}

#[inline]
fn camera_format(inputs: CameraInputs) -> Result<(u32, CameraFormat), ActionError> {
    let fourcc = inputs.fourcc.parse::<FrameFormat>().map_err(|e| {
        camera_error(format!(
            "Wrong fourcc value for camera {}: {e}",
            inputs.camera_index
        ))
    })?;
    let resolution = Resolution::new(inputs.x, inputs.y);
    let camera_format = CameraFormat::new(resolution, fourcc, inputs.fps);
    Ok((inputs.camera_index, camera_format))
}

pub(crate) async fn screenshot_exact(
    Json(inputs): Json<CameraInputs>,
) -> Result<StreamPayload, ActionError> {
    let (camera_index, camera_format) = camera_format(inputs)?;

    run_camera_screenshot(
        camera_index,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::Exact(camera_format)),
    )
}

pub(crate) async fn screenshot_closest(
    Json(inputs): Json<CameraInputs>,
) -> Result<StreamPayload, ActionError> {
    let (camera_index, camera_format) = camera_format(inputs)?;

    run_camera_screenshot(
        camera_index,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(camera_format)),
    )
}
