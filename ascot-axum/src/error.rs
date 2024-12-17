use alloc::borrow::Cow;

/// All possible error kinds.
#[derive(Debug, Copy, Clone)]
pub enum ErrorKind {
    /// Service error.
    Service,
    /// Not found address.
    NotFoundAddress,
    /// Serialize/Deserialize error.
    Serialization,
    /// An `Ascot` library error.
    AscotLibrary,
    /// Light error.
    Light,
    /// Fridge error.
    Fridge,
    /// External error.
    ///
    /// An error caused by an external dependency.
    External,
}

impl ErrorKind {
    pub(crate) const fn description(self) -> &'static str {
        match self {
            ErrorKind::Service => "Service",
            ErrorKind::NotFoundAddress => "Not Found Address",
            ErrorKind::Serialization => "Serialization",
            ErrorKind::AscotLibrary => "Ascot Library",
            ErrorKind::Light => "Light",
            ErrorKind::Fridge => "Fridge",
            ErrorKind::External => "External",
        }
    }
}

impl core::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Error kind: {}", self.description())
    }
}

/// Library error.
pub struct Error {
    kind: ErrorKind,
    info: Cow<'static, str>,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.format(f)
    }
}

impl core::fmt::Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.format(f)
    }
}

impl Error {
    /// Creates an [`Error`] passing a specific [`ErrorKind`] and a description.
    pub fn new(kind: ErrorKind, info: impl Into<Cow<'static, str>>) -> Self {
        Self {
            kind,
            info: info.into(),
        }
    }

    /// Creates an [`Error`] for [`ErrorKind::External`] passing a specific
    /// description.
    pub fn external(info: impl Into<Cow<'static, str>>) -> Self {
        Self::new(ErrorKind::External, info)
    }

    fn format(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "{}", self.kind)?;
        write!(f, "Cause: {}", self.info)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::new(ErrorKind::Serialization, e.to_string())
    }
}

impl From<ascot_library::Error> for Error {
    fn from(e: ascot_library::Error) -> Self {
        Self::new(ErrorKind::AscotLibrary, e.to_string())
    }
}

/// A specialized [`Result`] type for [`Error`].
pub type Result<T> = core::result::Result<T, Error>;
