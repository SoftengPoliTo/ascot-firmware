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
    utils::{CameraFormat, FrameFormat, RequestedFormat, RequestedFormatType, Resolution},
    Camera,
};

// Serde library.
use serde::Deserialize;

// Tracing library.
use tracing::info;

use crate::{camera_error, camera_index_error, thread_error, thread_with_error, InternalState};

async fn run_camera_screenshot(
    state: InternalState,
    format: RequestedFormat<'static>,
) -> Result<StreamPayload, ActionError> {
    let current_index = state.camera.lock().await;
    let index = current_index.clone();

    let (tx, rx) = tokio::sync::oneshot::channel();

    rayon::spawn(move || {
        let mut camera = Camera::new(index.clone(), format)
            .map_err(|e| thread_with_error("Impossible to create camera", &index, e))
            .unwrap();

        // Open camera stream
        camera
            .open_stream()
            .map_err(|e| thread_with_error("Impossible to open a stream on camera", &index, e))
            .unwrap();

        // Discard at least 10 camera frame before sending the correct one.
        for _ in 0..10 {
            camera
                .frame()
                .map_err(|e| {
                    thread_with_error("Impossible to retrieve a frame for camera", &index, e)
                })
                .unwrap();
        }

        // This also allows to focus in the lens.
        let frame = camera
            .frame()
            .map_err(|e| thread_with_error("Impossible to retrieve a frame for camera", &index, e))
            .unwrap();

        info!("Capture camera screenshot of size {}", frame.buffer().len());

        // Stop camera stream.
        camera
            .stop_stream()
            .map_err(|e| thread_with_error("Impossible to stop a stream for camera", &index, e))
            .unwrap();

        // Decode the frame and save its content into an image buffer
        let decoded_frame = frame
            .decode_image::<RgbAFormat>()
            .map_err(|e| thread_with_error("Impossible to decode a frame for camera", &index, e))
            .unwrap();

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
            .map_err(|e| {
                thread_with_error("Impossible to write a `png` image for camera", &index, e)
            })
            .unwrap();

        info!("Image size {}", bytes.len());

        tx.send(bytes)
            .map_err(|_| thread_error("Impossible to send a `png` image for camera", &index))
            .unwrap();
    });

    let headers = [(header::CONTENT_TYPE, "image/png")];
    let buf = rx.await.map_err(|e| {
        camera_index_error(
            "Impossible to retrieve the `png` image from thread for camera",
            &current_index,
            e,
        )
    })?;

    Ok(StreamPayload::from_headers_reader(
        headers,
        Cursor::new(buf),
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

fn camera_format(inputs: CameraInputs) -> Result<CameraFormat, ActionError> {
    let fourcc = inputs
        .fourcc
        .parse::<FrameFormat>()
        .map_err(|e| camera_error("Wrong fourcc value", e))?;
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
