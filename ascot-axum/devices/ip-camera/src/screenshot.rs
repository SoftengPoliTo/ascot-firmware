use std::io::Cursor;
use std::sync::Arc;

// Ascot axum.
use ascot_axum::actions::stream::StreamPayload;
use ascot_axum::actions::ActionError;

use ascot_axum::extract::{Json, State};
use ascot_axum::header;

use flume::Receiver;

use image::ImageFormat;

// Nokhwa library
use nokhwa::{
    pixel_format::{RgbAFormat, RgbFormat},
    utils::{
        CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType, Resolution,
    },
    Buffer, CallbackCamera, Camera,
};

// Serde library.
use serde::Deserialize;

// Tracing library.
use tracing::info;

use crate::{camera_error, InternalState};

fn run_camera_screenshot(
    camera_index: CameraIndex,
    format: RequestedFormat,
    suffix_filename: &str,
) -> Result<StreamPayload, ActionError> {
    let mut camera = Camera::new(camera_index.clone(), format).map_err(|e| {
        camera_error(format!(
            "Error in creating a camera with index {camera_index}: {e}"
        ))
    })?;

    // Open camera stream
    camera.open_stream().map_err(|e| {
        camera_error(format!(
            "Error in opening the stream camera with index {camera_index}: {e}"
        ))
    })?;

    // Retrieve a camera frame
    let frame = camera.frame().map_err(|e| {
        camera_error(format!(
            "Error in retrieving a frame for camera with index {camera_index}: {e}"
        ))
    })?;

    // Stop camera stream.
    camera.stop_stream().map_err(|e| {
        camera_error(format!(
            "Error in stopping a stream for camera with index {camera_index}: {e}"
        ))
    })?;

    info!("Capture camera screenshot of size {}", frame.buffer().len());

    // Decode the frame and save its content into an image buffer
    let decoded_frame = frame
        .decode_image::<RgbAFormat>()
        .map_err(|e| camera_error(format!("Error in decoding a frame for {camera_index}: {e}")))?;

    info!(
        "Decoded frame: {}x{} {}",
        decoded_frame.width(),
        decoded_frame.height(),
        decoded_frame.len()
    );

    // Convert the image buffer into a `png` image
    let mut cursor = Cursor::new(Vec::new());
    decoded_frame
        .write_to(&mut cursor, ImageFormat::Png)
        .map_err(|e| {
            camera_error(format!(
                "Error in converting the image buffer into `png` for {camera_index}: {e}"
            ))
        })?;

    // Retrieve raw data consuming the cursor
    let raw_data = cursor.into_inner();
    let raw_data_len = raw_data.len();

    info!("Image size {}", raw_data_len);

    let headers = [
        (header::CONTENT_TYPE, "image/png"),
        (header::CONTENT_LENGTH, &format!("{}", raw_data_len)),
    ];

    Ok(StreamPayload::new(headers, raw_data))
}

pub(crate) async fn screenshot_random(
    State(state): State<InternalState>,
) -> Result<StreamPayload, ActionError> {
    let camera_index = state.camera.lock().await;

    run_camera_screenshot(
        camera_index.clone(),
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::None),
        "none",
    )
}

/*pub(crate) async fn screenshot_absolute_resolution(
    State(state): State<InternalState>,
) -> Result<StreamPayload, ActionError> {
    run_camera_screenshot(
        state,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution),
        "absolute-resolution",
    )
    .await
}

pub(crate) async fn screenshot_absolute_framerate(
    State(state): State<InternalState>,
) -> Result<StreamPayload, ActionError> {
    run_camera_screenshot(
        state,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate),
        "absolute-framerate",
    )
    .await
}

#[derive(Deserialize)]
pub(crate) struct CameraResolution {
    x: u32,
    y: u32,
}

pub(crate) async fn screenshot_highest_resolution(
    State(state): State<InternalState>,
    Json(inputs): Json<CameraResolution>,
) -> Result<StreamPayload, ActionError> {
    let resolution = Resolution::new(inputs.x, inputs.y);

    run_camera_screenshot(
        state,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::HighestResolution(resolution)),
        "highest-resolution",
    )
    .await
}

#[derive(Deserialize)]
pub(crate) struct CameraFramerate {
    fps: u32,
}

pub(crate) async fn screenshot_highest_framerate(
    State(state): State<InternalState>,
    Json(inputs): Json<CameraFramerate>,
) -> Result<StreamPayload, ActionError> {
    run_camera_screenshot(
        state,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::HighestFrameRate(inputs.fps)),
        "highest-framerate",
    )
    .await
}

#[derive(Deserialize)]
pub(crate) struct CameraInputs {
    x: u32,
    y: u32,
    fps: u32,
    fourcc: String,
}

#[inline]
fn camera_format(inputs: CameraInputs) -> Result<CameraFormat, ActionError> {
    let fourcc = inputs
        .fourcc
        .parse::<FrameFormat>()
        .map_err(|e| camera_error(format!("Wrong fourcc value: {e}",)))?;
    let resolution = Resolution::new(inputs.x, inputs.y);
    Ok(CameraFormat::new(resolution, fourcc, inputs.fps))
}

pub(crate) async fn screenshot_exact(
    State(state): State<InternalState>,
    Json(inputs): Json<CameraInputs>,
) -> Result<StreamPayload, ActionError> {
    let camera_format = camera_format(inputs)?;

    run_camera_screenshot(
        state,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::Exact(camera_format)),
        "exact",
    )
    .await
}

pub(crate) async fn screenshot_closest(
    State(state): State<InternalState>,
    Json(inputs): Json<CameraInputs>,
) -> Result<StreamPayload, ActionError> {
    let camera_format = camera_format(inputs)?;

    run_camera_screenshot(
        state,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(camera_format)),
        "closest",
    )
    .await
}*/
