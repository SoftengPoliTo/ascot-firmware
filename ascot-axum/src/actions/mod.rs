pub mod empty;
pub mod info;
pub mod serial;
pub mod stream;

use ascot_library::actions::ActionErrorKind;
use ascot_library::hazards::{Hazard, Hazards};

use ascot_library::route::{RestKind, Route, RouteHazards, RouteMode};

use axum::{
    extract::Json,
    handler::Handler,
    http::StatusCode,
    response::{IntoResponse, Response},
    Router,
};

use serde::Serialize;

#[rustfmt::skip]
macro_rules! all_the_tuples {
    ($name:ident) => {
        $name!([], );
        $name!([], T1);
        $name!([T1], T2);
        $name!([T1, T2], T3);
        $name!([T1, T2, T3], T4);
        $name!([T1, T2, T3, T4], T5);
        $name!([T1, T2, T3, T4, T5], T6);
        $name!([T1, T2, T3, T4, T5, T6], T7);
        $name!([T1, T2, T3, T4, T5, T6, T7], T8);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8], T9);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9], T10);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10], T11);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11], T12);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12], T13);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13], T14);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14], T15);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15], T16);
    };
}

pub(super) use all_the_tuples;

#[derive(Serialize)]
struct ErrorPayload {
    kind: ActionErrorKind,
    description: &'static str,
    error: Option<String>,
}

/// An error which might arise during the execution of an action on a device.
pub struct ActionError(ErrorPayload);

impl ActionError {
    /// Creates a new [`ActionError`] with a specific [`ActionErrorKind`]
    /// and an error description.
    #[inline]
    pub fn with_description(kind: ActionErrorKind, description: &'static str) -> Self {
        Self(ErrorPayload {
            kind,
            description,
            error: None,
        })
    }

    /// Creates a new [`ActionError`] with a specific [`ActionErrorKind`],
    /// an error description, and the effective error.
    #[inline]
    pub fn with_description_error(
        kind: ActionErrorKind,
        description: &'static str,
        error: impl std::error::Error,
    ) -> Self {
        Self(ErrorPayload {
            kind,
            description,
            error: Some(error.to_string()),
        })
    }

    /// Creates an [`ActionError`] for invalid data with a description.
    #[inline]
    pub fn invalid_data(description: &'static str) -> Self {
        Self::with_description(ActionErrorKind::InvalidData, description)
    }

    /// Creates an [`ActionError`] for invalid data with a description and
    /// the effective error.
    #[inline]
    pub fn invalid_data_with_error(
        description: &'static str,
        error: impl std::error::Error,
    ) -> Self {
        Self::with_description_error(ActionErrorKind::InvalidData, description, error)
    }

    /// Creates an [`ActionError`] for an internal error with a description.
    #[inline]
    pub fn internal(description: &'static str) -> Self {
        Self::with_description(ActionErrorKind::Internal, description)
    }

    /// Creates an [`ActionError`] for an internal error with a description and
    /// the effective error.
    #[inline(always)]
    pub fn internal_with_error(description: &'static str, error: impl std::error::Error) -> Self {
        Self::with_description_error(ActionErrorKind::Internal, description, error)
    }
}

impl IntoResponse for ActionError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(self.0)).into_response()
    }
}

#[derive(Debug)]
pub struct DeviceAction {
    // Router.
    pub(crate) router: Router,
    // Route - Hazards
    pub(crate) route_hazards: RouteHazards,
}

impl DeviceAction {
    /// Checks whether an action does not define the given [`Hazard`].
    #[inline]
    pub fn miss_hazard(&self, hazard: Hazard) -> bool {
        !self.route_hazards.hazards.contains(hazard)
    }

    /// Checks whether an action does not define the given [`Hazard`]s.
    #[inline]
    pub fn miss_hazards(&self, hazards: &'static [Hazard]) -> bool {
        !hazards
            .iter()
            .all(|hazard| self.route_hazards.hazards.contains(*hazard))
    }

    /// Returns the [`Hazards`] collection associated with an action.
    #[inline]
    pub fn hazards(&self) -> &Hazards {
        &self.route_hazards.hazards
    }

    #[inline]
    pub(crate) fn stateless<H, T>(mut route_hazards: RouteHazards, handler: H) -> Self
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        route_hazards
            .route
            .join_inputs(RouteMode::Linear, Some(":"));

        Self {
            router: Self::create_router(
                route_hazards.route.route(),
                route_hazards.route.kind(),
                handler,
                (),
            ),
            route_hazards,
        }
    }

    #[inline]
    pub(crate) fn stateful<H, T, S>(mut route_hazards: RouteHazards, handler: H, state: S) -> Self
    where
        H: Handler<T, S>,
        T: 'static,
        S: Clone + Send + Sync + 'static,
    {
        route_hazards
            .route
            .join_inputs(RouteMode::Linear, Some(":"));

        Self {
            router: Self::create_router(
                route_hazards.route.route(),
                route_hazards.route.kind(),
                handler,
                state,
            ),
            route_hazards,
        }
    }

    pub(crate) fn empty() -> Self {
        Self {
            router: Router::new(),
            route_hazards: RouteHazards::new(Route::get(""), Hazards::empty()),
        }
    }

    #[inline]
    fn create_router<H, T, S>(route: &str, route_kind: RestKind, handler: H, state: S) -> Router
    where
        H: Handler<T, S>,
        T: 'static,
        S: Clone + Send + Sync + 'static,
    {
        Router::new()
            .route(
                route,
                match route_kind {
                    RestKind::Get => axum::routing::get(handler),
                    RestKind::Put => axum::routing::put(handler),
                    RestKind::Post => axum::routing::post(handler),
                    RestKind::Delete => axum::routing::delete(handler),
                },
            )
            .with_state(state)
    }
}

/// A mandatory [`DeviceAction`].
pub struct MandatoryAction<const SET: bool> {
    pub(crate) device_action: DeviceAction,
}

impl MandatoryAction<false> {
    #[inline(always)]
    pub(crate) fn empty() -> Self {
        Self {
            device_action: DeviceAction::empty(),
        }
    }

    pub(super) const fn new(device_action: DeviceAction) -> Self {
        Self { device_action }
    }
}

impl MandatoryAction<true> {
    /// Returns a [`DeviceAction`] reference.
    pub const fn action_as_ref(&self) -> &DeviceAction {
        &self.device_action
    }

    pub(crate) const fn init(device_action: DeviceAction) -> Self {
        Self { device_action }
    }
}
