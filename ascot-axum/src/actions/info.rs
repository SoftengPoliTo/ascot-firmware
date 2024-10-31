use core::future::Future;

use ascot_library::device::DeviceInfo;
use ascot_library::payloads::InfoPayload as LibraryInfoPayload;
use ascot_library::route::RouteHazards;

use axum::{
    extract::Json,
    handler::Handler,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use serde::{Deserialize, Serialize};

use super::{ActionError, DeviceAction};

/// An informative payload on a device.
#[derive(Serialize, Deserialize)]
pub struct InfoPayload(LibraryInfoPayload);

impl InfoPayload {
    /// Creates a [`InfoPayload`].
    pub const fn new(info: DeviceInfo) -> Self {
        Self(LibraryInfoPayload::new(info))
    }
}

impl IntoResponse for InfoPayload {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self.0)).into_response()
    }
}

mod private {
    pub trait InfoTypeName<Args> {}
}

impl<F, Fut> private::InfoTypeName<()> for F
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<InfoPayload, ActionError>> + Send,
{
}

macro_rules! impl_info_type_name {
    (
        [$($ty:ident),*], $($last:ident)?
    ) => {
        impl<F, Fut, M, $($ty,)* $($last)?> private::InfoTypeName<(M, $($ty,)* $($last)?)> for F
        where
            F: FnOnce($($ty,)* $($last)?) -> Fut,
            Fut: Future<Output = Result<InfoPayload, ActionError>> + Send,
            {
            }
    };
}
super::all_the_tuples!(impl_info_type_name);

/// Creates a stateful [`DeviceAction`] with a [`InfoPayload`].
pub fn info_stateful<H, T, S>(
    route_hazards: RouteHazards,
    handler: H,
) -> impl FnOnce(S) -> DeviceAction
where
    H: Handler<T, S> + private::InfoTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |state: S| DeviceAction::stateful(route_hazards, handler, state)
}

/// Creates a stateless [`DeviceAction`] with a [`InfoPayload`].
pub fn info_stateless<H, T, S>(
    route_hazards: RouteHazards,
    handler: H,
) -> impl FnOnce(S) -> DeviceAction
where
    H: Handler<T, ()> + private::InfoTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |_state: S| DeviceAction::stateless(route_hazards, handler)
}