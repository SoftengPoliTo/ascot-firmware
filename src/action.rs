use std::error::Error;
use std::fmt;
use std::sync::Arc;

/// A `[Task]` error.
#[derive(Debug)]
pub struct TaskError(pub(crate) Box<dyn Error + 'static>);

/// A specialized `Result` type.
pub type Result<T> = std::result::Result<T, TaskError>;

/// A trait to perform a task on a smart home device.
pub trait Task {
    /// The task which will be performed on the smart home device.
    fn task(&self) -> Result<()>;
}

impl fmt::Debug for dyn Task + Send + Sync + 'static {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl PartialEq for dyn Task + Send + Sync + 'static {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl Eq for dyn Task + Send + Sync + 'static {}

impl std::hash::Hash for dyn Task + Send + Sync + 'static {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {}
}

// `REST` API kind.
#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) enum RestKind {
    // `GET` API.
    Get,
    // `PUT` API.
    Put,
    // `POST` API.
    Post,
}

/// A smart home device action performs a determined task when a `REST` API is
/// being called on a specific server route.
#[derive(Debug, Eq)]
pub struct Action {
    // REST API kind.
    pub(crate) rest_kind: RestKind,
    // Server route.
    pub(crate) route: &'static str,
    // Task.
    pub(crate) task: Arc<dyn Task + Send + Sync + 'static>,
    // Description.
    pub(crate) description: Option<&'static str>,
}

impl PartialEq for Action {
    fn eq(&self, other: &Self) -> bool {
        self.route == other.route && self.rest_kind == other.rest_kind
    }
}

impl std::hash::Hash for Action {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.rest_kind.hash(state);
        self.route.hash(state);
    }
}

impl Action {
    /// Creates a new `[Action]` through a REST `GET` API.
    pub fn get(route: &'static str, task: impl Task + Send + Sync + 'static) -> Self {
        Self {
            rest_kind: RestKind::Get,
            route,
            task: Arc::new(task),
            description: None,
        }
    }

    /// Creates a new `[Action]` through a REST `PUT` API.
    pub fn put(route: &'static str, task: impl Task + Send + Sync + 'static) -> Self {
        Self {
            rest_kind: RestKind::Put,
            route,
            task: Arc::new(task),
            description: None,
        }
    }

    /// Creates a new `[Action]` through a REST `POST` API.
    pub fn post(route: &'static str, task: impl Task + Send + Sync + 'static) -> Self {
        Self {
            rest_kind: RestKind::Post,
            route,
            task: Arc::new(task),
            description: None,
        }
    }

    /// Sets the action description.
    pub fn description(mut self, description: &'static str) -> Self {
        self.description = Some(description);
        self
    }
}
