use core::future::Future;

use std::io::Cursor;

use ascot_library::route::RouteHazards;

use axum::{
    body::Body,
    handler::Handler,
    http::{
        header::{HeaderName, HeaderValue},
        StatusCode,
    },
    response::{IntoResponse, IntoResponseParts, Response, ResponseParts},
};

use tokio_util::io::ReaderStream;

use super::{ActionError, DeviceAction, MandatoryAction};

/// Stream headers.
pub struct Headers(&'static [(&'static str, &'static str)]);

impl Headers {
    /// Creates [`Headers`].
    pub const fn new(headers: &'static [(&'static str, &'static str)]) -> Self {
        Self(headers)
    }
}

impl IntoResponseParts for Headers {
    type Error = (StatusCode, String);

    fn into_response_parts(self, mut res: ResponseParts) -> Result<ResponseParts, Self::Error> {
        for (name, value) in self.0 {
            match (name.parse::<HeaderName>(), value.parse::<HeaderValue>()) {
                (Ok(name), Ok(value)) => {
                    res.headers_mut().insert(name, value);
                }
                (Err(_), _) => {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Invalid header name {name}"),
                    ));
                }
                (_, Err(_)) => {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Invalid header value {value}"),
                    ));
                }
            }
        }

        Ok(res)
    }
}

/// A stream payload.
pub struct StreamPayload {
    headers: Headers,
    data: Vec<u8>,
}

impl StreamPayload {
    /// Creates a new [`StreamPayload`].
    pub const fn new(headers: Headers, data: Vec<u8>) -> Self {
        Self { headers, data }
    }
}

impl IntoResponse for StreamPayload {
    fn into_response(self) -> Response {
        let stream = ReaderStream::new(Cursor::new(self.data));
        (self.headers, Body::from_stream(stream)).into_response()
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
pub fn mandatory_empty_stateful<H, T, S>(
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
pub fn empty_stateful<H, T, S>(
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
pub fn mandatory_empty_stateless<H, T, S>(
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
pub fn empty_stateless<H, T, S>(
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
