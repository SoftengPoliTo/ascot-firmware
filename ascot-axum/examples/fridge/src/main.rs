extern crate alloc;

mod dummy_fridge;

use alloc::sync::Arc;

use ascot_axum::devices::fridge::Fridge;
use async_lock::Mutex;
use serde::{Deserialize, Serialize};

// Library.
use ascot_library::hazards::Hazard;
use ascot_library::input::Input;
use ascot_library::route::Route;

// Miscellaneous.
use ascot_axum::error::Error;
use ascot_axum::service::ServiceBuilder;

// Device.
use ascot_axum::extract::{Extension, Json, Path};

use ascot_axum::device::{DeviceAction, DeviceError, DevicePayload};

// Server.
use ascot_axum::server::AscotServer;

// Fake fridge implementation
use dummy_fridge::DummyFridge;

#[derive(Clone, Default)]
struct DeviceState(Arc<Mutex<DummyFridge>>);

impl DeviceState {
    fn new(fridge: DummyFridge) -> Self {
        Self(Arc::new(Mutex::new(fridge)))
    }
}

impl core::ops::Deref for DeviceState {
    type Target = Arc<Mutex<DummyFridge>>;

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
struct ChangeTempResponse {
    temperature: f64,
}

async fn increase_temperature(
    Path(increment): Path<f64>,
    Extension(state): Extension<DeviceState>,
) -> Result<DevicePayload, DeviceError> {
    let mut fridge = state.lock().await;
    fridge.increase_temperature(increment);

    DevicePayload::new(ChangeTempResponse {
        temperature: fridge.temperature,
    })
}

#[derive(Deserialize)]
struct Inputs {
    increment: f64,
}

async fn increase_temp_post(
    Extension(state): Extension<DeviceState>,
    Json(inputs): Json<Inputs>,
) -> Result<DevicePayload, DeviceError> {
    let mut fridge = state.lock().await;
    fridge.increase_temperature(inputs.increment);

    DevicePayload::new(ChangeTempResponse {
        temperature: fridge.temperature
    })
}

async fn decrease_temperature(
    Path(decrement): Path<f64>,
    Extension(state): Extension<DeviceState>,
) -> Result<DevicePayload, DeviceError> {
    let mut fridge = state.lock().await;
    fridge.decrease_temperature(decrement);

    DevicePayload::new(ChangeTempResponse {
        temperature: fridge.temperature,
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Configuration for the `PUT` increase temperature route.
    let increase_temp_config = Route::put("/increase-temperature")
        .description("Increase temperature.")
        .input(Input::rangef64("increment", (1., 4., 0.1, 2.)));

    // Configuration for the `POST` increase temperature route.
    let increase_temp_post_config = Route::post("/increase-temperature")
        .description("Increase temperature.")
        .input(Input::rangef64("increment", (1., 4., 0.1, 2.)));

    // Configuration for the `PUT` decrease temperature route.
    let decrease_temp_config = Route::put("/decrease-temperature")
        .description("Decrease temperature.")
        .input(Input::rangef64("decrement", (1., 4., 0.1, 2.)));

    // A fridge device which is going to be run on the server.
    let device = Fridge::new()
        .increase_temperature(DeviceAction::with_hazards(
            increase_temp_config,
            increase_temperature,
            &[Hazard::ElectricEnergyConsumption, Hazard::SpoiledFood],
        ))?
        .decrease_temperature(DeviceAction::with_hazards(
            decrease_temp_config,
            decrease_temperature,
            &[Hazard::ElectricEnergyConsumption],
        ))?
        .add_action(DeviceAction::no_hazards(
            increase_temp_post_config,
            increase_temp_post,
        ))?
        .state(DeviceState::new(DummyFridge::default()))
        .build()?;

    // Run a discovery service and the device on the server.
    AscotServer::new(device)
        .run_service(ServiceBuilder::new("mdns-sd"))?
        .run()
        .await
}
