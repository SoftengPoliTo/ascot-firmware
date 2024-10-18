use core::any::type_name;
use core::future::Future;

use ascot_library::hazards::{Hazard, Hazards};
use ascot_library::payloads::SerialPayload as AscotSerialPayload;
use ascot_library::route::Route;

use axum::{
    extract::Json,
    handler::Handler,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use serde::{Deserialize, Serialize};

use super::{private::Internal, Action, ActionError, DeviceAction};

/// Serial payload structure.
#[derive(Serialize, Deserialize)]
pub struct SerialPayload<S: Serialize>(AscotSerialPayload<S>);

impl<S: Serialize> SerialPayload<S> {
    /// Creates a new [`SerialPayload`].
    pub const fn new(data: S) -> Self {
        Self(AscotSerialPayload::new(data))
    }
}

impl<S: Serialize> IntoResponse for SerialPayload<S> {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self.0)).into_response()
    }
}

pub trait SerialTypeName<Args> {
    fn serial_type_name(&self) -> &'static str;
}

impl<S, F, Fut> SerialTypeName<()> for F
where
    S: Serialize,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<SerialPayload<S>, ActionError>> + Send,
{
    fn serial_type_name(&self) -> &'static str {
        type_name::<Fut::Output>()
    }
}

macro_rules! impl_serial_type_name {
    (
        [$($ty:ident),*], $($last:ident)?
    ) => {
        impl<F, S, Fut, M, $($ty,)* $($last)?> SerialTypeName<(M, $($ty,)* $($last)?)> for F
        where
            S: Serialize,
            F: FnOnce($($ty,)* $($last)?) -> Fut,
            Fut: Future<Output = Result<SerialPayload<S>, ActionError>> + Send,
            {
                fn serial_type_name(&self) -> &'static str {
                    type_name::<Fut::Output>()
                }
            }
    };
}

super::all_the_tuples!(impl_serial_type_name);

pub struct SerialAction(DeviceAction);

impl Internal for SerialAction {
    fn device_action(self) -> DeviceAction {
        self.0
    }
}

impl Action for SerialAction {
    #[inline]
    fn miss_hazard(&self, hazard: Hazard) -> bool {
        !self.0.hazards.contains(hazard)
    }

    #[inline]
    fn miss_hazards(&self, hazards: &'static [Hazard]) -> bool {
        !hazards
            .iter()
            .all(|hazard| self.0.hazards.contains(*hazard))
    }
}

impl SerialAction {
    /// Creates a new [`SerialAction`].
    #[inline]
    pub fn no_hazards<H, T>(route: Route, handler: H) -> Self
    where
        H: Handler<T, ()> + SerialTypeName<T>,
        T: 'static,
    {
        Self(DeviceAction::init(route, Hazards::init(), handler))
    }

    /// Creates a new [`SerialAction`] with a single [`Hazard`].
    #[inline]
    pub fn with_hazard<H, T>(route: Route, handler: H, hazard: Hazard) -> Self
    where
        H: Handler<T, ()> + SerialTypeName<T>,
        T: 'static,
    {
        let mut hazards = Hazards::init();
        hazards.add(hazard);

        Self(DeviceAction::init(route, hazards, handler))
    }

    /// Creates a new [`SerialAction`] with [`Hazard`]s.
    #[inline]
    pub fn with_hazards<H, T>(route: Route, handler: H, input_hazards: &'static [Hazard]) -> Self
    where
        H: Handler<T, ()> + SerialTypeName<T>,
        T: 'static,
    {
        let mut hazards = Hazards::init();
        input_hazards.iter().for_each(|hazard| {
            hazards.add(*hazard);
        });

        Self(DeviceAction::init(route, hazards, handler))
    }

    /// Checks whether a [`SerialAction`] misses a specific [`Hazard`].
    #[inline]
    pub fn miss_hazard(&self, hazard: Hazard) -> bool {
        !self.0.hazards.contains(hazard)
    }

    /// Checks whether a [`SerialAction`] misses the given [`Hazard`]s.
    #[inline]
    pub fn miss_hazards(&self, hazards: &'static [Hazard]) -> bool {
        !hazards
            .iter()
            .all(|hazard| self.0.hazards.contains(*hazard))
    }
}
