use std::collections::{HashMap, HashSet};

use serde::{ser::SerializeMap, Serialize, Serializer};
use serde_json::{Error, Value};

use crate::action::Action;

/// A `[Device]` kind.
#[derive(Debug, Serialize)]
pub enum DeviceKind {
    /// Light.
    Light,
}

// Default main route for a device.
const DEFAULT_MAIN_ROUTE: &str = "/device";

/// A general smart home device.
#[derive(Debug)]
pub struct Device {
    // Kind.
    kind: DeviceKind,
    // Main server route for device actions.
    pub(crate) main_route: &'static str,
    // All device actions.
    actions: HashSet<Action>,
}

// Serialize route data.
#[derive(Debug, Serialize)]
struct RouteSerialize {
    // Final route obtained associating the main device route with the action
    // route.
    route: String,
    // Route description.
    description: Option<&'static str>,
}

impl Serialize for Device {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;

        // Serialize device kind.
        map.serialize_entry("kind", &self.kind)?;

        let mut routes = HashMap::new();
        for action in self.actions.iter() {
            let route_data = RouteSerialize {
                route: format!("{}/{}", self.main_route, action.route),
                description: action.description,
            };

            routes.insert(action.route, route_data);
        }

        // Serialize routes.
        map.serialize_entry("routes", &routes)?;
        map.end()
    }
}

impl Device {
    /// Creates a new `[Device]` instance.
    pub fn new(kind: DeviceKind) -> Self {
        Self {
            kind,
            main_route: DEFAULT_MAIN_ROUTE,
            actions: HashSet::new(),
        }
    }

    /// Sets a new main route.
    pub fn main_route(mut self, main_route: &'static str) -> Self {
        self.main_route = main_route;
        self
    }

    /// Adds an action.
    pub fn add_action(mut self, device_action: Action) -> Self {
        self.actions.insert(device_action);
        self
    }

    /// Returns an iterator over actions.
    pub fn actions(&self) -> std::collections::hash_set::Iter<Action> {
        self.actions.iter()
    }

    // Adds an action with reference.
    fn add_action_as_reference(&mut self, device_action: Action) {
        self.actions.insert(device_action);
    }

    // Returns a device as a json format.
    pub(crate) fn json(&self) -> Result<Value, Error> {
        serde_json::to_value(self)
    }
}

/// A light device.
pub mod light {
    use std::collections::HashSet;

    use super::{Action, Device, DeviceKind};

    // The default main route for a light.
    const LIGHT_MAIN_ROUTE: &str = "/light";

    /// A smart home light.
    ///
    /// The default main server route for a light is `light`.
    /// If a smart home needs more lights, each light **MUST** provide a
    /// **different** main route in order to be registered.
    pub struct Light {
        // Turn light on action.
        turn_light_on: Action,
        // Turn light off action.
        turn_light_off: Action,
        // Main server route for light actions.
        main_route: &'static str,
        // Other light actions.
        actions: HashSet<Action>,
    }

    impl Light {
        /// Creates a new `[Light]` instance.
        pub fn new(turn_light_on: Action, turn_light_off: Action) -> Self {
            Self {
                turn_light_on,
                turn_light_off,
                main_route: LIGHT_MAIN_ROUTE,
                actions: HashSet::new(),
            }
        }

        /// Sets a new main route.
        pub fn main_route(mut self, main_route: &'static str) -> Self {
            self.main_route = main_route;
            self
        }

        /// Adds a light action.
        pub fn add_action(mut self, light_action: Action) -> Self {
            self.actions.insert(light_action);
            self
        }

        /// Builds a new `[Device]`.
        pub fn build(self) -> Device {
            let mut device = Device::new(DeviceKind::Light)
                .main_route(self.main_route)
                .add_action(self.turn_light_on)
                .add_action(self.turn_light_off);

            for action in self.actions {
                device.add_action_as_reference(action);
            }

            device
        }
    }
}
