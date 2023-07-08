use std::{env, thread, time::Duration};

use govee_rs::{models::PowerState, GoveeClient};

#[tokio::main]
async fn main() {
    let key = env::var("GOVEE_KEY").expect("GOVEE_KEY not set");
    let client =
        GoveeClient::new("https://developer-api.govee.com", &key).expect("Failed to make client");
    let devices = client.devices().await.expect("Failed to fetch devices");

    for dev in devices.iter() {
        println!("{:#?}", dev);
    }

    let first = devices.first().expect("No devices returned");

    client.turn(first, PowerState::On).await.unwrap();

    thread::sleep(Duration::from_secs(5));

    client.turn(first, PowerState::Off).await.unwrap();
}
