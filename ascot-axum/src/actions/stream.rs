use core::error::Error;
use core::future::Future;

use ascot_library::route::RouteHazards;

use axum::{
    body::{Body, Bytes},
    handler::Handler,
    http::header::HeaderName,
    response::{IntoResponse, Response},
};

use futures_core::TryStream;

use tokio::io::AsyncRead;
use tokio_util::io::ReaderStream;

use super::{ActionError, DeviceAction, MandatoryAction};

/// A stream payload.
pub struct StreamPayload(Response);

impl StreamPayload {
    /// Creates a new [`StreamPayload`] from headers and stream.
    #[inline]
    pub fn from_headers_and_stream<const N: usize, S>(
        headers: [(HeaderName, &str); N],
        stream: S,
    ) -> Self
    where
        S: TryStream + Send + 'static,
        S::Ok: Into<Bytes>,
        S::Error: Into<Box<dyn Error + Sync + Send>>,
    {
        Self((headers, Body::from_stream(stream)).into_response())
    }

    /// Creates a new [`StreamPayload`] from a stream.
    #[inline]
    pub fn from_stream<S>(stream: S) -> Self
    where
        S: TryStream + Send + 'static,
        S::Ok: Into<Bytes>,
        S::Error: Into<Box<dyn Error + Sync + Send>>,
    {
        Self(Body::from_stream(stream).into_response())
    }

    /// Creates a new [`StreamPayload`] from headers and reader.
    #[inline]
    pub fn from_headers_and_reader<const N: usize, R>(
        headers: [(HeaderName, &str); N],
        reader: R,
    ) -> Self
    where
        R: AsyncRead + Send + 'static,
    {
        let stream = ReaderStream::new(reader);
        Self((headers, Body::from_stream(stream)).into_response())
    }

    /// Creates a new [`StreamPayload`] from a reader.
    #[inline]
    pub fn from_reader<R>(reader: R) -> Self
    where
        R: AsyncRead + Send + 'static,
    {
        let stream = ReaderStream::new(reader);
        Self(Body::from_stream(stream).into_response())
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
