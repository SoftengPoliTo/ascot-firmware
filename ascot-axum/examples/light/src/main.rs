extern crate alloc;

mod dummy_light;

use alloc::sync::Arc;

use async_lock::Mutex;
use serde::{Deserialize, Serialize};

// Library.
use ascot_library::device::DeviceErrorKind;
use ascot_library::hazards::Hazard;
use ascot_library::input::Input;
use ascot_library::route::Route;

// Miscellaneous.
use ascot_axum::error::Error;
use ascot_axum::service::ServiceBuilder;

// Device.
use ascot_axum::extract::{Extension, Json, Path};

use ascot_axum::device::{DeviceAction, DeviceError, DevicePayload};
use ascot_axum::devices::light::Light;

// Server.
use ascot_axum::server::AscotServer;

// Fake light implementation
use dummy_light::DummyLight;

#[derive(Clone, Default)]
struct DeviceState(Arc<Mutex<DummyLight>>);

impl DeviceState {
    fn new(light: DummyLight) -> Self {
        Self(Arc::new(Mutex::new(light)))
    }
}

impl core::ops::Deref for DeviceState {
    type Target = Arc<Mutex<DummyLight>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for DeviceState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Serialize)]
struct LightOnResponse {
    brightness: f64,
    save_energy: bool,
}

async fn turn_light_on(
    Path((brightness, save_energy)): Path<(f64, bool)>,
    Extension(state): Extension<DeviceState>,
) -> Result<DevicePayload, DeviceError> {
    let mut light = state.lock().await;
    light.turn_light_on(brightness, save_energy);

    DevicePayload::new(LightOnResponse {
        brightness: light.brightness,
        save_energy: light.save_energy,
    })
}

#[derive(Deserialize)]
struct Inputs {
    brightness: f64,
    save_energy: bool,
}

// The `Json` or the `Form` parameter must be the last argument
// to the handler function, since it consumes the request body.
async fn turn_light_on_post(
    Extension(state): Extension<DeviceState>,
    Json(inputs): Json<Inputs>,
) -> Result<DevicePayload, DeviceError> {
    let mut light = state.lock().await;
    light.turn_light_on(inputs.brightness, inputs.save_energy);

    DevicePayload::new(LightOnResponse {
        brightness: light.brightness,
        save_energy: light.save_energy,
    })
}

async fn turn_light_off(
    Extension(state): Extension<DeviceState>,
) -> Result<DevicePayload, DeviceError> {
    state.lock().await.turn_light_off();
    Err(DeviceError::from_str(DeviceErrorKind::Wrong, "Get failed"))
}

async fn toggle(Extension(state): Extension<DeviceState>) -> Result<DevicePayload, DeviceError> {
    state.lock().await.toggle();
    Ok(DevicePayload::empty())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Configuration for the `PUT` turn light on route.
    let light_on_config = Route::put("/on").description("Turn light on.").inputs([
        Input::rangef64("brightness", (0., 20., 0.1, 0.)),
        Input::boolean("save-energy", false),
    ]);

    // Configuration for the `POST` turn light on route.
    let light_on_post_config = Route::post("/on").description("Turn light on.").inputs([
        Input::rangef64("brightness", (0., 20., 0.1, 0.)),
        Input::boolean("save-energy", false),
    ]);

    // Configuration for the turn light off route.
    let light_off_config = Route::put("/off").description("Turn light off.");

    // Configuration for the toggle route.
    let toggle_config = Route::put("/toggle").description("Toggle a light.");

    // A light device which is going to be run on the server.
    let device = Light::new(
        DeviceAction::with_hazard(light_on_config, turn_light_on, Hazard::FireHazard),
        DeviceAction::no_hazards(light_off_config, turn_light_off),
    )?
    .add_action(DeviceAction::no_hazards(toggle_config, toggle))?
    .add_action(DeviceAction::no_hazards(
        light_on_post_config,
        turn_light_on_post,
    ))?
    .state(DeviceState::new(DummyLight::default()))
    .build();

    // Run a discovery service and the device on the server.
    AscotServer::new(device)
        .run_service(ServiceBuilder::new("mdns-sd"))?
        .run()
        .await
}
