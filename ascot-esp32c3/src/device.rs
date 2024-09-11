use ascot_library::device::{DeviceData, DeviceKind, DeviceSerializer};
use ascot_library::hazards::{Hazard, Hazards};
use ascot_library::route::{Route, RouteConfigs, RouteHazards};

use esp_idf_svc::http::server::{EspHttpConnection, Request};
use esp_idf_svc::io::EspIOError;

use heapless::Vec;

// Default main route for a device.
const DEFAULT_MAIN_ROUTE: &str = "/device";

// Maximum stack elements.
const MAXIMUM_ELEMENTS: usize = 16;

// Handler type
type Handler = dyn for<'r> Fn(Request<&mut EspHttpConnection<'r>>) -> core::result::Result<(), EspIOError>
    + Send
    + 'static;

/// A device action connects a server route with a device handler and,
/// optionally, with every possible hazards associated with the handler.
pub struct DeviceAction {
    // Route and hazards.
    pub(crate) route_hazards: RouteHazards,
    // Handler.
    pub(crate) handler: Box<Handler>,
}

impl DeviceAction {
    /// Creates a new [`DeviceAction`].
    pub fn no_hazards<F>(route: Route, function: F) -> Self
    where
        F: for<'r> Fn(Request<&mut EspHttpConnection<'r>>) -> core::result::Result<(), EspIOError>
            + Send
            + 'static,
    {
        Self::init(route, function, Hazards::init())
    }

    /// Creates a new [`DeviceAction`] with a single [`Hazard`].
    pub fn with_hazard<F>(route: Route, function: F, hazard: Hazard) -> Self
    where
        F: for<'r> Fn(Request<&mut EspHttpConnection<'r>>) -> core::result::Result<(), EspIOError>
            + Send
            + 'static,
    {
        let mut hazards = Hazards::init();
        hazards.add(hazard);

        Self::init(route, function, hazards)
    }

    /// Creates a new [`DeviceAction`] with [`Hazard`]s.
    pub fn with_hazards<F>(route: Route, function: F, input_hazards: &'static [Hazard]) -> Self
    where
        F: for<'r> Fn(Request<&mut EspHttpConnection<'r>>) -> core::result::Result<(), EspIOError>
            + Send
            + 'static,
    {
        let mut hazards = Hazards::init();
        input_hazards.iter().for_each(|hazard| {
            hazards.add(*hazard);
        });

        Self::init(route, function, hazards)
    }

    /// Checks whether a [`DeviceAction`] misses a specific [`Hazard`].
    pub fn miss_hazard(&self, hazard: Hazard) -> bool {
        !self.route_hazards.hazards.contains(hazard)
    }

    /// Checks whether a [`DeviceAction`] misses the given [`Hazard`]s.
    pub fn miss_hazards(&self, hazards: &'static [Hazard]) -> bool {
        !hazards
            .iter()
            .all(|hazard| self.route_hazards.hazards.contains(*hazard))
    }

    fn init<F>(route: Route, handler: F, hazards: Hazards) -> Self
    where
        F: for<'r> Fn(Request<&mut EspHttpConnection<'r>>) -> core::result::Result<(), EspIOError>
            + Send
            + 'static,
    {
        Self {
            route_hazards: RouteHazards::new(route, hazards),
            handler: Box::new(handler),
        }
    }
}

/// A general smart home device.
pub struct Device {
    // Kind.
    kind: DeviceKind,
    // Main device route.
    main_route: &'static str,
    // All device routes with their hazards and handlers.
    pub(crate) routes_data: Vec<DeviceAction, MAXIMUM_ELEMENTS>,
}

impl DeviceSerializer for Device {
    fn serialize_data(&self) -> DeviceData {
        let mut route_configs = RouteConfigs::init();
        for route_data in self.routes_data.iter() {
            route_configs.add(route_data.route_hazards.serialize_data());
        }

        DeviceData {
            kind: self.kind,
            main_route: self.main_route,
            route_configs,
        }
    }
}

impl Device {
    /// Creates a new [`Device`] instance.
    pub fn new(kind: DeviceKind) -> Self {
        Self {
            kind,
            main_route: DEFAULT_MAIN_ROUTE,
            routes_data: Vec::new(),
        }
    }

    /// Sets a new main route.
    pub fn main_route(mut self, main_route: &'static str) -> Self {
        self.main_route = main_route;
        self
    }

    /// Adds a [`DeviceAction`].
    pub fn add_action(mut self, device_action: DeviceAction) -> Self {
        let _ = self.routes_data.push(device_action);
        self
    }
}
