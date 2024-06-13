use ascot_library::device::DeviceKind;
use ascot_library::hazards::Hazard;

use crate::device::{Device, DeviceAction, DeviceBuilder};
use crate::error::{Error, ErrorKind, Result};

// The default main route for a light.
const LIGHT_MAIN_ROUTE: &str = "/light";

// Mandatory actions hazards.
const TURN_LIGHT_ON: Hazard = Hazard::FireHazard;

// Allowed hazards.
const ALLOWED_HAZARDS: &[Hazard] = &[Hazard::FireHazard, Hazard::ElectricEnergyConsumption];

/// A smart home light.
///
/// The default server main route for a light is `light`.
///
/// If a smart home needs more lights, each light **MUST** provide a
/// **different** main route in order to be registered.
pub struct Light<S>
where
    S: Clone + Send + Sync + 'static,
{
    // Main server route for light routes.
    main_route: &'static str,
    // Light state.
    state: Option<S>,
    // Device.
    device: Device<S>,
    // Allowed light hazards.
    allowed_hazards: &'static [Hazard],
}

impl<S> DeviceBuilder<S> for Light<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn into_device(self) -> Device<S> {
        self.device
            .main_route(self.main_route)
            .internal_state(self.state)
    }
}

impl<S> Light<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Creates a new [`Light`] instance.
    pub fn new(turn_light_on: DeviceAction, turn_light_off: DeviceAction) -> Result<Self> {
        // Raise an error whether turn light_on does not contain a
        // fire hazard.
        if turn_light_on.miss_hazard(TURN_LIGHT_ON) {
            return Err(Error::new(
                ErrorKind::Light,
                "No fire hazard for the `turn_light_on` route",
            ));
        }

        // Create a new device.
        let device = Device::new(DeviceKind::Light)
            .add_action(turn_light_on)
            .add_action(turn_light_off);

        Ok(Self {
            main_route: LIGHT_MAIN_ROUTE,
            device,
            state: None,
            allowed_hazards: ALLOWED_HAZARDS,
        })
    }

    /// Sets a new main route.
    pub fn main_route(mut self, main_route: &'static str) -> Self {
        self.main_route = main_route;
        self
    }

    /// Adds an additional action for a [`Light`].
    pub fn add_action(mut self, light_action: DeviceAction) -> Result<Self> {
        // Return an error if action hazards are not a subset of allowed hazards.
        for hazard in light_action.hazards.iter() {
            if !self.allowed_hazards.contains(hazard) {
                return Err(Error::new(
                    ErrorKind::Light,
                    format!("{hazard} hazard is not allowed for light"),
                ));
            }
        }

        self.device = self.device.add_action(light_action);

        Ok(self)
    }

    /// Sets a state for a [`Light`].
    pub fn state(mut self, state: S) -> Self {
        self.state = Some(state);
        self
    }

    /// Builds a new [`Device`].
    pub fn build(self) -> Device<S> {
        self.into_device()
    }
}
