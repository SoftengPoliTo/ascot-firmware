use ascot_library::device::DeviceKind;
use ascot_library::hazards::Hazard;

use axum::handler::Handler;
use heapless::FnvIndexSet;

use crate::device::{Device, DeviceAction, DeviceBuilder};
use crate::error::{Error, ErrorKind, Result};
use crate::MAXIMUM_ELEMENTS;

// The default main route for a fridge.
const FRIDGE_MAIN_ROUTE: &str = "/fridge";

// Mandatory actions hazards.
const INCREASE_TEMPERATURE: &'static [Hazard] =
    &[Hazard::ElectricEnergyConsumption, Hazard::SpoiledFood];
const DECREASE_TEMPERATURE: Hazard = Hazard::ElectricEnergyConsumption;

// Allowed hazards.
const ALLOWED_HAZARDS: &'static [Hazard] =
    &[Hazard::ElectricEnergyConsumption, Hazard::SpoiledFood];

// Mandatory fridge actions.
#[derive(Debug, PartialEq, Eq, Hash)]
enum Actions {
    IncreaseTemperature,
    DecreaseTemperature,
}

/// A smart home fridge.
///
/// The default server main route for a fridge is `fridge`.
///
/// If a smart home needs more fridges, each fridge **MUST** provide a
/// **different** main route in order to be registered.
pub struct Fridge<S>
where
    S: Clone + Send + Sync + 'static,
{
    // Main server route for fridge routes.
    main_route: &'static str,
    // Fridge state.
    state: Option<S>,
    // Device.
    device: Device<S>,
    // Mandatory fridge actions.
    mandatory_actions: FnvIndexSet<Actions, MAXIMUM_ELEMENTS>,
    // Allowed fridge hazards.
    allowed_hazards: &'static [Hazard],
}

impl<S> DeviceBuilder<S> for Fridge<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn into_device(self) -> Device<S> {
        self.device
            .main_route(self.main_route)
            .internal_state(self.state)
    }
}

impl<S> Fridge<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Creates a new [`Fridge`] instance.
    pub fn new() -> Self {
        // Create a new device.
        let device = Device::new(DeviceKind::Fridge);

        // Define mandatory actions.
        let mut mandatory_actions = FnvIndexSet::new();
        let _ = mandatory_actions.insert(Actions::IncreaseTemperature);
        let _ = mandatory_actions.insert(Actions::DecreaseTemperature);

        Self {
            main_route: FRIDGE_MAIN_ROUTE,
            device,
            state: None,
            mandatory_actions,
            allowed_hazards: ALLOWED_HAZARDS,
        }
    }

    /// Sets a new main route.
    pub fn main_route(mut self, main_route: &'static str) -> Self {
        self.main_route = main_route;
        self
    }

    /// Adds increase temperature action for a [`Fridge`].
    pub fn increase_temperature<H, T>(
        mut self,
        increase_temperature: DeviceAction<H, T>,
    ) -> Result<Self>
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        // Raise an error whether increase_temperature does not contain
        // electric energy consumption or spoiled food hazards.
        if increase_temperature.miss_hazards(INCREASE_TEMPERATURE) {
            return Err(Error::new(
                ErrorKind::Fridge,
                "No electric energy consumption or spoiled food hazards for the `increase_temperature` route",
            ));
        }

        self.device = self.device.add_action(increase_temperature);

        // Remove increase_temperature action from the list of actions to set.
        self.mandatory_actions.remove(&Actions::IncreaseTemperature);

        Ok(self)
    }

    /// Adds decrease temperature action for a [`Fridge`].
    pub fn decrease_temperature<H, T>(
        mut self,
        decrease_temperature: DeviceAction<H, T>,
    ) -> Result<Self>
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        // Raise an error whether decrease_temperature does not contain
        // electric energy consumption hazard.
        if decrease_temperature.miss_hazard(DECREASE_TEMPERATURE) {
            return Err(Error::new(
                ErrorKind::Fridge,
                "No electric energy consumption hazard for the `decrease_temperature` route",
            ));
        }

        self.device = self.device.add_action(decrease_temperature);

        // Remove decrease_temperature action from the list of actions to set.
        self.mandatory_actions.remove(&Actions::DecreaseTemperature);

        Ok(self)
    }

    /// Adds an additional action for a [`Fridge`].
    pub fn add_action<H, T>(mut self, fridge_action: DeviceAction<H, T>) -> Result<Self>
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        // Return an error if action hazards are not a subset of allowed hazards.
        for hazard in fridge_action.hazards.iter() {
            if !self.allowed_hazards.contains(hazard) {
                return Err(Error::new(
                    ErrorKind::Fridge,
                    format!("{hazard} hazard is not allowed for fridge"),
                ));
            }
        }

        self.device = self.device.add_action(fridge_action);

        Ok(self)
    }

    /// Sets a state for a [`Fridge`].
    pub fn state(mut self, state: S) -> Self {
        self.state = Some(state);
        self
    }

    /// Builds a new [`Device`].
    pub fn build(self) -> Result<Device<S>> {
        // Return an error if not all mandatory actions are set.
        if !self.mandatory_actions.is_empty() {
            return Err(Error::new(
                ErrorKind::Fridge,
                format!(
                    "The following mandatory actions are not set: {:?}",
                    self.mandatory_actions
                ),
            ));
        };

        Ok(self.into_device())
    }
}
