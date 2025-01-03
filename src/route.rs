use serde::{Deserialize, Serialize};

use crate::collections::Collection;
use crate::hazards::{Hazard, Hazards};
use crate::input::{Input, Inputs, InputsData};

use crate::MAXIMUM_ELEMENTS;

#[cfg(feature = "std")]
mod route_data {
    use alloc::borrow::Cow;

    use super::{Deserialize, InputsData, Route, Serialize};

    /// Route data.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RouteData {
        /// Name.
        pub name: Cow<'static, str>,
        /// Description.
        pub description: Option<Cow<'static, str>>,
        /// Inputs associated with a route..
        #[serde(skip_serializing_if = "InputsData::is_empty")]
        pub inputs: InputsData,
    }

    impl PartialEq for RouteData {
        fn eq(&self, other: &Self) -> bool {
            self.name.eq(&other.name)
        }
    }

    impl RouteData {
        pub(super) fn new(route: &Route) -> Self {
            Self {
                name: route.route.into(),
                description: route.description.map(core::convert::Into::into),
                inputs: InputsData::from(&route.inputs),
            }
        }
    }
}

#[cfg(not(feature = "std"))]
mod route_data {
    use super::{InputsData, Route, Serialize};

    /// Route data.
    #[derive(Debug, Clone, Serialize)]
    pub struct RouteData {
        /// Name.
        pub name: &'static str,
        /// Description.
        pub description: Option<&'static str>,
        /// Inputs associated with a route..
        #[serde(skip_serializing_if = "InputsData::is_empty")]
        pub inputs: InputsData,
    }

    impl PartialEq for RouteData {
        fn eq(&self, other: &Self) -> bool {
            self.name.eq(other.name)
        }
    }

    impl RouteData {
        pub(super) fn new(route: &Route) -> Self {
            Self {
                name: route.route,
                description: route.description,
                inputs: InputsData::from(&route.inputs),
            }
        }
    }
}

pub use route_data::RouteData;

/// Kind of a `REST` API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RestKind {
    // `GET` API.
    Get,
    // `PUT` API.
    Put,
    // `POST` API.
    Post,
    // `DELETE` API
    Delete,
}

impl core::fmt::Display for RestKind {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Get => "GET",
            Self::Put => "PUT",
            Self::Post => "POST",
            Self::Delete => "DELETE",
        }
        .fmt(f)
    }
}

#[cfg(feature = "std")]
mod route_config {
    use super::{Deserialize, Hazards, RestKind, RouteData, Serialize};

    /// A server route configuration.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RouteConfig {
        /// Route.
        #[serde(flatten)]
        pub data: RouteData,
        /// Kind of a `REST` API.
        #[serde(rename = "REST kind")]
        pub rest_kind: RestKind,
        /// Hazards data.
        #[serde(skip_serializing_if = "Hazards::is_empty")]
        pub hazards: Hazards,
    }

    /// A collection of [`RouteConfig`]s.
    pub type RouteConfigs = crate::collections::OutputCollection<RouteConfig>;
}

#[cfg(not(feature = "std"))]
mod route_config {
    use super::{Hazards, RestKind, RouteData, Serialize};

    /// A server route configuration.
    #[derive(Debug, Clone, Serialize)]
    pub struct RouteConfig {
        /// Route.
        #[serde(flatten)]
        pub data: RouteData,
        /// Kind of a `REST` API.
        #[serde(rename = "REST kind")]
        pub rest_kind: RestKind,
        /// Hazards data.
        #[serde(skip_serializing_if = "Hazards::is_empty")]
        pub hazards: Hazards,
    }

    /// A collection of [`RouteConfig`]s.
    pub type RouteConfigs = crate::collections::SerialCollection<RouteConfig>;
}

impl PartialEq for route_config::RouteConfig {
    fn eq(&self, other: &Self) -> bool {
        self.data.eq(&other.data) && self.rest_kind == other.rest_kind
    }
}

impl Eq for route_config::RouteConfig {}

impl core::hash::Hash for route_config::RouteConfig {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.data.name.hash(state);
        self.rest_kind.hash(state);
    }
}

impl route_config::RouteConfig {
    fn new(route: &Route) -> Self {
        Self {
            rest_kind: route.rest_kind,
            hazards: route.hazards.clone(),
            data: RouteData::new(route),
        }
    }
}

pub use route_config::RouteConfig;
pub use route_config::RouteConfigs;

/// A server route.
///
/// It represents a specific `REST` API which, when invoked, runs a task on
/// a remote device.
#[derive(Debug)]
pub struct Route {
    // Route.
    route: &'static str,
    // REST kind.
    rest_kind: RestKind,
    // Description.
    description: Option<&'static str>,
    // Inputs.
    inputs: Inputs,
    // Hazards.
    hazards: Hazards,
}

impl PartialEq for Route {
    fn eq(&self, other: &Self) -> bool {
        self.route == other.route && self.rest_kind == other.rest_kind
    }
}

impl Eq for Route {}

impl core::hash::Hash for Route {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.route.hash(state);
        self.rest_kind.hash(state);
    }
}

impl Route {
    /// Creates a new [`Route`] through a REST `GET` API.
    #[must_use]
    pub const fn get(route: &'static str) -> Self {
        Self::init(RestKind::Get, route)
    }

    /// Creates a new [`Route`] through a REST `PUT` API.
    #[must_use]
    pub const fn put(route: &'static str) -> Self {
        Self::init(RestKind::Put, route)
    }

    /// Creates a new [`Route`] through a REST `POST` API.
    #[must_use]
    pub const fn post(route: &'static str) -> Self {
        Self::init(RestKind::Post, route)
    }

    /// Creates a new [`Route`] through a REST `DELETE` API.
    #[must_use]
    pub const fn delete(route: &'static str) -> Self {
        Self::init(RestKind::Delete, route)
    }

    /// Sets the route description.
    #[must_use]
    pub const fn description(mut self, description: &'static str) -> Self {
        self.description = Some(description);
        self
    }

    /// Sets a single [`Input`].
    #[must_use]
    #[inline]
    pub fn input(mut self, input: Input) -> Self {
        self.inputs.add(input);
        self
    }

    /// Sets more [`Input`]s.
    #[must_use]
    #[inline]
    pub fn inputs<const N: usize>(mut self, inputs: [Input; N]) -> Self {
        inputs.into_iter().take(MAXIMUM_ELEMENTS).for_each(|input| {
            self.inputs.add(input);
        });
        self
    }

    /// Returns route.
    #[must_use]
    pub fn route(&self) -> &str {
        self.route
    }

    /// Returns [`RestKind`].
    #[must_use]
    pub const fn kind(&self) -> RestKind {
        self.rest_kind
    }

    /// Returns [`Hazards`].
    #[must_use]
    pub const fn hazards(&self) -> &Hazards {
        &self.hazards
    }

    /// Adds [`Hazards`] to a [`Route`].
    #[must_use]
    #[inline]
    pub fn with_hazards(mut self, hazards: Hazards) -> Self {
        self.hazards = hazards;
        self
    }

    /// Adds a single [`Hazard`] to a [`Route`].
    #[must_use]
    #[inline]
    pub fn with_single_hazard(mut self, hazard: Hazard) -> Self {
        self.hazards = Hazards::init(hazard);
        self
    }

    /// Adds a slice of [`Hazard`]s to a [`Route`].
    #[must_use]
    #[inline]
    pub fn with_slice_hazards(mut self, hazards: &'static [Hazard]) -> Self {
        self.hazards = Hazards::init_with_elements(hazards);
        self
    }

    /// Serializes [`Route`] data.
    ///
    /// It consumes the data.
    #[must_use]
    #[inline]
    pub fn serialize_data(&self) -> RouteConfig {
        RouteConfig::new(self)
    }

    const fn init(rest_kind: RestKind, route: &'static str) -> Self {
        Self {
            route,
            rest_kind,
            description: None,
            hazards: Hazards::empty(),
            inputs: Inputs::empty(),
        }
    }
}

/// A collection of [`Route`]s.
pub type Routes = Collection<Route>;

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::serialize;

    use super::{Hazard, Hazards, Input, Route};

    #[test]
    fn test_all_routes() {
        assert_eq!(
            serialize(
                Route::get("/route")
                    .description("A GET route")
                    .serialize_data()
            ),
            json!({
                "name": "/route",
                "description": "A GET route",
                "REST kind": "Get"
            })
        );

        assert_eq!(
            serialize(
                Route::put("/route")
                    .description("A PUT route")
                    .serialize_data()
            ),
            json!({
                "name": "/route",
                "description": "A PUT route",
                "REST kind": "Put"
            })
        );

        assert_eq!(
            serialize(
                Route::post("/route")
                    .description("A POST route")
                    .serialize_data()
            ),
            json!({
                "name": "/route",
                "description": "A POST route",
                "REST kind": "Post"
            })
        );

        assert_eq!(
            serialize(
                Route::delete("/route")
                    .description("A DELETE route")
                    .serialize_data()
            ),
            json!({
                "name": "/route",
                "description": "A DELETE route",
                "REST kind": "Delete"
            })
        );
    }

    #[test]
    fn test_all_hazards() {
        assert_eq!(
            serialize(
                Route::get("/route")
                    .description("A GET route")
                    .with_single_hazard(Hazard::FireHazard)
                    .serialize_data()
            ),
            json!({
                "name": "/route",
                "description": "A GET route",
                "REST kind": "Get",
                "hazards": [
                    "FireHazard"
                ],
            })
        );

        let mut hazards = Hazards::empty();
        hazards.add(Hazard::FireHazard);
        hazards.add(Hazard::AirPoisoning);

        assert_eq!(
            serialize(
                Route::get("/route")
                    .description("A GET route")
                    .with_hazards(hazards)
                    .serialize_data()
            ),
            json!({
                "name": "/route",
                "description": "A GET route",
                "REST kind": "Get",
                "hazards": [
                    "FireHazard",
                    "AirPoisoning",
                ],
            })
        );

        assert_eq!(
            serialize(
                Route::get("/route")
                    .description("A GET route")
                    .with_slice_hazards(&[Hazard::FireHazard, Hazard::AirPoisoning])
                    .serialize_data()
            ),
            json!({
                "name": "/route",
                "description": "A GET route",
                "REST kind": "Get",
                "hazards": [
                    "FireHazard",
                    "AirPoisoning",
                ],
            })
        );
    }

    #[test]
    fn test_all_inputs() {
        let expected = json!({
            "name": "/route",
            "description": "A GET route",
            "REST kind": "Get",
            "inputs": [
                {
                    "name": "rangeu64",
                    "kind": "RangeU64",
                    "structure": {
                        "RangeU64": {
                            "min": 0,
                            "max": 20,
                            "step": 1,
                            "default": 5
                        }
                    }
                },
                {
                    "name": "rangef64",
                    "kind": "RangeF64",
                    "structure": {
                        "RangeF64": {
                            "min": 0.0,
                            "max": 20.0,
                            "step": 0.1,
                            "default": 0.0
                        }
                    }
                }
            ],
            "REST kind": "Get"
        });

        assert_eq!(
            serialize(
                Route::get("/route")
                    .description("A GET route")
                    .input(Input::rangeu64_with_default("rangeu64", (0, 20, 1), 5))
                    .input(Input::rangef64("rangef64", (0., 20., 0.1)))
                    .serialize_data()
            ),
            expected
        );

        assert_eq!(
            serialize(
                Route::get("/route")
                    .description("A GET route")
                    .inputs([
                        Input::rangeu64_with_default("rangeu64", (0, 20, 1), 5),
                        Input::rangef64("rangef64", (0., 20., 0.1))
                    ])
                    .serialize_data()
            ),
            expected
        );
    }
}
