use core::any::type_name;
use core::future::Future;

use ascot_library::hazards::{Hazard, Hazards};
use ascot_library::payloads::EmptyPayload as AscotEmptyPayload;
use ascot_library::route::Route;

use axum::{
    extract::Json,
    handler::Handler,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use serde::{Deserialize, Serialize};

use super::{Action, ActionError, DeviceAction, Internal};

/// Empty payload.
#[derive(Serialize, Deserialize)]
pub struct EmptyPayload(AscotEmptyPayload);

impl EmptyPayload {
    /// Creates a new [`EmptyPayload`].
    #[inline(always)]
    pub fn new(description: &str) -> Self {
        Self(AscotEmptyPayload::new(description))
    }
}

impl IntoResponse for EmptyPayload {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self.0)).into_response()
    }
}

pub trait EmptyTypeName<Args> {
    fn empty_type_name(&self) -> &'static str;
}

impl<F, Fut> EmptyTypeName<()> for F
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<EmptyPayload, ActionError>> + Send,
{
    fn empty_type_name(&self) -> &'static str {
        type_name::<Fut::Output>()
    }
}

macro_rules! impl_empty_type_name {
    (
        [$($ty:ident),*], $($last:ident)?
    ) => {
        impl<F, Fut, M, $($ty,)* $($last)?> EmptyTypeName<(M, $($ty,)* $($last)?)> for F
        where
            F: FnOnce($($ty,)* $($last)?) -> Fut,
            Fut: Future<Output = Result<EmptyPayload, ActionError>> + Send,
            {
                fn empty_type_name(&self) -> &'static str {
                    type_name::<Fut::Output>()
                }
            }
    };
}
super::all_the_tuples!(impl_empty_type_name);

pub struct EmptyAction(DeviceAction);

impl Internal for EmptyAction {
    fn device_action(self) -> DeviceAction {
        self.0
    }
}

impl Action for EmptyAction {
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

impl EmptyAction {
    /// Creates a new [`EmptyAction`].
    #[inline]
    pub fn no_hazards<H, T>(route: Route, handler: H) -> Self
    where
        H: Handler<T, ()> + EmptyTypeName<T>,
        T: 'static,
    {
        Self(DeviceAction::init(route, Hazards::init(), handler))
    }

    /// Creates a new [`EmptyAction`] with a single [`Hazard`].
    #[inline]
    pub fn with_hazard<H, T>(route: Route, handler: H, hazard: Hazard) -> Self
    where
        H: Handler<T, ()> + EmptyTypeName<T>,
        T: 'static,
    {
        let mut hazards = Hazards::init();
        hazards.add(hazard);

        Self(DeviceAction::init(route, hazards, handler))
    }

    /// Creates a new [`EmptyAction`] with [`Hazard`]s.
    #[inline]
    pub fn with_hazards<H, T>(route: Route, handler: H, input_hazards: &'static [Hazard]) -> Self
    where
        H: Handler<T, ()> + EmptyTypeName<T>,
        T: 'static,
    {
        let mut hazards = Hazards::init();
        input_hazards.iter().for_each(|hazard| {
            hazards.add(*hazard);
        });

        Self(DeviceAction::init(route, hazards, handler))
    }
}
