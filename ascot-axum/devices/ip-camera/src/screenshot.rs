use std::io::Cursor;

// Ascot axum.
use ascot_axum::actions::error::ErrorResponse;
use ascot_axum::actions::stream::StreamResponse;

use ascot_axum::extract::{Json, State};
use ascot_axum::header;

use image::ImageFormat;

// Nokhwa library
use nokhwa::{
    pixel_format::{RgbAFormat, RgbFormat},
    utils::{CameraFormat, FrameFormat, RequestedFormat, RequestedFormatType, Resolution},
    Camera,
};

// Serde library.
use serde::Deserialize;

use tokio::task::spawn_blocking;

// Tracing library.
use tracing::info;

use crate::{camera_error, InternalState};

async fn run_camera_screenshot(
    state: InternalState,
    format: RequestedFormat<'static>,
) -> Result<StreamResponse, ErrorResponse> {
    let camera = state.camera.lock().await;
    let index = camera.index.clone();

    let buffer = spawn_blocking(move || {
        let mut camera = Camera::new(index.clone(), format)
            .map_err(|e| camera_error("Impossible to create camera", e))?;

        // Open camera stream
        camera
            .open_stream()
            .map_err(|e| camera_error("Impossible to open a stream on camera", e))?;

        // Discard at least 10 camera frame before sending the correct one
        // in order to focus in lens.
        for _ in 0..10 {
            camera
                .frame()
                .map_err(|e| camera_error("Impossible to retrieve a frame for camera", e))?;
        }

        // This also allows to focus in the lens.
        let frame = camera
            .frame()
            .map_err(|e| camera_error("Impossible to retrieve a frame for camera", e))?;

        info!("Capture camera screenshot of size {}", frame.buffer().len());

        // Stop camera stream.
        camera
            .stop_stream()
            .map_err(|e| camera_error("Impossible to stop a stream for camera", e))?;

        // Decode the frame and save its content into an image buffer
        let decoded_frame = frame
            .decode_image::<RgbAFormat>()
            .map_err(|e| camera_error("Impossible to decode a frame for camera", e))?;

        info!(
            "Decoded frame: {}x{} {}",
            decoded_frame.width(),
            decoded_frame.height(),
            decoded_frame.len()
        );

        // Convert the image buffer into a `png` image, and returns a bytes buffer
        let mut bytes = Vec::new();
        decoded_frame
            .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
            .map_err(|e| camera_error("Impossible to write a `png` image for camera", e))?;

        info!("Image size {}", bytes.len());

        Ok(bytes)
    })
    .await
    .map_err(|e| camera_error("Impossible to retrieve the `png` image from thread", e))?;

    let headers = [(header::CONTENT_TYPE, "image/png")];
    Ok(StreamResponse::from_headers_reader(
        headers,
        Cursor::new(buffer?),
    ))
}

pub(crate) async fn screenshot_random(
    State(state): State<InternalState>,
) -> Result<StreamResponse, ErrorResponse> {
    run_camera_screenshot(
        state,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::None),
    )
    .await
}

pub(crate) async fn screenshot_absolute_resolution(
    State(state): State<InternalState>,
) -> Result<StreamResponse, ErrorResponse> {
    run_camera_screenshot(
        state,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution),
    )
    .await
}

pub(crate) async fn screenshot_absolute_framerate(
    State(state): State<InternalState>,
) -> Result<StreamResponse, ErrorResponse> {
    run_camera_screenshot(
        state,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate),
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
) -> Result<StreamResponse, ErrorResponse> {
    let resolution = Resolution::new(inputs.x, inputs.y);

    run_camera_screenshot(
        state,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::HighestResolution(resolution)),
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
) -> Result<StreamResponse, ErrorResponse> {
    run_camera_screenshot(
        state,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::HighestFrameRate(inputs.fps)),
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

#[allow(clippy::result_large_err)]
fn camera_format(inputs: CameraInputs) -> Result<CameraFormat, ErrorResponse> {
    let CameraInputs { x, y, fps, fourcc } = inputs;
    let fourcc = fourcc
        .parse::<FrameFormat>()
        .map_err(|e| camera_error("Wrong fourcc value", e))?;
    let resolution = Resolution::new(x, y);
    Ok(CameraFormat::new(resolution, fourcc, fps))
}

pub(crate) async fn screenshot_exact(
    State(state): State<InternalState>,
    Json(inputs): Json<CameraInputs>,
) -> Result<StreamResponse, ErrorResponse> {
    let camera_format = camera_format(inputs)?;

    run_camera_screenshot(
        state,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::Exact(camera_format)),
    )
    .await
}

pub(crate) async fn screenshot_closest(
    State(state): State<InternalState>,
    Json(inputs): Json<CameraInputs>,
) -> Result<StreamResponse, ErrorResponse> {
    let camera_format = camera_format(inputs)?;

    run_camera_screenshot(
        state,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(camera_format)),
    )
    .await
}
