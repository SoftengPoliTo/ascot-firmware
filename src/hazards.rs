use heapless::{FnvIndexSet, IndexSetIter};

use serde::{Deserialize, Serialize};

use crate::MAXIMUM_ELEMENTS;

/// Hazard data.
#[derive(Debug, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct HazardData<'a> {
    /// Identifier.
    pub id: u16,
    /// Name.
    pub name: &'a str,
    /// Description.
    pub description: &'a str,
    /// Category.
    pub category: CategoryData<'a>,
}

impl<'a> HazardData<'a> {
    fn new(id: u16, name: &'a str, description: &'a str, category: CategoryData<'a>) -> Self {
        Self {
            id,
            name,
            description,
            category,
        }
    }
}

impl<'a> core::cmp::PartialEq for HazardData<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// A collection of [`HazardData`]s.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HazardsData<'a>(#[serde(borrow)] FnvIndexSet<HazardData<'a>, MAXIMUM_ELEMENTS>);

impl<'a> HazardsData<'a> {
    /// Initializes a new [`HazardsData`] collection.
    pub fn init() -> Self {
        Self(FnvIndexSet::new())
    }

    /// Initializes a new [`HazardsData`] collection from [`Hazards`].
    pub fn from_hazards(hazards: &Hazards) -> Self {
        let mut hazards_data = Self::init();
        for hazard in hazards.iter() {
            let hazard_data = HazardData::new(
                hazard.id(),
                hazard.name(),
                hazard.description(),
                CategoryData::new(*hazard),
            );

            let _ = hazards_data.0.insert(hazard_data);
        }
        hazards_data
    }

    /// Adds a new [`HazardData`] to the [`HazardsData`] collection.
    pub fn add(&mut self, hazard_data: HazardData<'a>) {
        let _ = self.0.insert(hazard_data);
    }

    /// Whether the [`HazardsData`] collection is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Checks whether a [`HazardData`] is contained into [`HazardsData`].
    pub fn contains(&self, hazard_data: &HazardData) -> bool {
        self.0.contains(hazard_data)
    }

    /// Returns an iterator over [`HazardData`]s.
    pub fn iter(&self) -> IndexSetIter<'_, HazardData> {
        self.0.iter()
    }

    /// Merges the collection with another [`HazardsData`].
    pub fn merge(&mut self, hazards_data: &Self) {
        self.0 = self.0.union(&hazards_data.0).cloned().collect();
    }
}

/// All possible hazards for a device task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Hazard {
    /// The execution may release toxic gases.
    AirPoisoning,
    /// The execution may cause oxygen deficiency by gaseous substances.
    Asphyxia,
    /// The execution authorises the app to record and save a video with audio on persistent storage.
    AudioVideoRecordAndStore,
    /// The execution authorises the app to obtain a video stream with audio.
    AudioVideoStream,
    /// The execution enables a device that consumes electricity.
    ElectricEnergyConsumption,
    /// The execution may cause an explosion.
    Explosion,
    /// The execution may cause fire.
    FireHazard,
    /// The execution enables a device that consumes gas.
    GasConsumption,
    /// The execution authorises the app to get and save information about the app's energy impact on the device the app runs on.
    LogEnergyConsumption,
    /// The execution authorises the app to get and save information about the app's duration of use.
    LogUsageTime,
    /// The execution authorises the app to use payment information and make a periodic payment.
    PaySubscriptionFee,
    /// The execution may cause an interruption in the supply of electricity.
    PowerOutage,
    /// The execution may lead to exposure to high voltages.
    PowerSurge,
    /// The execution authorises the app to get and save user inputs.
    RecordIssuedCommands,
    /// The execution authorises the app to get and save information about the user's preferences.
    RecordUserPreferences,
    /// The execution authorises the app to use payment information and make a payment transaction.
    SpendMoney,
    /// The execution may lead to rotten food.
    SpoiledFood,
    /// The execution authorises the app to read the display output and take screenshots of it.
    TakeDeviceScreenshots,
    /// The execution authorises the app to use a camera and take photos.
    TakePictures,
    /// The execution disables a protection mechanism and unauthorised individuals may physically enter home.
    UnauthorisedPhysicalAccess,
    /// The execution enables a device that consumes water.
    WaterConsumption,
    /// The execution allows water usage which may lead to flood.
    WaterFlooding,
}

impl core::fmt::Display for Hazard {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.name().fmt(f)
    }
}

impl Hazard {
    /// Returns an [`Hazard`] name.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::AirPoisoning => "Air Poisoning",
            Self::Asphyxia => "Asphyxia",
            Self::AudioVideoRecordAndStore => "Audio Video Record And Store",
            Self::AudioVideoStream => "Audio Video Stream",
            Self::ElectricEnergyConsumption => "Electric Energy Consumption",
            Self::Explosion => "Explosion",
            Self::FireHazard => "Fire Hazard",
            Self::GasConsumption => "Gas Consumption",
            Self::LogEnergyConsumption => "Log Energy Consumption",
            Self::LogUsageTime => "Log Usage Time",
            Self::PaySubscriptionFee => "Pay Subscription Fee",
            Self::PowerOutage => "Power Outage",
            Self::PowerSurge => "Power Surge",
            Self::RecordIssuedCommands => "Record Issued Commands",
            Self::RecordUserPreferences => "Record User Preferences",
            Self::SpendMoney => "Spend Money",
            Self::SpoiledFood => "Spoiled Food",
            Self::TakeDeviceScreenshots => "Take Device Screenshots",
            Self::TakePictures => "Take Pictures",
            Self::UnauthorisedPhysicalAccess => "Unauthorised Physical Access",
            Self::WaterConsumption => "Water Consumption",
            Self::WaterFlooding => "Water Flooding",
        }
    }

    /// Returns an [`Hazard`] description.
    pub const fn description(&self) -> &'static str {
        match self {
            Self::AirPoisoning => "The execution may release toxic gases.",
            Self::Asphyxia => "The execution may cause oxygen deficiency by gaseous substances.",
            Self::AudioVideoRecordAndStore => "The execution authorises the app to record and save a video with audio on persistent storage.",
            Self::AudioVideoStream => "The execution authorises the app to obtain a video stream with audio.",
            Self::ElectricEnergyConsumption => "The execution enables a device that consumes electricity.",
            Self::Explosion => "The execution may cause an explosion.",
            Self::FireHazard => "The execution may cause fire.",
            Self::GasConsumption => "The execution enables a device that consumes gas.",
            Self::LogEnergyConsumption => "The execution authorises the app to get and save information about the app's energy impact on the device the app runs on.",
            Self::LogUsageTime => "The execution authorises the app to get and save information about the app's duration of use.",
            Self::PaySubscriptionFee => "The execution authorises the app to use payment information and make a periodic payment.",
            Self::PowerOutage => "The execution may cause an interruption in the supply of electricity.",
            Self::PowerSurge => "The execution may lead to exposure to high voltages.",
            Self::RecordIssuedCommands => "The execution authorises the app to get and save user inputs.",
            Self::RecordUserPreferences => "The execution authorises the app to get and save information about the user's preferences.",
            Self::SpendMoney => "The execution authorises the app to use payment information and make a payment transaction.",
            Self::SpoiledFood => "The execution may lead to rotten food.",
            Self::TakeDeviceScreenshots => "The execution authorises the app to read the display output and take screenshots of it.",
            Self::TakePictures => "The execution authorises the app to use a camera and take photos.",
            Self::UnauthorisedPhysicalAccess => "The execution disables a protection mechanism and unauthorised individuals may physically enter home.",
            Self::WaterConsumption => "The execution enables a device that consumes water.",
            Self::WaterFlooding => "The execution allows water usage which may lead to flood.",
        }
    }

    /// Returns the [`Category`] associated with an [`Hazard`].
    ///
    /// An hazard **must** be associated with **only** one category.
    pub const fn category(&self) -> Category {
        match self {
            Self::AirPoisoning => Category::Safety,
            Self::Asphyxia => Category::Safety,
            Self::AudioVideoRecordAndStore => Category::Privacy,
            Self::AudioVideoStream => Category::Privacy,
            Self::ElectricEnergyConsumption => Category::Financial,
            Self::Explosion => Category::Safety,
            Self::FireHazard => Category::Safety,
            Self::GasConsumption => Category::Financial,
            Self::LogEnergyConsumption => Category::Privacy,
            Self::LogUsageTime => Category::Privacy,
            Self::PaySubscriptionFee => Category::Financial,
            Self::PowerOutage => Category::Safety,
            Self::PowerSurge => Category::Safety,
            Self::RecordIssuedCommands => Category::Privacy,
            Self::RecordUserPreferences => Category::Privacy,
            Self::SpendMoney => Category::Financial,
            Self::SpoiledFood => Category::Safety,
            Self::TakeDeviceScreenshots => Category::Privacy,
            Self::TakePictures => Category::Privacy,
            Self::UnauthorisedPhysicalAccess => Category::Safety,
            Self::WaterConsumption => Category::Financial,
            Self::WaterFlooding => Category::Safety,
        }
    }

    /// Returns the identifier associated with an [`Hazard`].
    pub const fn id(&self) -> u16 {
        match self {
            Self::AirPoisoning => 0,
            Self::Asphyxia => 1,
            Self::AudioVideoRecordAndStore => 2,
            Self::AudioVideoStream => 3,
            Self::ElectricEnergyConsumption => 4,
            Self::Explosion => 5,
            Self::FireHazard => 6,
            Self::GasConsumption => 7,
            Self::LogEnergyConsumption => 8,
            Self::LogUsageTime => 9,
            Self::PaySubscriptionFee => 10,
            Self::PowerOutage => 11,
            Self::PowerSurge => 12,
            Self::RecordIssuedCommands => 13,
            Self::RecordUserPreferences => 14,
            Self::SpendMoney => 15,
            Self::SpoiledFood => 16,
            Self::TakeDeviceScreenshots => 17,
            Self::TakePictures => 18,
            Self::UnauthorisedPhysicalAccess => 19,
            Self::WaterConsumption => 20,
            Self::WaterFlooding => 21,
        }
    }

    /// Returns an [`Hazard`] from an integer identifier.
    ///
    /// The value is [`None`] whenever the identifier does not exist or
    /// it is not correct.
    pub const fn from_id(id: u16) -> Option<Self> {
        match id {
            0 => Some(Self::AirPoisoning),
            1 => Some(Self::Asphyxia),
            2 => Some(Self::AudioVideoRecordAndStore),
            3 => Some(Self::AudioVideoStream),
            4 => Some(Self::ElectricEnergyConsumption),
            5 => Some(Self::Explosion),
            6 => Some(Self::FireHazard),
            7 => Some(Self::GasConsumption),
            8 => Some(Self::LogEnergyConsumption),
            9 => Some(Self::LogUsageTime),
            10 => Some(Self::PaySubscriptionFee),
            11 => Some(Self::PowerOutage),
            12 => Some(Self::PowerSurge),
            13 => Some(Self::RecordIssuedCommands),
            14 => Some(Self::RecordUserPreferences),
            15 => Some(Self::SpendMoney),
            16 => Some(Self::SpoiledFood),
            17 => Some(Self::TakeDeviceScreenshots),
            18 => Some(Self::TakePictures),
            19 => Some(Self::UnauthorisedPhysicalAccess),
            20 => Some(Self::WaterConsumption),
            21 => Some(Self::WaterFlooding),
            _ => None,
        }
    }
}

/// A collection of [`Hazard`]s.
#[derive(Debug, Clone)]
pub struct Hazards(FnvIndexSet<Hazard, MAXIMUM_ELEMENTS>);

impl Hazards {
    /// Initializes a new [`Hazards`] collection.
    pub fn init() -> Self {
        Self(FnvIndexSet::new())
    }

    /// Adds a new [`Hazard`] to the [`Hazards`] collection.
    pub fn add(&mut self, hazard: Hazard) {
        let _ = self.0.insert(hazard);
    }

    /// Whether the [`Hazards`] collection is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Checks whether an [`Hazard`] is contained into
    /// the [`Hazards`] collection.
    pub fn contains(&self, hazard: Hazard) -> bool {
        self.0.contains(&hazard)
    }

    /// Returns an iterator over [`Hazard`]s.
    pub fn iter(&self) -> IndexSetIter<'_, Hazard> {
        self.0.iter()
    }
}

/// Hazard category data.
#[derive(Debug, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct CategoryData<'a> {
    /// Name.
    pub name: &'a str,
    /// Description.
    pub description: &'a str,
}

impl<'a> CategoryData<'a> {
    fn new(hazard: Hazard) -> Self {
        Self {
            name: hazard.category().name(),
            description: hazard.category().description(),
        }
    }
}

impl<'a> core::cmp::PartialEq for CategoryData<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

/// Hazard categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum Category {
    /// Category which includes all the financial-related hazards.
    Financial,
    /// Category which includes all the privacy-related hazards.
    Privacy,
    /// Category which includes all the safety-related hazards.
    Safety,
}

impl core::fmt::Display for Category {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.name().fmt(f)
    }
}

impl Category {
    /// Returns a [`Category`] name.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Financial => "Financial",
            Self::Privacy => "Privacy",
            Self::Safety => "Safety",
        }
    }

    /// Returns a [`Category`] description.
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Financial => "Category which includes all the financial-related hazards.",
            Self::Privacy => "Category which includes all the privacy-related hazards.",
            Self::Safety => "Category which includes all the safety-related hazards.",
        }
    }

    /// Returns all [`Hazard`]s associated with a [`Category`].
    pub const fn hazards(&self) -> &[Hazard] {
        match self {
            Self::Financial => &[
                Hazard::ElectricEnergyConsumption,
                Hazard::GasConsumption,
                Hazard::PaySubscriptionFee,
                Hazard::SpendMoney,
                Hazard::WaterConsumption,
            ],
            Self::Privacy => &[
                Hazard::AudioVideoRecordAndStore,
                Hazard::AudioVideoStream,
                Hazard::LogEnergyConsumption,
                Hazard::LogUsageTime,
                Hazard::RecordIssuedCommands,
                Hazard::RecordUserPreferences,
                Hazard::TakeDeviceScreenshots,
                Hazard::TakePictures,
            ],
            Self::Safety => &[
                Hazard::AirPoisoning,
                Hazard::Asphyxia,
                Hazard::Explosion,
                Hazard::FireHazard,
                Hazard::PowerOutage,
                Hazard::PowerSurge,
                Hazard::SpoiledFood,
                Hazard::UnauthorisedPhysicalAccess,
                Hazard::WaterFlooding,
            ],
        }
    }
}
