use heapless::{FnvIndexSet, IndexSetIter, Vec};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::hazards::{Hazards, HazardsData};
use crate::input::{Input, Inputs, InputsData};
use crate::{MiniString, MAXIMUM_ELEMENTS};

/// Route inputs writing modes.
///
/// Route inputs can be represented in different ways according to
/// the server used to run a device. A server, indeed, can parse and reply to
/// routes whose inputs present a specific structure.
#[derive(Debug, Clone, Copy)]
pub enum RouteMode {
    /// Linear routes. Inputs are written on after the other i.e. route/input1/input2
    Linear,
}

impl RouteMode {
    #[inline]
    fn join_input(self, route: &mut MiniString, text: &str) -> Result<()> {
        match self {
            Self::Linear => {
                route.push("/:")?;
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

/// A collection of [`RouteConfig`]s.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfigs<'a>(#[serde(borrow)] Vec<RouteConfig<'a>, MAXIMUM_ELEMENTS>);

impl<'a> RouteConfigs<'a> {
    /// Initializes a new [`RouteConfigs`] collection.
    pub fn init() -> Self {
        Self(Vec::new())
    }

    /// Adds a new [`RouteConfig`] to the [`RouteConfigs`] collection.
    pub fn add(&mut self, route_config: RouteConfig<'a>) {
        let _ = self.0.push(route_config);
    }

    /// Whether the [`RouteConfigs`] collection is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an iterator over [`RouteConfig`]s.
    pub fn iter(&self) -> core::slice::Iter<'_, RouteConfig<'a>> {
        self.0.iter()
    }
}

/// A route to execute a precise smart home device task.
///
/// It represents a specific `REST` API which invokes an external
/// process to be run on a device.
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

impl Eq for Route {}

impl PartialEq for Route {
    fn eq(&self, other: &Self) -> bool {
        self.route == other.route && self.rest_kind == other.rest_kind
    }
}

impl core::hash::Hash for Route {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.route.hash(state);
        self.rest_kind.hash(state);
    }
}

impl Route {
    /// Creates a new [`Route`] through a REST `GET` API.
    pub fn get(route: &'static str) -> Self {
        Self::init(RestKind::Get, route)
    }

    /// Creates a new [`Route`] through a REST `PUT` API.
    pub fn put(route: &'static str) -> Self {
        Self::init(RestKind::Put, route)
    }

    /// Creates a new [`Route`] through a REST `POST` API.
    pub fn post(route: &'static str) -> Self {
        Self::init(RestKind::Post, route)
    }

    /// Sets the action description.
    pub fn description(mut self, description: &'static str) -> Self {
        self.description = Some(description);
        self
    }

    /// Sets the route as stateless.
    pub fn stateless(mut self) -> Self {
        self.stateless = true;
        self
    }

    /// Sets a single [`Input`].
    pub fn input(mut self, input: Input) -> Self {
        self.inputs.add(input);
        self
    }

    /// Sets more [`Input`]s.
    pub fn inputs<const N: usize>(mut self, inputs: [Input; N]) -> Self {
        inputs.into_iter().take(MAXIMUM_ELEMENTS).for_each(|input| {
            self.inputs.add(input);
        });
        self
    }

    /// Computes a new route joining together the given inputs according to the
    /// chosen route mode.
    ///
    /// A route remains unchanged in the following cases:
    /// - A `POST` REST kind has been set
    /// - No inputs have been provided
    /// - An internal error occurred
    pub fn join_inputs(&mut self, route_mode: RouteMode) {
        if self.rest_kind == RestKind::Post
            || !self.inputs_route.is_empty()
            || self.inputs_route.push(self.route).is_err()
        {
            return;
        }

        for input in self.inputs.iter() {
            // If an error occurred adding an input, reset the input string,
            // and break the loop.
            if route_mode
                .join_input(&mut self.inputs_route, input.name)
                .is_err()
            {
                self.inputs_route = MiniString::empty();
                break;
            }
        }
    }

    /// Returns route.
    pub fn route(&self) -> &str {
        if self.inputs_route.is_empty() {
            self.route
        } else {
            self.inputs_route.as_str()
        }
    }

    /// Returns [`RestKind`].
    pub fn kind(&self) -> RestKind {
        self.rest_kind
    }

    fn init(rest_kind: RestKind, route: &'static str) -> Self {
        Self {
            route,
            rest_kind,
            description: None,
            stateless: false,
            inputs: Inputs::init(),
            inputs_route: MiniString::empty(),
        }
    }
}

/// A route with its associated hazards.
#[derive(Debug)]
pub struct RouteHazards {
    /// Route.
    route: Route,
    /// Hazards.
    hazards: Hazards,
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
    /// Creates a new [`RouteHazards`].
    pub fn new(route: Route, hazards: Hazards) -> Self {
        Self { route, hazards }
    }

    /// Serializes [`RouteHazards`] data.
    pub fn serialize_data(&self) -> RouteConfig {
        RouteConfig {
            rest_kind: self.route.rest_kind,
            hazards: HazardsData::from_hazards(&self.hazards),
            data: RouteData {
                name: self.route.route(),
                description: self.route.description,
                stateless: self.route.stateless,
                inputs: InputsData::from_inputs(&self.route.inputs),
            },
        }
    }
}

/// A collection of [`RouteHazards`]s.
#[derive(Debug)]
pub struct RoutesHazards(FnvIndexSet<RouteHazards, MAXIMUM_ELEMENTS>);

impl RoutesHazards {
    /// Initializes a new [`RoutesHazards`] collection.
    pub fn init() -> Self {
        Self(FnvIndexSet::new())
    }

    /// Adds a new [`RouteHazards`] to the [`RoutesHazards`] collection.
    pub fn add(&mut self, route_hazards: RouteHazards) {
        let _ = self.0.insert(route_hazards);
    }

    /// Whether the [`RoutesHazards`] collection is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Checks whether a [`RouteHazards`] is contained into [`RoutesHazards`].
    pub fn contains(&self, route_hazards: &RouteHazards) -> bool {
        self.0.contains(route_hazards)
    }

    /// Returns an iterator over [`RouteHazards`]s.
    pub fn iter(&self) -> IndexSetIter<'_, RouteHazards> {
        self.0.iter()
    }
}
