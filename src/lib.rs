use crate::error::Result;
use crate::schema::{
    Color, Device, DeviceState, DeviceStateResponse, Devices, DevicesResponse, PowerState,
};

pub mod error;
pub mod schema;

pub const API_BASE: &str = "https://developer-api.govee.com/v1";

#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
enum Verb {
    GET,
    PUT,
}

#[derive(Debug)]
pub struct Client {
    base_url: String,
    api_key: String,
}

impl Client {
    pub fn new(base_url: &str, api_key: &str) -> Self {
        Client {
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
        }
    }

    fn base_request(&self, verb: Verb, path: &str) -> reqwest::blocking::RequestBuilder {
        let full_path = format!("{}/{}/", self.base_url, path);
        let client = reqwest::blocking::Client::new();

        let builder = match verb {
            Verb::GET => client.get(&full_path),
            Verb::PUT => client.put(&full_path),
        };

        builder.header("Govee-API-key", self.api_key.clone())
    }

    pub fn devices(&self) -> Result<Devices> {
        let res: DevicesResponse = self.base_request(Verb::GET, "devices").send()?.json()?;

        match res.data {
            Some(devices) => Ok(devices),
            None => Err(error::GoveeError::NoDevicesReturned()),
        }
    }

    pub fn state(&self, device: &Device) -> Result<DeviceState> {
        let res: DeviceStateResponse = self
            .base_request(Verb::GET, "devices/state")
            .query(&[("device", &device.device), ("model", &device.model)])
            .send()?
            .json()?;
        Ok(res.data)
    }

    pub fn toggle(&self, device: &Device, state: PowerState) -> Result<()> {
        let req_body = device.toggle_request(state)?;
        self.base_request(Verb::PUT, "devices/control")
            .json(&req_body)
            .send()?;
        Ok(())
    }

    pub fn set_color(&self, device: &Device, color: &Color) -> Result<()> {
        let req_body = device.color_request(color)?;
        self.base_request(Verb::PUT, "devices/control")
            .json(&req_body)
            .send()?;
        Ok(())
    }

    pub fn set_color_temperature(&self, device: &Device, value: u32) -> Result<()> {
        let req_body = device.color_temperature_request(value)?;
        self.base_request(Verb::PUT, "devices/control")
            .json(&req_body)
            .send()?;
        Ok(())
    }

    pub fn set_brightness(&self, device: &Device, value: u32) -> Result<()> {
        let req_body = device.brightness_request(value)?;
        self.base_request(Verb::PUT, "devices/control")
            .json(&req_body)
            .send()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::Method::*;
    use httpmock::MockServer;
    use std::collections::HashSet;

    #[test]
    fn base_request_adds_auth_headers() {
        let fake_api_key = "foobarbaz";
        let client = Client::new(API_BASE, fake_api_key);

        let request = client.base_request(Verb::GET, "mypath").build().unwrap();
        let headers = request.headers();
        let auth = headers.get("Govee-API-key");
        assert!(auth.is_some());
        assert_eq!(auth.unwrap(), &client.api_key);
    }

    #[test]
    fn devices_yields_devices() {
        let server = MockServer::start();
        let fake_api_key = "foobarbaz";
        let client = Client::new(&server.base_url(), fake_api_key);

        let fake_response = r#"
{
    "data": {
        "devices": [
            {
                "device": "99:E5:A4:C1:38:29:DA:7B",
                "model": "H6159",
                "deviceName": "test light",
                "controllable": true,
                "retrievable": true,
                "supportCmds": [
                    "turn",
                    "brightness",
                    "color",
                    "colorTem"
                ]
            },
            {
                "device": "C6:EA:B8:56:C8:C6:89:BE",
                "model": "H6188",
                "deviceName": "H6188_89BE",
                "controllable": true,
                "retrievable": true,
                "supportCmds": [
                    "turn",
                    "brightness",
                    "color",
                    "colorTem"
                ]
            },
            {
                "device": "34:20:03:2e:30:2b",
                "model": "H5081",
                "deviceName": "Smart Plug",
                "controllable": true,
                "retrievable": true,
                "supportCmds": [
                    "turn"
                ]
            }
        ]
    },
    "message": "Success",
    "code": 200
}"#;
        // Create a mock on the server.
        let devices_mock = server.mock(|when, then| {
            when.method(GET).path("/devices/");
            then.status(200)
                .header("Content-Type", "application/json")
                .body(fake_response);
        });

        // Send an HTTP request to the mock server. This simulates your code.
        let response = client.devices();
        assert!(response.is_ok());
        let devices = response.unwrap();
        assert_eq!(devices.devices.len(), 3);

        // Ensure the specified mock was called exactly one time.
        devices_mock.assert();
    }

    fn fake_device(cmds: HashSet<String>) -> Device {
        Device {
            device: "34:20:03:15:82:ae".to_string(),
            model: "H6089".to_string(),
            name: "fake device".to_string(),
            controllable: true,
            retrievable: true,
            supported_commands: cmds,
        }
    }

    #[test]
    fn toggle_sends_turn_request() {
        let server = MockServer::start();
        let fake_api_key = "foobarbaz";
        let client = Client::new(&server.base_url(), fake_api_key);

        let mut cmds = HashSet::new();
        cmds.insert("turn".to_string());

        let device = fake_device(cmds);

        let power_request = device.toggle_request(PowerState::On).unwrap();

        let control_mock = server.mock(|when, then| {
            when.method(PUT)
                .path("/devices/control/")
                .header("Content-Type", "application/json")
                .json_body_obj(&power_request);
            then.status(201).header("Content-Type", "application/json");
        });

        client.toggle(&device, PowerState::On).unwrap();

        control_mock.assert();
    }

    #[test]
    fn set_color_sends_color_request() {
        let server = MockServer::start();
        let fake_api_key = "foobarbaz";
        let client = Client::new(&server.base_url(), fake_api_key);

        let mut cmds = HashSet::new();
        cmds.insert("color".to_string());

        let device = fake_device(cmds);

        let color = Color {
            r: 0,
            g: 195,
            b: 255,
        };

        let color_request = device.color_request(&color).unwrap();

        let control_mock = server.mock(|when, then| {
            when.method(PUT)
                .path("/devices/control/")
                .header("Content-Type", "application/json")
                .json_body_obj(&color_request);
            then.status(201).header("Content-Type", "application/json");
        });

        client.set_color(&device, &color).unwrap();

        control_mock.assert();
    }

    #[test]
    fn set_color_temperature_sends_color_temperature_request() {
        let server = MockServer::start();
        let fake_api_key = "foobarbaz";
        let client = Client::new(&server.base_url(), fake_api_key);

        let mut cmds = HashSet::new();
        cmds.insert("colorTem".to_string());

        let device = fake_device(cmds);

        let color_request = device.color_temperature_request(2500).unwrap();

        let control_mock = server.mock(|when, then| {
            when.method(PUT)
                .path("/devices/control/")
                .header("Content-Type", "application/json")
                .json_body_obj(&color_request);
            then.status(201).header("Content-Type", "application/json");
        });

        client.set_color_temperature(&device, 2500).unwrap();

        control_mock.assert();
    }

    #[test]
    fn set_brightness_sends_brightness_request() {
        let server = MockServer::start();
        let fake_api_key = "foobarbaz";
        let client = Client::new(&server.base_url(), fake_api_key);

        let mut cmds = HashSet::new();
        cmds.insert("brightness".to_string());

        let device = fake_device(cmds);

        let brightness_request = device.brightness_request(22).unwrap();

        let control_mock = server.mock(|when, then| {
            when.method(PUT)
                .path("/devices/control/")
                .header("Content-Type", "application/json")
                .json_body_obj(&brightness_request);
            then.status(201).header("Content-Type", "application/json");
        });

        client.set_brightness(&device, 22).unwrap();

        control_mock.assert();
    }
}
