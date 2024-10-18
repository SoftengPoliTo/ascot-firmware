use ascot_library::device::{DeviceData, DeviceKind, DeviceSerializer};
use ascot_library::route::{RouteConfigs, RouteHazards, RoutesHazards};

use axum::Router;

use crate::actions::Action;

// Default main route for a device.
const DEFAULT_MAIN_ROUTE: &str = "/device";

// Build a device from a precise device.
pub(crate) trait DeviceBuilder<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn into_device(self) -> Device<S>;
}

/// A general smart home device.
#[derive(Debug)]
pub struct Device<S> {
    // Kind.
    kind: DeviceKind,
    // Main device route.
    main_route: &'static str,
    // All device routes and their hazards.
    routes_hazards: RoutesHazards,
    // Router.
    pub(crate) router: Router,
    // Device state.
    pub(crate) state: Option<S>,
}

impl<S> DeviceSerializer for Device<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn serialize_data(&self) -> DeviceData {
        let mut route_configs = RouteConfigs::init();
        for route_hazards in self.routes_hazards.iter() {
            route_configs.add(route_hazards.serialize_data());
        }

        DeviceData {
            kind: self.kind,
            main_route: self.main_route,
            route_configs,
        }
    }
}

impl<S> Device<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Creates an unknown [`Device`].
    #[inline]
    pub fn unknown() -> Self {
        Self::new(DeviceKind::Unknown)
    }

    /// Sets a new main route.
    pub const fn main_route(mut self, main_route: &'static str) -> Self {
        self.main_route = main_route;
        self
    }

    /// Adds a [`DeviceAction`].
    #[inline]
    pub fn add_action(mut self, device_chainer: impl Action) -> Self {
        let device_chainer = device_chainer.device_action();
        self.router = self.router.merge(device_chainer.router);
        self.routes_hazards.add(RouteHazards::new(
            device_chainer.route,
            device_chainer.hazards,
        ));
        self
    }

    /// Sets a device state.
    #[inline]
    pub fn state(self, state: S) -> Self {
        self.internal_state(Some(state))
    }

    // Creates a new instance defining the DeviceKind.
    #[inline]
    pub(crate) fn new(kind: DeviceKind) -> Self {
        Self {
            kind,
            main_route: DEFAULT_MAIN_ROUTE,
            routes_hazards: RoutesHazards::init(),
            router: Router::new(),
            state: None,
        }
    }

    // Sets internal state.
    #[inline]
    pub(crate) fn internal_state(mut self, state: Option<S>) -> Self {
        self.state = state;
        self
    }

    // Finalizes a device composing all correct routes.
    #[inline]
    pub(crate) fn finalize(mut self) -> Self {
        self.router = Router::new().nest(self.main_route, self.router);
        self
    }
}
