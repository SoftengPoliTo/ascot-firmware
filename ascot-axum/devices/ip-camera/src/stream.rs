use std::io::Cursor;

// Ascot axum.
use ascot_axum::actions::stream::StreamPayload;
use ascot_axum::actions::ActionError;

use ascot_axum::extract::State;
use ascot_axum::header;

use image::ImageFormat;

use nokhwa::{
    pixel_format::{RgbAFormat, RgbFormat},
    utils::RequestedFormat,
    Camera,
};

use tokio::sync::mpsc::unbounded_channel;
use tokio::task::spawn_blocking;

use tracing::{error, info};

use crate::InternalState;

fn thread_error<T: std::fmt::Display>(msg: &str, e: T) {
    error!("{msg}");
    error!("{e}");
}

// To avoid busy resources we need a total time of 200ms.
pub(crate) async fn show_camera_stream(
    State(state): State<InternalState>,
) -> Result<StreamPayload, ActionError> {
    let camera = state.camera.lock().await.clone();
    let format = RequestedFormat::new::<RgbFormat>(camera.format_type);

    let (tx, rx) = unbounded_channel::<Result<Vec<u8>, &'static str>>();

    spawn_blocking(move || {
        let mut camera = match Camera::new(camera.index, format) {
            Ok(camera) => camera,
            Err(e) => {
                thread_error("Error in creating the camera.", e);
                return;
            }
        };

        // If a request is sent with a high throttle, we should wait for
        // an amount of time to access to the camere because of delay.
        std::thread::sleep(std::time::Duration::from_millis(200));

        // Open camera stream
        if let Err(e) = camera.open_stream() {
            thread_error("Error in opening a camera stream.", e);
            return;
        }

        while !tx.is_closed() {
            // Retrieve a camera frame.
            //
            // If a frame cannot be retrieved, because the camera has been
            // disconnected for example, return the thread. The stream will
            // stop at the last frame.
            let frame = match camera.frame() {
                Ok(frame) => frame,
                Err(e) => {
                    thread_error("Error retrieving a camera frame.", e);
                    return;
                }
            };

            info!("Capture camera screenshot of size {}", frame.buffer().len());

            // Decode the frame as RGBA format.
            //
            // If an error occurs, pass at the next loop cycle, discarding the
            // frame.
            let decoded_frame = match frame.decode_image::<RgbAFormat>() {
                Ok(frame) => frame,
                Err(e) => {
                    thread_error("Error decoding a camera frame.", e);
                    continue;
                }
            };

            info!(
                "Decoded frame: {}x{} {}",
                decoded_frame.width(),
                decoded_frame.height(),
                decoded_frame.len()
            );

            let mut bytes = Vec::new();

            // Convert the decoded frame into a `png` image and returns a
            // bytes buffer.
            //
            // If an error occurs, pass at the next loop cycle, discarding the
            // frame.
            if let Err(e) = decoded_frame.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png) {
                thread_error("Error converting frame format into a `png` image", e);
                continue;
            }

            info!("Image size {}", bytes.capacity());

            // If we do not add this check, we could send data to a
            // non-existent channel.
            if !tx.is_closed() {
                if let Err(e) = tx.send(Ok(bytes)) {
                    error!("Error sending image {e}");
                }
            }
        }
    });

    let headers = [(header::CONTENT_TYPE, "image/png")];
    Ok(StreamPayload::from_headers_stream(
        headers,
        tokio_stream::wrappers::UnboundedReceiverStream::new(rx),
    ))
}
