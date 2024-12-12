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

use tracing::info;

use crate::{thread_error, thread_with_error, InternalState};

pub(crate) async fn show_cameras_info(
    State(state): State<InternalState>,
) -> Result<StreamPayload, ActionError> {
    let current_index = state.camera.lock().await;
    let format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::None);
    let index = current_index.clone();

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<
        Result<Vec<u8>, tokio::sync::mpsc::error::SendError<Vec<u8>>>,
    >();

    tokio::spawn(async move {
        let mut camera = Camera::new(index.clone(), format)
            .map_err(|e| thread_with_error("Impossible to create camera", &index, e))
            .unwrap();

        // Open camera stream
        camera
            .open_stream()
            .map_err(|e| thread_with_error("Impossible to open a stream on camera", &index, e))
            .unwrap();

        loop {
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

            tx.send(Ok(bytes))
                .map_err(|_| thread_error("Impossible to send a `png` image for camera", &index))
                .unwrap();
        }
    });

    let headers = [(header::CONTENT_TYPE, "image/png")];

    Ok(StreamPayload::from_headers_stream(
        headers,
        tokio_stream::wrappers::UnboundedReceiverStream::new(rx),
    ))
}

/*info!("Start");
// this will go out of scope either when this function returns (which it never does)
// or hyper drops the future because the client closed the connection
let _guard = Guard;

// Retrieves camera images.
let mut send_task = tokio::spawn(async move { loop {} });

// Checks browser connection. This task waits forever.
let mut recv_task = tokio::spawn(async move { std::future::pending::<()>().await });

// If any one of the tasks run to completion, we abort the other.
tokio::select! {
    _ = &mut send_task => recv_task.abort(),
    _ = &mut recv_task => send_task.abort(),
};*/

struct Guard;

impl Drop for Guard {
    fn drop(&mut self) {
        info!("Dropping `Guard` when a connection is closed!")
    }
}
