// Ascot axum.
use ascot_axum::actions::serial::SerialPayload;
use ascot_axum::actions::ActionError;
use ascot_axum::extract::State;

// Nokhwa library
use nokhwa::{
    query,
    utils::{frame_formats, ApiBackend, CameraIndex, CameraInfo, FrameFormat, Resolution},
};

use serde::Serialize;

use tracing::info;

use crate::{camera_error, InternalState};

#[derive(Serialize)]
pub(crate) struct ViewCamerasResponse {
    cameras: Vec<CameraInfo>,
}

// Not a computationally intensive route, just some matches.
pub(crate) async fn view_available_cameras(
) -> Result<SerialPayload<ViewCamerasResponse>, ActionError> {
    // Retrieve all cameras present on a system
    let cameras =
        query(ApiBackend::Auto).map_err(|e| camera_error(format!("No cameras found: {e}")))?;

    info!("There are {} available cameras.", cameras.len());

    for camera in &cameras {
        info!("{camera}");
    }

    Ok(SerialPayload::new(ViewCamerasResponse { cameras }))
}

#[derive(Serialize)]
pub(crate) struct CameraDataResponse {
    camera_index: CameraIndex,
    controls: Vec<String>,
    frame_formats: Vec<CameraFrameFormat>,
}

#[derive(Serialize)]
pub(crate) struct CameraFrameFormat {
    frame_format: FrameFormat,
    format_data: Vec<FormatData>,
}

#[derive(Serialize)]
pub(crate) struct FormatData {
    resolution: Resolution,
    fps: String,
}

pub(crate) async fn camera_info(
    State(state): State<InternalState>,
) -> Result<SerialPayload<CameraDataResponse>, ActionError> {
    let mut camera = state.camera.lock().await;
    let camera_index = camera.index().clone();

    // Get controls for a camera
    let controls = camera.camera_controls().map_err(|e| {
        camera_error(format!(
            "Error in retrieving controls for camera with index {camera_index}: {e}"
        ))
    })?;

    info!("Control for camera with index {camera_index}");

    // Convert controls into strings.
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
