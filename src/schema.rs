use std::collections::{HashMap, HashSet};
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{GoveeError, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct DevicesResponse {
    pub data: Option<Devices>,
    pub message: String,
    pub code: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Devices {
    pub devices: Vec<Device>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub device: String,
    pub model: String,
    #[serde(rename = "deviceName")]
    pub name: String,
    pub controllable: bool,
    pub retrievable: bool,
    #[serde(rename = "supportCmds")]
    pub supported_commands: HashSet<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceStateResponse {
    pub data: DeviceState,
    pub message: String,
    pub code: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceState {
    pub device: String,
    pub model: String,
    pub properties: Vec<HashMap<String, Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PowerRequest<'a> {
    device: &'a str,
    model: &'a str,
    cmd: HashMap<&'a str, &'a str>,
}

#[derive(Debug, Clone)]
pub enum PowerState {
    On,
    Off,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color {
    pub r: u32,
    pub g: u32,
    pub b: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColorRequest {
    device: String,
    model: String,
    cmd: ColorInner,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColorInner {
    name: String,
    value: Color,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColorTemRequest {
    device: String,
    model: String,
    cmd: ColorTemInner,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColorTemInner {
    name: String,
    value: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BrightnessRequest<'a> {
    device: &'a str,
    model: &'a str,
    cmd: BrightnessInner,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BrightnessInner {
    name: String,
    value: u32,
}

impl Device {
    pub fn supports(&self, command: Command) -> bool {
        match command {
            Command::Turn => self.supported_commands.contains("turn"),
            Command::Brightness => self.supported_commands.contains("brightness"),
            Command::Color => self.supported_commands.contains("color"),
            Command::ColorTem => self.supported_commands.contains("colorTem"),
        }
    }

    pub fn toggle_request(&self, state: PowerState) -> Result<PowerRequest> {
        if !self.supports(Command::Turn) {
            return Err(GoveeError::Unsupported(Command::Turn, self.clone()));
        }

        let mut cmd = HashMap::new();
        cmd.insert("name", "turn");
        match state {
            PowerState::On => cmd.insert("value", "on"),
            PowerState::Off => cmd.insert("value", "off"),
        };

        Ok(PowerRequest {
            device: &self.device,
            model: &self.model,
            cmd: cmd,
        })
    }

    pub fn color_request(&self, color: &Color) -> Result<ColorRequest> {
        if !self.supports(Command::Color) {
            return Err(GoveeError::Unsupported(Command::Color, self.clone()));
        }

        Ok(ColorRequest {
            device: self.device.clone(),
            model: self.model.clone(),
            cmd: ColorInner {
                name: "color".to_string(),
                value: color.clone(),
            },
        })
    }

    pub fn color_temperature_request(&self, value: u32) -> Result<ColorTemRequest> {
        if !self.supports(Command::ColorTem) {
            return Err(GoveeError::Unsupported(Command::ColorTem, self.clone()));
        }

        if value < 2000 || value > 9000 {
            return Err(GoveeError::Error(
                "Color temperatures must be from 2000 to 9000 inclusive".to_string(),
            ));
        }

        Ok(ColorTemRequest {
            device: self.device.clone(),
            model: self.model.clone(),
            cmd: ColorTemInner {
                name: "color".to_string(),
                value: value,
            },
        })
    }

    pub fn brightness_request(&self, value: u32) -> Result<BrightnessRequest> {
        if !self.supports(Command::Brightness) {
            return Err(GoveeError::Unsupported(Command::Brightness, self.clone()));
        }

        Ok(BrightnessRequest {
            device: &self.device,
            model: &self.model,
            cmd: BrightnessInner {
                name: "brightness".to_string(),
                value: value,
            },
        })
    }
}

impl fmt::Display for Devices {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut devices = String::new();
        for device in &self.devices {
            devices += &format!("{}\n", device);
        }
        write!(f, "{}", devices)
    }
}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({}, {}, {:#?})",
            self.model, self.name, self.supported_commands
        )
    }
}

#[derive(Debug)]
pub enum Command {
    Turn,
    Brightness,
    Color,
    ColorTem,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Turn => write!(f, "turn"),
            Command::Brightness => write!(f, "brightness"),
            Command::Color => write!(f, "color"),
            Command::ColorTem => write!(f, "colorTem"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn supports_checks_supported_commands() {
        let mut cmds = HashSet::new();
        cmds.insert("turn".to_string());

        let device = fake_device(cmds);
        assert!(device.supports(Command::Turn));
        assert!(!device.supports(Command::Brightness));
        assert!(!device.supports(Command::Color));
        assert!(!device.supports(Command::ColorTem));

        let mut cmds = HashSet::new();
        cmds.insert("brightness".to_string());

        let device = fake_device(cmds);
        assert!(!device.supports(Command::Turn));
        assert!(device.supports(Command::Brightness));
        assert!(!device.supports(Command::Color));
        assert!(!device.supports(Command::ColorTem));

        let mut cmds = HashSet::new();
        cmds.insert("color".to_string());

        let device = fake_device(cmds);
        assert!(!device.supports(Command::Turn));
        assert!(!device.supports(Command::Brightness));
        assert!(device.supports(Command::Color));
        assert!(!device.supports(Command::ColorTem));

        let mut cmds = HashSet::new();
        cmds.insert("colorTem".to_string());

        let device = fake_device(cmds);
        assert!(!device.supports(Command::Turn));
        assert!(!device.supports(Command::Brightness));
        assert!(!device.supports(Command::Color));
        assert!(device.supports(Command::ColorTem));

        let mut cmds = HashSet::new();
        cmds.insert("turn".to_string());
        cmds.insert("brightness".to_string());
        cmds.insert("color".to_string());
        cmds.insert("colorTem".to_string());

        let device = fake_device(cmds);
        assert!(device.supports(Command::Turn));
        assert!(device.supports(Command::Brightness));
        assert!(device.supports(Command::Color));
        assert!(device.supports(Command::ColorTem));
    }

    #[test]
    fn color_temperature_must_be_between_2000_and_9000() {
        let mut cmds = HashSet::new();
        cmds.insert("colorTem".to_string());
        let device = fake_device(cmds);
        assert!(device.color_temperature_request(1999).is_err());
        assert!(device.color_temperature_request(2000).is_ok());
        assert!(device.color_temperature_request(5000).is_ok());
        assert!(device.color_temperature_request(9000).is_ok());
        assert!(device.color_temperature_request(9001).is_err());
    }
}
