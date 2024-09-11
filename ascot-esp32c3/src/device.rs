use core::fmt::Debug;
use core::marker::PhantomData;

use ascot_library::device::{DeviceData, DeviceKind, DeviceSerializer};
use ascot_library::hazards::{Hazard, Hazards};
use ascot_library::route::{Route, RouteConfigs, RouteHazards};

use esp_idf_svc::http::server::{EspHttpConnection, Request};

use heapless::Vec;

// Default main route for a device.
const DEFAULT_MAIN_ROUTE: &str = "/device";

// Maximum stack elements.
const MAXIMUM_ELEMENTS: usize = 16;

/// A device action connects a server route with a device handler and,
/// optionally, with every possible hazards associated with the handler.
pub struct DeviceAction<E, F>
where
    F: for<'r> Fn(Request<&mut EspHttpConnection<'r>>) -> anyhow::Result<(), E> + Send + 'static,
    E: Debug,
{
    // Route and hazards.
    pub(crate) route_hazards: RouteHazards,
    // Handler.
    pub(crate) handler: F,
    // Handler error.
    handler_error: PhantomData<E>,
}

impl<E, F> DeviceAction<E, F>
where
    F: for<'r> Fn(Request<&mut EspHttpConnection<'r>>) -> anyhow::Result<(), E> + Send + 'static,
    E: Debug,
{
    /// Creates a new [`DeviceAction`].
    pub fn no_hazards(route: Route, function: F) -> Self {
        Self::init(route, function, Hazards::init())
    }

    /// Creates a new [`DeviceAction`] with a single [`Hazard`].
    pub fn with_hazard(route: Route, function: F, hazard: Hazard) -> Self {
        let mut hazards = Hazards::init();
        hazards.add(hazard);

        Self::init(route, function, hazards)
    }

    /// Creates a new [`DeviceAction`] with [`Hazard`]s.
    pub fn with_hazards(route: Route, function: F, input_hazards: &'static [Hazard]) -> Self {
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

    fn init(route: Route, function: F, hazards: Hazards) -> Self {
        Self {
            route_hazards: RouteHazards::new(route, hazards),
            handler: function,
            handler_error: PhantomData,
        }
    }
}

/// A general smart home device.
pub struct Device<E, F>
where
    F: for<'r> Fn(Request<&mut EspHttpConnection<'r>>) -> anyhow::Result<(), E> + Send + 'static,
    E: Debug,
{
    // Kind.
    kind: DeviceKind,
    // Main device route.
    main_route: &'static str,
    // All device routes with their hazards and hanlders.
    pub(crate) routes_data: Vec<DeviceAction<E, F>, MAXIMUM_ELEMENTS>,
}

impl<E, F> DeviceSerializer for Device<E, F>
where
    F: for<'r> Fn(Request<&mut EspHttpConnection<'r>>) -> anyhow::Result<(), E> + Send + 'static,
    E: Debug,
{
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

impl<E, F> Device<E, F>
where
    F: for<'r> Fn(Request<&mut EspHttpConnection<'r>>) -> anyhow::Result<(), E> + Send + 'static,
    E: Debug,
{
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
    pub fn add_action(mut self, device_chainer: DeviceAction<E, F>) -> Self {
        let _ = self.routes_data.push(device_chainer);
        self
    }
}
