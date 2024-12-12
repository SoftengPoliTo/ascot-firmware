use std::io::Cursor;

// Ascot axum.
use ascot_axum::actions::stream::StreamPayload;
use ascot_axum::actions::ActionError;

use ascot_axum::extract::{Json, State};
use ascot_axum::header;

use image::ImageFormat;

// Nokhwa library
use nokhwa::{
    pixel_format::{RgbAFormat, RgbFormat},
    utils::{
        CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType, Resolution,
    },
    Camera,
};

// Serde library.
use serde::Deserialize;

// Tracing library.
use tracing::info;

use crate::{camera_error, InternalState};

async fn run_camera_screenshot(
    state: InternalState,
    format: RequestedFormat<'_>,
) -> Result<StreamPayload, ActionError> {
    let camera_index = state.camera.lock().await;

    /*let (tx, rx) = tokio::sync::oneshot::channel();

    rayon::spawn(move || {
        // some potentially expensive work here, including serialization
        let mut buf = Vec::with_capacity(16 * 4096);
        loop {
            // potentially expensive work here
            // then write chunk of calculated data into buf

            if buf.len() >= 16 * 4096 {
                tx.send(std::mem::replace(&mut buf, Vec::with_capacity(16 * 4096)));
            }
        }

        tx.send(buf);
    });*/

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

    // Discard at least 30 camera frame before sending the correct one.
    for _ in 0..30 {
        camera.frame().map_err(|e| {
            camera_error(format!(
                "Error in retrieving a frame for camera with index {camera_index}: {e}"
            ))
        })?;
    }

    // This also allows to focus in the lens.
    let frame = camera.frame().map_err(|e| {
        camera_error(format!(
            "Error in retrieving a frame for camera with index {camera_index}: {e}"
        ))
    })?;

    info!("Capture camera screenshot of size {}", frame.buffer().len());

    // Stop camera stream.
    camera.stop_stream().map_err(|e| {
        camera_error(format!(
            "Error in stopping a stream for camera with index {camera_index}: {e}"
        ))
    })?;

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

    // Convert the image buffer into a `png` image, and returns a bytes buffer
    let mut bytes: Vec<u8> = Vec::new();
    decoded_frame
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .map_err(|e| {
            camera_error(format!(
                "Error in converting the image buffer into `png` for {camera_index}: {e}"
            ))
        })?;

    info!("Image size {}", bytes.len());

    let headers = [(header::CONTENT_TYPE, "image/png")];

    Ok(StreamPayload::from_headers_reader(
        headers,
        Cursor::new(bytes),
    ))
}

pub(crate) async fn screenshot_random(
    State(state): State<InternalState>,
) -> Result<StreamPayload, ActionError> {
    run_camera_screenshot(
        state,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::None),
    )
    .await
}

pub(crate) async fn screenshot_absolute_resolution(
    State(state): State<InternalState>,
) -> Result<StreamPayload, ActionError> {
    run_camera_screenshot(
        state,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution),
    )
    .await
}

pub(crate) async fn screenshot_absolute_framerate(
    State(state): State<InternalState>,
) -> Result<StreamPayload, ActionError> {
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
) -> Result<StreamPayload, ActionError> {
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
) -> Result<StreamPayload, ActionError> {
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
    )
    .await
}
