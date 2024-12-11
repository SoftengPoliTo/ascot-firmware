use std::io::Cursor;

// Ascot axum.
use ascot_axum::actions::stream::StreamPayload;
use ascot_axum::actions::ActionError;

use ascot_axum::extract::{Json, Path};
use ascot_axum::header;

use image::ImageFormat;

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
    suffix_filename: &str,
) -> Result<StreamPayload, ActionError> {
    // Create camera
    let mut camera = Camera::new(CameraIndex::Index(camera_index), format)
        .map_err(|e| camera_error(format!("Error in retrieving camera {camera_index}: {e}")))?;

    // Open camera stream.
    camera
        .open_stream()
        .map_err(|e| camera_error(format!("Error in opening a stream for {camera_index}: {e}")))?;

    // Retrieve camera frame as data buffer.
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

    // Decode the frame and save its content into an image buffer
    let decoded_frame = frame
        .decode_image::<RgbFormat>()
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
        (
            header::CONTENT_DISPOSITION,
            &format!("attachment; filename=\"screenshot-{suffix_filename}.png\""),
        ),
    ];

    Ok(StreamPayload::new(headers, raw_data))
}

/*#[derive(Deserialize)]
pub(crate) struct CameraInputIndex {
    camera_index: u32,
}*/

pub(crate) async fn screenshot_none(
    Path(camera_index): Path<u32>,
) -> Result<StreamPayload, ActionError> {
    run_camera_screenshot(
        camera_index,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::None),
        "none",
    )
}

pub(crate) async fn screenshot_absolute_resolution(
    Path(camera_index): Path<u32>,
) -> Result<StreamPayload, ActionError> {
    run_camera_screenshot(
        camera_index,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution),
        "absolute-resolution",
    )
}

pub(crate) async fn screenshot_absolute_framerate(
    Path(camera_index): Path<u32>,
) -> Result<StreamPayload, ActionError> {
    run_camera_screenshot(
        camera_index,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate),
        "absolute-framerate",
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
        "highest-resolution",
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
        "highest-framerate",
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
        "exact",
    )
}

pub(crate) async fn screenshot_closest(
    Json(inputs): Json<CameraInputs>,
) -> Result<StreamPayload, ActionError> {
    let (camera_index, camera_format) = camera_format(inputs)?;

    run_camera_screenshot(
        camera_index,
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(camera_format)),
        "closest",
    )
}
