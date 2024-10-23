use alloc::sync::Arc;

use ascot_library::device::{DeviceData, DeviceKind, DeviceSerializer};
use ascot_library::economy::Economy;
use ascot_library::energy::Energy;
use ascot_library::route::{Route, RouteConfigs, RoutesHazards};

use async_lock::Mutex;

use axum::{extract::FromRef, Router};

use serde::Serialize;

use crate::actions::{Action, SerialAction, SerialPayload};

// Default main route for a device.
const DEFAULT_MAIN_ROUTE: &str = "/device";

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
        let mut route_configs = RouteConfigs::empty();
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

pub struct AppState<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub api_state: S,
    energy: EnergyState,
}

#[derive(Clone)]
struct EnergyState {
    energy: Arc<Mutex<Energy>>,
}

impl<S> FromRef<AppState<S>> for EnergyState
where
    S: Clone + Send + Sync + 'static,
{
    fn from_ref(app_state: &AppState<S>) -> EnergyState {
        app_state.energy.clone()
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

    /// Adds an [`Action`] to the device.
    #[inline]
    pub fn add_action(mut self, device_chainer: impl Action) -> Self {
        let (router, route_hazards) = device_chainer.data();
        self.router = self.router.merge(router);
        self.routes_hazards.add(route_hazards);
        self
    }

    /// Adds a `GET` route to visualize the device energy data.
    pub fn add_energy(
        self,
        route_path: &'static str,
        description: &'static str,
        energy_data: Energy,
    ) -> Self {
        self.define_get_route(route_path, description, energy_data)
    }

    /// Adds a `GET` route to visualize the device economy data.
    #[inline(always)]
    pub fn add_economy(
        self,
        route_path: &'static str,
        description: &'static str,
        economy_data: Economy,
    ) -> Self {
        self.define_get_route(route_path, description, economy_data)
    }

    /// Sets a device state.
    #[inline]
    pub fn state(mut self, state: S) -> Self {
        self.state = Some(state);
        self
    }

    // Creates a new instance defining the DeviceKind.
    #[inline]
    pub(crate) fn new(kind: DeviceKind) -> Self {
        Self {
            kind,
            main_route: DEFAULT_MAIN_ROUTE,
            routes_hazards: RoutesHazards::empty(),
            router: Router::new(),
            state: None,
        }
    }

    // Finalizes a device composing all correct routes.
    #[inline]
    pub(crate) fn finalize(mut self) -> Self {
        self.router = Router::new().nest(self.main_route, self.router);
        self
    }

    fn define_get_route<K: Serialize + Clone + Send + 'static>(
        self,
        route_path: &'static str,
        description: &'static str,
        data: K,
    ) -> Self {
        let data_closure = move || Ok(SerialPayload::new(data));

        let data_action = SerialAction::no_hazards(
            Route::get(route_path).description(description),
            move || async { data_closure() },
        );

        self.add_action(data_action)
    }
}
