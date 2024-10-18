mod empty;
mod serial;

use ascot_library::device::{DeviceError as AscotActionError, DeviceErrorKind};
use ascot_library::hazards::{Hazard, Hazards};

use ascot_library::route::{RestKind, Route, RouteMode};

use axum::{
    extract::Json,
    handler::Handler,
    http::StatusCode,
    response::{IntoResponse, Response},
    Router,
};

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

pub struct ActionError(AscotActionError);

impl ActionError {
    /// Creates a new [`ActionError`] where the description of the error is
    /// passed as a string slice.
    #[inline(always)]
    pub fn from_str(kind: DeviceErrorKind, description: &str) -> Self {
        Self(AscotActionError::from_str(kind, description))
    }

    /// Creates an invalid data [`ActionError`].
    #[inline(always)]
    pub fn invalid_data(description: &str) -> Self {
        Self(AscotActionError::invalid_data(description))
    }

    /// Creates an internal error [`ActionError`].
    #[inline(always)]
    pub fn internal(description: &str) -> Self {
        Self(AscotActionError::internal(description))
    }

    /// Adds information about the error.
    #[inline(always)]
    pub fn info(self, info: &str) -> Self {
        Self(self.0.info(info))
    }
}

impl IntoResponse for ActionError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(self.0)).into_response()
    }
}

pub trait Action: Internal {
    fn miss_hazard(&self, hazard: Hazard) -> bool;
    fn miss_hazards(&self, hazards: &'static [Hazard]) -> bool;
}

pub(crate) trait Internal {
    fn device_action(self) -> DeviceAction;
}

#[derive(Debug)]
pub(crate) struct DeviceAction {
    // Route.
    pub(crate) route: Route,
    // Hazards.
    pub(crate) hazards: Hazards,
    // Router.
    pub(crate) router: Router,
}

impl DeviceAction {
    fn init<H, T>(mut route: Route, hazards: Hazards, handler: H) -> Self
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        route.join_inputs(RouteMode::Linear, Some(":"));

        Self {
            hazards,
            router: Router::new().route(
                route.route(),
                match route.kind() {
                    RestKind::Get => axum::routing::get(handler),
                    RestKind::Put => axum::routing::put(handler),
                    RestKind::Post => axum::routing::post(handler),
                    RestKind::Delete => axum::routing::delete(handler),
                },
            ),
            route,
        }
    }
}

pub use empty::EmptyAction;
pub use serial::SerialAction;
