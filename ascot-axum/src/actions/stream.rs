use core::future::Future;

use std::io::Cursor;

use ascot_library::route::RouteHazards;

use axum::{
    body::Body,
    handler::Handler,
    http::header::HeaderName,
    response::{IntoResponse, Response},
};

use tokio_util::io::ReaderStream;

use super::{ActionError, DeviceAction, MandatoryAction};

/// A stream payload.
pub struct StreamPayload(Response);

impl StreamPayload {
    /// Creates a new [`StreamPayload`].
    pub fn new<const N: usize>(headers: [(HeaderName, &'static str); N], data: Vec<u8>) -> Self {
        let stream = ReaderStream::new(Cursor::new(data));
        let response = (headers, Body::from_stream(stream)).into_response();
        Self(response)
    }
}

impl IntoResponse for StreamPayload {
    fn into_response(self) -> Response {
        self.0
    }
}

mod private {
    pub trait StreamTypeName<Args> {}
}

impl<F, Fut> private::StreamTypeName<()> for F
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<StreamPayload, ActionError>> + Send,
{
}

macro_rules! impl_empty_type_name {
    (
        [$($ty:ident),*], $($last:ident)?
    ) => {
        impl<F, Fut, M, $($ty,)* $($last)?> private::StreamTypeName<(M, $($ty,)* $($last)?)> for F
        where
            F: FnOnce($($ty,)* $($last)?) -> Fut,
            Fut: Future<Output = Result<StreamPayload, ActionError>> + Send,
            {
            }
    };
}
super::all_the_tuples!(impl_empty_type_name);

/// Creates a mandatory stateful [`DeviceAction`] with a [`StreamPayload`].
#[inline(always)]
pub fn mandatory_stream_stateful<H, T, S>(
    route_hazards: RouteHazards,
    handler: H,
) -> impl FnOnce(S) -> MandatoryAction<false>
where
    H: Handler<T, S> + private::StreamTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |state: S| MandatoryAction::new(DeviceAction::stateful(route_hazards, handler, state))
}

/// Creates a stateful [`DeviceAction`] with a [`StreamPayload`].
pub fn stream_stateful<H, T, S>(
    route_hazards: RouteHazards,
    handler: H,
) -> impl FnOnce(S) -> DeviceAction
where
    H: Handler<T, S> + private::StreamTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |state: S| DeviceAction::stateful(route_hazards, handler, state)
}

/// Creates a mandatory stateless [`DeviceAction`] with an [`StreamPayload`].
#[inline(always)]
pub fn mandatory_stream_stateless<H, T, S>(
    route_hazards: RouteHazards,
    handler: H,
) -> impl FnOnce(S) -> MandatoryAction<false>
where
    H: Handler<T, ()> + private::StreamTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |_state: S| MandatoryAction::new(DeviceAction::stateless(route_hazards, handler))
}

/// Creates a stateless [`DeviceAction`] with a [`StreamPayload`].
pub fn stream_stateless<H, T, S>(
    route_hazards: RouteHazards,
    handler: H,
) -> impl FnOnce(S) -> DeviceAction
where
    H: Handler<T, ()> + private::StreamTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |_state: S| DeviceAction::stateless(route_hazards, handler)
}
