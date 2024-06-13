use ascot_library::device::{DeviceData, DeviceErrorKind, DeviceKind, DeviceSerializer};
use ascot_library::hazards::{Hazard, Hazards};
use ascot_library::route::{RestKind, Route, RouteMode, Routes};

use axum::{
    extract::Json,
    handler::Handler,
    http::StatusCode,
    response::{IntoResponse, Response},
    Router,
};

use heapless::FnvIndexSet;

use serde::Serialize;

use crate::MAXIMUM_ELEMENTS;

// Default main route for a device.
const DEFAULT_MAIN_ROUTE: &str = "/device";

/// A device payload for a determined action.
pub struct DevicePayload(ascot_library::device::DevicePayload);

impl DevicePayload {
    /// Creates an empty [`DevicePayload`].
    pub fn empty() -> Self {
        Self(ascot_library::device::DevicePayload::empty())
    }

    /// Creates a [`DevicePayload`].
    pub fn new(value: impl Serialize) -> core::result::Result<Self, DeviceError> {
        ascot_library::device::DevicePayload::new(value)
            .map(Self)
            .map_err(DeviceError)
    }
}

impl IntoResponse for DevicePayload {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self.0)).into_response()
    }
}

/// A device error response.
pub struct DeviceError(ascot_library::device::DeviceError);

impl DeviceError {
    /// Creates a new [`DeviceError`] where the error is given as
    /// a string slice.
    pub fn from_str(kind: DeviceErrorKind, info: &str) -> Self {
        Self(ascot_library::device::DeviceError::from_str(kind, info))
    }

    /// Creates a new [`DeviceError`] where the error is given as a
    /// [`String`].
    pub fn from_string(kind: DeviceErrorKind, info: String) -> Self {
        Self::from_str(kind, &info)
    }
}

impl IntoResponse for DeviceError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(self.0)).into_response()
    }
}

/// A device action connects a server route with a device handler and,
/// optionally, with every possible hazards associated with the handler.
#[derive(Debug)]
pub struct DeviceAction {
    // Route.
    pub(crate) route: Route,
    // Hazards.
    pub(crate) hazards: Hazards,
    // Router.
    pub(crate) router: Router,
}

impl DeviceAction {
    /// Creates a new [`DeviceAction`].
    pub fn no_hazards<H, T>(route: Route, handler: H) -> Self
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        Self::init(route, Hazards::init(), handler)
    }

    /// Creates a new [`DeviceAction`] with a single [`Hazard`].
    pub fn with_hazard<H, T>(route: Route, handler: H, hazard: Hazard) -> Self
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        let mut hazards = Hazards::init();
        hazards.add(hazard);

        Self::init(route, hazards, handler)
    }

    /// Creates a new [`DeviceAction`] with [`Hazard`]s.
    pub fn with_hazards<H, T>(route: Route, handler: H, input_hazards: &'static [Hazard]) -> Self
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        let mut hazards = Hazards::init();
        input_hazards.iter().for_each(|hazard| {
            hazards.add(*hazard);
        });

        Self::init(route, hazards, handler)
    }

    /// Checks whether a [`DeviceAction`] misses a specific [`Hazard`].
    pub fn miss_hazard(&self, hazard: Hazard) -> bool {
        !self.hazards.contains(hazard)
    }

    /// Checks whether a [`DeviceAction`] misses the given [`Hazard`]s.
    pub fn miss_hazards(&self, hazards: &'static [Hazard]) -> bool {
        !hazards.iter().all(|hazard| self.hazards.contains(*hazard))
    }

    fn init<H, T>(mut route: Route, hazards: Hazards, handler: H) -> Self
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        route.join_inputs(RouteMode::Linear);

        Self {
            hazards,
            router: Router::new().route(
                route.route(),
                match route.kind() {
                    RestKind::Get => axum::routing::get(handler),
                    RestKind::Put => axum::routing::put(handler),
                    RestKind::Post => axum::routing::post(handler),
                },
            ),
            route,
        }
    }
}

// A route with its associated hazards.
#[derive(Debug)]
struct RouteHazards {
    // Route information.
    route: Route,
    // Hazards.
    hazards: Hazards,
}

impl core::cmp::PartialEq for RouteHazards {
    fn eq(&self, other: &Self) -> bool {
        self.route.eq(&other.route)
    }
}

impl core::cmp::Eq for RouteHazards {}

impl core::hash::Hash for RouteHazards {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.route.hash(state);
    }
}

impl RouteHazards {
    pub(crate) fn new(route: Route, hazards: Hazards) -> Self {
        Self { route, hazards }
    }
}

// Build a device from a precise device.
pub(crate) trait DeviceBuilder<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn into_device(self) -> Device<S>;
}

/// A general smart home device.
#[derive(Debug)]
pub struct Device<S>
where
    S: Clone + Send + Sync + 'static,
{
    // Kind.
    kind: DeviceKind,
    // Main device route.
    main_route: &'static str,
    // All device routes and their hazards.
    routes: FnvIndexSet<RouteHazards, MAXIMUM_ELEMENTS>,
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
        let mut routes = Routes::init();
        for route_hazards in self.routes.into_iter() {
            routes.add(route_hazards.route.serialize_data(&route_hazards.hazards));
        }

        DeviceData {
            kind: self.kind,
            main_route: self.main_route,
            routes,
        }
    }
}

impl<S> Device<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Creates a new [`Device`] instance.
    pub fn new(kind: DeviceKind) -> Self {
        Self {
            kind,
            main_route: DEFAULT_MAIN_ROUTE,
            routes: FnvIndexSet::new(),
            router: Router::new(),
            state: None,
        }
    }

    /// Sets a new main route.
    pub fn main_route(mut self, main_route: &'static str) -> Self {
        self.main_route = main_route;
        self
    }

    /// Adds a [`DeviceAction`].
    pub fn add_action(mut self, device_chainer: DeviceAction) -> Self {
        self.router = self.router.merge(device_chainer.router);
        let _ = self.routes.insert(RouteHazards::new(
            device_chainer.route,
            device_chainer.hazards,
        ));
        self
    }

    /// Sets a device state.
    pub fn state(self, state: S) -> Self {
        self.internal_state(Some(state))
    }

    // Sets internal state.
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
