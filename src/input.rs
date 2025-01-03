use serde::{Deserialize, Serialize};

use crate::collections::Collection;

/// All supported [`Input`] kinds.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum InputKind {
    /// A [`bool`].
    Bool,
    /// A [`u8`].
    U8,
    /// A [`u64`] range.
    RangeU64,
    /// A [`f64`] range.
    RangeF64,
}

/// The structure of an [`InputKind`].
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum InputStructure {
    /// A [`bool`] default value.
    Bool { default: bool },
    /// A [`u8`] default value.
    U8 { default: u8 },
    /// A [`u64`] range with a **[minimum, maximum, step, default]**
    /// set of values.
    RangeU64 {
        min: u64,
        max: u64,
        step: u64,
        default: u64,
    },
    /// A [`f64`] range with a **[minimum, maximum, step, default]**
    /// set of values.
    RangeF64 {
        min: f64,
        max: f64,
        step: f64,
        default: f64,
    },
}

#[cfg(feature = "std")]
mod input_data {
    use super::*;

    /// Input data.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct InputData {
        /// Name.
        pub name: alloc::borrow::Cow<'static, str>,
        /// Input kind.
        #[serde(rename = "kind")]
        pub kind: InputKind,
        /// Input structure.
        #[serde(rename = "structure")]
        pub structure: InputStructure,
    }

    impl InputData {
        pub(super) fn new(input: Input) -> Self {
            Self {
                name: input.name.into(),
                kind: input.kind,
                structure: input.structure,
            }
        }
    }

    /// A collection of [`InputData`]s.
    pub type InputsData = crate::collections::OutputCollection<InputData>;
}

#[cfg(not(feature = "std"))]
mod input_data {
    use super::*;

    /// Input data.
    #[derive(Debug, Clone, Serialize)]
    pub struct InputData {
        /// Name.
        pub name: &'static str,
        /// Input kind.
        #[serde(rename = "kind")]
        pub kind: InputKind,
        /// Input structure.
        #[serde(rename = "structure")]
        pub structure: InputStructure,
    }

    impl InputData {
        pub(super) fn new(input: Input) -> Self {
            Self {
                name: input.name,
                kind: input.kind,
                structure: input.structure,
            }
        }
    }

    /// A collection of [`InputData`]s.
    pub type InputsData = crate::collections::SerialCollection<InputData>;
}

impl core::cmp::PartialEq for input_data::InputData {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl core::cmp::Eq for input_data::InputData {}

impl core::hash::Hash for input_data::InputData {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl From<Input> for input_data::InputData {
    fn from(input: Input) -> Self {
        Self::new(input)
    }
}

pub use input_data::InputData;
pub use input_data::InputsData;

/// All supported inputs.
#[derive(Debug, Clone, Copy)]
pub struct Input {
    // Name.
    pub(crate) name: &'static str,
    // Input kind.
    kind: InputKind,
    // Input structure.
    structure: InputStructure,
}

impl core::cmp::PartialEq for Input {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl core::cmp::Eq for Input {}

impl core::hash::Hash for Input {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Input {
    /// Creates a [`bool`] input.
    #[must_use]
    #[inline]
    pub fn bool(name: &'static str, default: bool) -> Self {
        Self {
            name,
            kind: InputKind::Bool,
            structure: InputStructure::Bool { default },
        }
    }

    /// Creates an [`u8`] input.
    #[must_use]
    #[inline]
    pub fn u8(name: &'static str, default: u8) -> Self {
        Self {
            name,
            kind: InputKind::U8,
            structure: InputStructure::U8 { default },
        }
    }

    /// Creates an [`u64`] range without a default value.
    #[must_use]
    #[inline]
    pub fn rangeu64(name: &'static str, range: (u64, u64, u64)) -> Self {
        Self::rangeu64_with_default(name, range, 0)
    }

    /// Creates an [`u64`] range with a default value.
    #[must_use]
    #[inline]
    pub fn rangeu64_with_default(name: &'static str, range: (u64, u64, u64), default: u64) -> Self {
        Self {
            name,
            kind: InputKind::RangeU64,
            structure: InputStructure::RangeU64 {
                min: range.0,
                max: range.1,
                step: range.2,
                default,
            },
        }
    }

    /// Creates an [`f64`] range without a default value.
    #[must_use]
    #[inline]
    pub fn rangef64(name: &'static str, range: (f64, f64, f64)) -> Self {
        Self::rangef64_with_default(name, range, 0.0)
    }

    /// Creates an [`f64`] range with a default value.
    #[must_use]
    #[inline]
    pub fn rangef64_with_default(name: &'static str, range: (f64, f64, f64), default: f64) -> Self {
        Self {
            name,
            kind: InputKind::RangeF64,
            structure: InputStructure::RangeF64 {
                min: range.0,
                max: range.1,
                step: range.2,
                default,
            },
        }
    }
}

/// A collection of [`Input`]s.
pub type Inputs = Collection<Input>;

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::serialize;

    use super::{Input, InputData};

    #[test]
    fn test_all_inputs() {
        assert_eq!(
            serialize(InputData::from(Input::bool("bool", true))),
            json!({
                "name": "bool",
                "kind": "Bool",
                "structure": {
                    "Bool": {
                        "default": true
                    }
                }
            })
        );

        assert_eq!(
            serialize(InputData::from(Input::u8("u8", 0))),
            json!({
                "name": "u8",
                "kind": "U8",
                "structure": {
                    "U8": {
                        "default": 0
                    }
                }
            })
        );

        assert_eq!(
            serialize(InputData::from(Input::rangeu64_with_default(
                "rangeu64",
                (0, 20, 1),
                5
            ))),
            json!({
                "name": "rangeu64",
                "kind": "RangeU64",
                "structure": {
                    "RangeU64": {
                        "min": 0,
                        "max": 20,
                        "step": 1,
                        "default": 5,
                    }
                }
            })
        );

        assert_eq!(
            serialize(InputData::from(Input::rangef64_with_default(
                "rangef64",
                (0., 20., 0.1),
                5.
            ))),
            json!({
                "name": "rangef64",
                "kind": "RangeF64",
                "structure": {
                    "RangeF64": {
                        "min": 0.,
                        "max": 20.,
                        "step": 0.1,
                        "default": 5.,
                    }
                }
            })
        );
    }
}
