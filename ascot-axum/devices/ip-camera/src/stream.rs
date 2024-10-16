use std::io::Cursor;

// Ascot axum.
use ascot_axum::actions::stream::StreamPayload;
use ascot_axum::actions::ActionError;

use ascot_axum::extract::State;
use ascot_axum::header;

use image::ImageFormat;

use nokhwa::{
    pixel_format::{RgbAFormat, RgbFormat},
    utils::{RequestedFormat, RequestedFormatType},
    Camera,
};

use tokio::sync::mpsc::{error::SendError, unbounded_channel};
use tokio::task::spawn_blocking;

use tracing::info;

use crate::{thread_with_error, InternalState};

// To avoid busy resources we need a total time of 200ms.
pub(crate) async fn show_camera_stream(
    State(state): State<InternalState>,
) -> Result<StreamPayload, ActionError> {
    let current_index = state.camera.lock().await;
    let format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::None);
    let index = current_index.clone();

    let (tx, rx) = unbounded_channel::<Result<Vec<u8>, SendError<()>>>();

    spawn_blocking(move || {
        let mut camera = Camera::new(index.clone(), format)
            .map_err(|e| thread_with_error("Impossible to create camera", &index, e))
            .unwrap();

        // If a request is sent with a high throttle, we should wait for
        // an amount of time to clean up old resources and
        // obtain the access to the camera.
        std::thread::sleep(std::time::Duration::from_millis(300));

        // Open camera stream
        camera
            .open_stream()
            .map_err(|e| thread_with_error("Impossible to open a stream on camera", &index, e))
            .unwrap();

        while !tx.is_closed() {
            // This also allows to focus in the lens.
            let frame = camera
                .frame()
                .map_err(|e| {
                    thread_with_error("Impossible to retrieve a frame for camera", &index, e)
                })
                .unwrap();

            info!("Capture camera screenshot of size {}", frame.buffer().len());

            // Decode the frame and save its content into an image buffer
            let decoded_frame = frame
                .decode_image::<RgbAFormat>()
                .map_err(|e| {
                    thread_with_error("Impossible to decode a frame for camera", &index, e)
                })
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

            // If we do not add this check, we could send a `png` image to a
            // non-existent channel.
            if !tx.is_closed() {
                tx.send(Ok(bytes)).unwrap();
            }
        }
    });

    let headers = [(header::CONTENT_TYPE, "image/png")];
    Ok(StreamPayload::from_headers_stream(
        headers,
        tokio_stream::wrappers::UnboundedReceiverStream::new(rx),
    ))
}
