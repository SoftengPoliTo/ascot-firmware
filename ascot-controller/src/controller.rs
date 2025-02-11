use crate::device::Devices;
use crate::discovery::Discovery;
use crate::error::Error;
use crate::policy::Policy;

// TODO: As of now, we are using an id which is represented by the device
// position in a list. We are going to use a String instead.

/// A controller for sending requests.
///
/// It sends or does not send requests to devices according to:
///
/// - A privacy policy
/// - Some scheduled programs
///
/// When the controller receives a response from a device, it forwards it
/// directly to the caller.
pub struct Controller {
    discovery: Discovery,
    devices: Devices,
    privacy_policy: Policy,
    // scheduler: Scheduler,
}

impl Controller {
    /// Creates a [`Controller`] given a [`Discovery`] configuration.
    #[must_use]
    #[inline]
    pub fn new(discovery: Discovery) -> Self {
        Self {
            discovery,
            devices: Devices::new(),
            privacy_policy: Policy::init(),
        }
    }

    /// Creates a [`Controller`] from a [`Discovery`] configuration and
    /// a set of initial [`Devices`].
    ///
    /// This method might be useful for [`Devices`] retrieved from a database.
    #[must_use]
    #[inline]
    pub fn from_devices(discovery: Discovery, devices: Devices) -> Self {
        Self {
            discovery,
            devices,
            privacy_policy: Policy::init(),
        }
    }

    /// Sets a [`Policy`].
    #[must_use]
    #[inline]
    pub fn policy(mut self, privacy_policy: Policy) -> Self {
        self.privacy_policy = privacy_policy;
        self
    }

    /// Change [`Policy`].
    #[inline]
    pub fn change_policy(&mut self, privacy_policy: Policy) {
        self.privacy_policy = privacy_policy;
    }

    /// Discovers all available [`Devices`] in a network.
    ///
    /// # Errors
    ///
    /// ## Discovery Errors
    ///
    /// During a discovery process some of the most common errors are the
    /// impossibility to connect to a network, disable a particular interface,
    /// or close the discovery process itself.
    ///
    /// ## Sending Requests Errors
    ///
    /// While sending a request to a device to obtain the description of its
    /// structure and all of its routes, some network failures or
    /// timeouts can prevent the effective sending.
    /// Moreover, the same issues can also affect the return response.
    #[inline]
    pub async fn discover(&mut self) -> Result<(), Error> {
        self.devices = self.discovery.discover().await?;
        Ok(())
    }

    /// Returns discovered [`Devices`].
    #[must_use]
    pub const fn devices(&self) -> &Devices {
        &self.devices
    }
}
/*

// Configure a controller
// TODO: Define an ID on devices to tell which kind of devices switch on and
// off!!!!
// Run a parallel thread to perform that.
// Device Id, Action id, [start_time, end_time]
.schedule_task(Device.id("DiningRoomLight"), "/on", [19:30, 22]));
.configure()?; --> Checks if the passed device id, action and everything else is
correct

controller.change_scheduled_tasks(...)?;


let device_runtime = controller.get_device(id); Result<Device, RuntimeError>
let action_runtime = device_runtime.get_action("/on"); Result<ActionRuntime, RuntimeError>

// Run with the input parameters.
let response = action_runtime.run_with_params(Inputs::empty().insert_f64("value", 5.0)).await?; Result<Response; RuntimeError>

or

// Run without any params (using default input values)
let response = action_runtime.run_without_params().await?; Result<ReponseRuntimeError>

*/
