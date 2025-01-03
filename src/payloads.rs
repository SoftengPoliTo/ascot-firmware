use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::actions::ActionError;
use crate::device::DeviceInfo;
use crate::strings::ShortString;

/// Payload kinds for an action response.
#[derive(Serialize, Deserialize)]
pub enum PayloadKind {
    /// A short message to notify a receiver that an action terminated
    /// correctly.
    Ok,
    /// Serial data (i.e. JSON).
    ///
    /// This payload adds further information to an action response.
    Serial,
    /// Informative data to describe a device (i.e. JSON).
    ///
    /// This payload contains additional information on a device.
    Info,
    /// Stream of data expressed as a sequence of bytes.
    Stream,
}

/// An `Ok` payload sends a boolean as action response to notify a receiver that
/// a device action has terminated correctly.
#[derive(Serialize, Deserialize)]
pub struct OkPayload {
    action_terminated_correctly: bool,
}

impl OkPayload {
    /// Creates an [`OkPayload`].
    #[must_use]
    #[inline]
    pub fn ok() -> Self {
        Self {
            action_terminated_correctly: true,
        }
    }
}

/// Serial payload.
///
/// This payload adds further information to an action response.
#[derive(Serialize, Deserialize)]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub struct SerialPayload<T: DeserializeOwned> {
    #[serde(flatten)]
    data: T,
}

impl<T: Serialize + DeserializeOwned> SerialPayload<T> {
    /// Creates a [`SerialPayload`].
    #[must_use]
    pub const fn new(data: T) -> Self {
        Self { data }
    }
}

/// Informative payload.
///
/// This payload contains additional information on a device.
#[derive(Serialize, Deserialize)]
pub struct InfoPayload {
    #[serde(flatten)]
    data: DeviceInfo,
}

impl InfoPayload {
    /// Creates a [`InfoPayload`].
    #[must_use]
    pub const fn new(data: DeviceInfo) -> Self {
        Self { data }
    }
}

/// A payload containing information about an error occurred within an action.
///
/// It describes the kind of error, the cause, and optional information.
#[derive(Serialize, Deserialize)]
pub struct ErrorPayload {
    /// Action error type.
    pub error: ActionError,
    /// Error description.
    pub description: ShortString,
    /// Information about an error.
    pub info: Option<ShortString>,
}

impl ErrorPayload {
    /// Creates an [`ErrorPayload`] with a specific [`ActionError`] and
    /// a description.
    ///
    /// If an error occurs, an empty description is returned.
    #[must_use]
    #[inline]
    pub fn with_description(error: ActionError, description: &'static str) -> Self {
        Self {
            error,
            description: ShortString::infallible(description),
            info: None,
        }
    }

    /// Creates an [`ErrorPayload`] with a specific [`ActionError`], an
    /// error description, and additional information about the error.
    ///
    /// If this method fails for some internal reasons, empty description and
    /// information are returned.
    #[must_use]
    #[inline]
    pub fn with_description_error(
        error: ActionError,
        description: &'static str,
        info: &str,
    ) -> Self {
        Self {
            error,
            description: ShortString::infallible(description),
            info: Some(ShortString::infallible(info)),
        }
    }

    /// Creates an [`ErrorPayload`] for invalid data with a description.
    ///
    /// If this method fails for some internal reasons, an empty description
    /// is returned.
    #[must_use]
    #[inline]
    pub fn invalid_data(description: &'static str) -> Self {
        Self::with_description(ActionError::InvalidData, description)
    }

    /// Creates an [`ErrorPayload`] for invalid data with a description and
    /// additional information about the error.
    ///
    /// If this method fails for some internal reasons, empty description and
    /// information are returned.
    #[must_use]
    #[inline]
    pub fn invalid_data_with_error(description: &'static str, info: &str) -> Self {
        Self::with_description_error(ActionError::InvalidData, description, info)
    }

    /// Creates an [`ErrorPayload`] for an internal error with a description.
    ///
    /// If this method fails for some internal reasons, an empty description
    /// is returned.
    #[must_use]
    #[inline]
    pub fn internal(description: &'static str) -> Self {
        Self::with_description(ActionError::Internal, description)
    }

    /// Creates an [`ErrorPayload`] for an internal error with a description and
    /// additional information about the error.
    ///
    /// If this method fails for some internal reasons, empty description and
    /// information are returned.
    #[must_use]
    #[inline]
    pub fn internal_with_error(description: &'static str, info: &str) -> Self {
        Self::with_description_error(ActionError::Internal, description, info)
    }
}

#[cfg(test)]
mod tests {

    use serde_json::json;

    use crate::energy::{Energy, WaterUseEfficiency};
    use crate::serialize;

    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    struct Serial {
        value: u32,
    }

    #[test]
    fn test_ok_payload() {
        assert_eq!(
            serialize(OkPayload::ok()),
            json!({
                "action_terminated_correctly": true,
            })
        );
    }

    #[test]
    fn test_serial_payload() {
        assert_eq!(
            serialize(SerialPayload::new(Serial { value: 42 })),
            json!({
                "value": 42,
            })
        );
    }

    #[test]
    fn test_info_payload() {
        let energy =
            Energy::init_with_water_use_efficiency(WaterUseEfficiency::init_with_gpp(42.0));

        assert_eq!(
            serialize(InfoPayload::new(DeviceInfo::empty().add_energy(energy))),
            json!({
                 "energy": {
                     "water-use-efficiency": {
                         "gross-primary-productivity": 42.0,
                         "penman-monteith-equation": null,
                         "water-equivalent-ratio": null
                     }
                 }
            })
        );
    }

    #[test]
    fn test_error_payload() {
        let error = ErrorPayload::with_description(
            ActionError::InvalidData,
            "Invalid data error description",
        );

        assert_eq!(
            serialize(error),
            json!({
                 "error": "Invalid Data",
                 "description": "Invalid data error description",
                 "info": null,
            })
        );
    }
}
