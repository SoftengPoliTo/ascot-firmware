use std::collections::HashMap;
use std::net::IpAddr;

use ascot_library::device::{DeviceEnvironment, DeviceKind};
use ascot_library::route::RouteConfigs;

use crate::request::{create_requests_senders, RequestSender};

pub(crate) fn build_device_address(scheme: &str, address: &IpAddr, port: u16) -> String {
    format!("{scheme}://{address}:{port}")
}

/// Device network information.
///
/// All data needed to contact a device in a network.
pub struct NetworkInformation {
    /// Device complete name.
    pub name: String,
    /// Device addresses.
    pub addresses: Vec<IpAddr>,
    /// Device port.
    pub port: u16,
    /// Device properties.
    pub properties: HashMap<String, String>,
}

impl NetworkInformation {
    pub(crate) const fn new(
        name: String,
        addresses: Vec<IpAddr>,
        port: u16,
        properties: HashMap<String, String>,
    ) -> Self {
        Self {
            name,
            addresses,
            port,
            properties,
        }
    }
}

/// Device description.
///
/// All data needed to describe a device.
pub struct Description {
    /// Device kind.
    pub kind: DeviceKind,
    /// Device environment.
    pub environment: DeviceEnvironment,
    /// Device main route.
    pub main_route: String,
    /// Device last reachable address.
    pub last_reachable_address: String,
}

impl Description {
    pub(crate) const fn new(
        kind: DeviceKind,
        environment: DeviceEnvironment,
        main_route: String,
        last_reachable_address: String,
    ) -> Self {
        Self {
            kind,
            environment,
            main_route,
            last_reachable_address,
        }
    }
}

/// A compliant device.
pub struct Device {
    // Information needed to contact a device in a network.
    network_info: NetworkInformation,
    // All data needed to describe a device.
    description: Description,
    // All device requests.
    requests: HashMap<String, RequestSender>,
}

impl Device {
    /// Creates a [`Device`] from [`NetworkInformation`], [`Description`],
    /// and [`RouteConfigs`] data.
    ///
    /// This method might be useful to create a device from data contained in
    /// a database.
    #[must_use]
    #[inline]
    pub fn new(
        network_info: NetworkInformation,
        description: Description,
        route_configs: RouteConfigs,
    ) -> Self {
        let requests = create_requests_senders(
            route_configs,
            &description.last_reachable_address,
            &description.main_route,
            description.environment,
        );

        // TODO: Check if the last reachable address works or it is better to
        // build a new one.

        Self {
            network_info,
            description,
            requests,
        }
    }

    /// Returns an immutable reference to [`NetworkInformation`].
    #[must_use]
    pub const fn network_info(&self) -> &NetworkInformation {
        &self.network_info
    }

    /// Returns an immutable reference to [`Description`].
    #[must_use]
    pub const fn description(&self) -> &Description {
        &self.description
    }

    /// Returns the number of available requests for a device.
    #[must_use]
    #[inline]
    pub fn requests_count(&self) -> usize {
        self.requests.len()
    }

    /// Returns the [`RequestSender`] associated with the given route.
    ///
    /// If [`None`], the given route **does not** exist.
    #[must_use]
    #[inline]
    pub fn request(&self, route: &str) -> Option<&RequestSender> {
        self.requests.get(route)
    }

    pub(crate) const fn init(
        network_info: NetworkInformation,
        description: Description,
        requests: HashMap<String, RequestSender>,
    ) -> Self {
        Self {
            network_info,
            description,
            requests,
        }
    }
}

/// A collection of [`Device`]s.
pub struct Devices(Vec<Device>);

impl Default for Devices {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for Devices {
    type Item = Device;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Devices {
    type Item = &'a Device;
    type IntoIter = std::slice::Iter<'a, Device>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Devices {
    /// Creates a [`Device`]s collection.
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Creates [`Devices`] from a vector of [`Device`]s.
    #[must_use]
    pub const fn from_devices(devices: Vec<Device>) -> Self {
        Self(devices)
    }

    /// Adds a [`Device`].
    #[inline]
    pub fn add(&mut self, device: Device) {
        self.0.push(device);
    }

    /// Checks whether the collection is empty.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of [`Device`] contained in a collection.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Gets a [`Device`] reference identified by the given index.
    #[must_use]
    #[inline]
    pub fn get(&self, index: usize) -> Option<&Device> {
        self.0.get(index)
    }

    /// Returns an iterator over [`Device`]s.
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, Device> {
        self.0.iter()
    }
}
