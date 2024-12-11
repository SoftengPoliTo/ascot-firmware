use tracing::info;

/*fn camera_stream(
    index: &str,
    frame_format_type: &str,
    frame_format_option: Option<&str>,
    display_frame: bool,
) {
    // Camera index.
    let index = CameraIndex::String(index.into());

    // Camera frame format.
    let requested_format = make_requested_format(frame_format_type, frame_format_option).unwrap();

    /*if display_frame {
        // Sender and receiver channels.
        let (sender, receiver) = flume::unbounded();
        let (sender, receiver) = (Arc::new(sender), Arc::new(receiver));
        let sender_clone = sender.clone();

        // Create a thread for sending the camera frame.
        let mut camera = CallbackCamera::new(index, requested_format, move |buf| {
            sender_clone.send(buf).expect("Error sending frame!!!!");
        })
        .unwrap();

        // Retrieve camera info.
        let camera_info = camera.info().clone();
        // Retrieve camera format.
        let format = camera.camera_format().unwrap();

        // Open camera stream.
        camera.open_stream().unwrap();
    } else {
        // Define a thread which captures a frame
        let mut cb = CallbackCamera::new(index, requested_format, |buf| {
            info!("Captured frame of size {}", buf.buffer().len());
        })
        .unwrap();

        // Open a camera stream
        cb.open_stream().unwrap();
        // Run the stream endlessly
        loop {}
    }*/
}*/

pub(crate) async fn show_cameras_info() {
    info!("Start");
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
    };
}

struct Guard;

impl Drop for Guard {
    fn drop(&mut self) {
        info!("Dropping `Guard` when a connection is closed!")
    }
}
