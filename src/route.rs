use serde::{Deserialize, Serialize};

use crate::collections::{Collection, OutputCollection};
use crate::error::Result;
use crate::hazards::{Hazard, Hazards, HazardsData};
use crate::input::{Input, Inputs, InputsData};
use crate::strings::MiniString;

use crate::MAXIMUM_ELEMENTS;

/// Route inputs writing modes.
///
/// Route inputs can be added to a route in a different way depending on the
/// underlying implementation of a server.
///
/// A server, indeed, might produce a response for a request if and only if
/// a route presents a specific structure for its inputs.
#[derive(Debug, Clone, Copy)]
pub enum RouteMode {
    /// Linear routes. Inputs are written on after the other in the route:
    /// i.e. route/input1/input2
    ///
    /// If a server implements another kind of linear schema, the produced
    /// route should be changed accordingly by a developer.
    Linear,
}

impl RouteMode {
    // Some servers requires a symbol to represent inputs.
    #[inline]
    fn join_input(self, route: &mut MiniString, text: &str, symbol: Option<&str>) -> Result<()> {
        match self {
            Self::Linear => {
                route.push("/")?;
                if let Some(symbol) = symbol {
                    route.push(symbol)?;
                }
                route.push(text)
            }
        }
    }
}

/// Route data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteData<'a> {
    /// Name.
    pub name: &'a str,
    /// Description.
    pub description: Option<&'a str>,
    /// Stateless parameter.
    ///
    /// The route does not modify the internal state of a device.
    ///
    /// The default value is [`false`].
    pub stateless: bool,
    /// Inputs associated with a route..
    #[serde(skip_serializing_if = "InputsData::is_empty")]
    pub inputs: InputsData<'a>,
}

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

/// A server route configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig<'a> {
    /// Route.
    #[serde(flatten, borrow)]
    pub data: RouteData<'a>,
    /// Kind of a `REST` API.
    #[serde(rename = "REST kind")]
    pub rest_kind: RestKind,
    /// Hazards data.
    #[serde(skip_serializing_if = "HazardsData::is_empty")]
    pub hazards: HazardsData<'a>,
}

impl<'a> PartialEq for RouteConfig<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.data.name.eq(other.data.name) && self.rest_kind == other.rest_kind
    }
}

impl<'a> Eq for RouteConfig<'a> {}

impl<'a> core::hash::Hash for RouteConfig<'a> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.data.name.hash(state);
        self.rest_kind.hash(state);
    }
}

/// A collection of [`RouteConfig`]s.
pub type RouteConfigs<'a> = OutputCollection<RouteConfig<'a>>;

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
    // Stateless parameter.
    stateless: bool,
    // Inputs.
    inputs: Inputs,
    // Route with inputs.
    inputs_route: MiniString,
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
    pub const fn get(route: &'static str) -> Self {
        Self::init(RestKind::Get, route)
    }

    /// Creates a new [`Route`] through a REST `PUT` API.
    pub const fn put(route: &'static str) -> Self {
        Self::init(RestKind::Put, route)
    }

    /// Creates a new [`Route`] through a REST `POST` API.
    pub const fn post(route: &'static str) -> Self {
        Self::init(RestKind::Post, route)
    }

    /// Sets the route description.
    pub const fn description(mut self, description: &'static str) -> Self {
        self.description = Some(description);
        self
    }

    /// Sets the route as stateless.
    pub const fn stateless(mut self) -> Self {
        self.stateless = true;
        self
    }

    /// Sets a single [`Input`].
    #[inline(always)]
    pub fn input(mut self, input: Input) -> Self {
        self.inputs.add(input);
        self
    }

    /// Sets more [`Input`]s.
    #[inline]
    pub fn inputs<const N: usize>(mut self, inputs: [Input; N]) -> Self {
        inputs.into_iter().take(MAXIMUM_ELEMENTS).for_each(|input| {
            self.inputs.add(input);
        });
        self
    }

    /// Changes the route joining together the given inputs according to the
    /// schema defined by the [`RouteMode`] argument.
    ///
    /// An optional symbol to identify an input can be added if a server
    /// requires that kind of schema.
    ///
    /// This operation is performed **only** for the `GET` route mode.
    ///
    ///
    /// A route remains unchanged in the following cases:
    /// - Any other route kind has been set
    /// - No inputs have been provided
    /// - An internal error occurred
    pub fn join_inputs(&mut self, route_mode: RouteMode, symbol: Option<&str>) {
        if self.rest_kind != RestKind::Get
            || !self.inputs_route.is_empty()
            || self.inputs_route.push(self.route).is_err()
        {
            return;
        }

        for input in self.inputs.iter() {
            // If an error occurred adding an input, reset the input string,
            // and break the loop.
            if route_mode
                .join_input(&mut self.inputs_route, input.name, symbol)
                .is_err()
            {
                self.inputs_route = MiniString::empty();
                break;
            }
        }
    }

    /// Returns route.
    #[inline(always)]
    pub fn route(&self) -> &str {
        if self.inputs_route.is_empty() {
            self.route
        } else {
            self.inputs_route.as_str()
        }
    }

    /// Returns [`RestKind`].
    pub const fn kind(&self) -> RestKind {
        self.rest_kind
    }

    const fn init(rest_kind: RestKind, route: &'static str) -> Self {
        Self {
            route,
            rest_kind,
            description: None,
            stateless: false,
            inputs: Inputs::empty(),
            inputs_route: MiniString::empty(),
        }
    }
}

/// A route with its associated hazards.
#[derive(Debug)]
pub struct RouteHazards {
    /// Route.
    pub route: Route,
    /// Hazards.
    pub hazards: Hazards,
}

impl core::cmp::PartialEq for RouteHazards {
    fn eq(&self, other: &Self) -> bool {
        self.route.eq(&other.route)
    }
}

impl core::cmp::Eq for RouteHazards {}

impl core::hash::Hash for RouteHazards {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.route.hash(state);
    }
}

impl RouteHazards {
    /// Creates a [`RouteHazards`] instance.
    pub const fn new(route: Route, hazards: Hazards) -> Self {
        Self { route, hazards }
    }

    /// Creates a [`RouteHazards`] without any hazard.
    pub const fn no_hazards(route: Route) -> Self {
        Self {
            route,
            hazards: Hazards::empty(),
        }
    }

    /// Initializes a [`RouteHazards`] with a single [`Hazard`].
    #[inline]
    pub fn single_hazard(route: Route, hazard: Hazard) -> Self {
        Self {
            route,
            hazards: Hazards::init(hazard),
        }
    }

    /// Initializes a [`RouteHazards`] with some [`Hazard`]s.
    #[inline]
    pub fn with_hazards(route: Route, hazards: &'static [Hazard]) -> Self {
        Self {
            route,
            hazards: Hazards::init_with_elements(hazards),
        }
    }

    /// Serializes [`RouteHazards`] data.
    #[inline]
    pub fn serialize_data(&self) -> RouteConfig {
        RouteConfig {
            rest_kind: self.route.rest_kind,
            hazards: HazardsData::from(&self.hazards),
            data: RouteData {
                name: self.route.route(),
                description: self.route.description,
                stateless: self.route.stateless,
                inputs: InputsData::from(&self.route.inputs),
            },
        }
    }
}

/// A collection of [`RouteHazards`]s.
pub type RoutesHazards = Collection<RouteHazards>;
