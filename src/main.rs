use ascot_firmware::server::AscotServer;

use ascot_firmware::action::{Action, Result, Task};
use ascot_firmware::device::light::Light;

struct TurnLightOn(f64);

impl Task for TurnLightOn {
    fn task(&self) -> Result<()> {
        println!("Light Brightness: {}", self.0);
        Ok(())
    }
}

struct TurnLightOff;

impl Task for TurnLightOff {
    fn task(&self) -> Result<()> {
        println!("Turning light off...");
        Ok(())
    }
}

struct Toggle;

impl Task for Toggle {
    fn task(&self) -> Result<()> {
        println!("Toggle light...");
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let turn_light_on = Action::put("on", TurnLightOn(4.0)).description("Turn light on.");
    let turn_light_off = Action::put("off", TurnLightOff).description("Turn light off.");
    let toggle = Action::put("toggle", Toggle).description("Toggle a light.");

    let device = Light::new(turn_light_on, turn_light_off)
        .add_action(toggle)
        .build();

    AscotServer::new(device)
        .run()
        .await
        .expect("Failed running the server.");
}
