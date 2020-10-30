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
pub struct ColorInner {
   name: String,
   value: Color,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColorRequest {
    device: String,
    model: String,
    cmd: ColorInner,
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

        Ok(
            PowerRequest {
                device: &self.device,
                model: &self.model,
                cmd: cmd,
            }
        )
    }

    pub fn color_request(&self, color: &Color) -> Result<ColorRequest> {
        if !self.supports(Command::Color) {
            return Err(GoveeError::Unsupported(Command::Color, self.clone()));
        }

        Ok(
            ColorRequest {
                device: self.device.clone(),
                model: self.model.clone(),
                cmd: ColorInner {
                    name: "color".to_string(),
                    value: color.clone(),
                }
            }
        )
    }

    pub fn brightness_request(&self, value: u32) -> Result<BrightnessRequest> {
        if !self.supports(Command::Brightness) {
            return Err(GoveeError::Unsupported(Command::Brightness, self.clone()));
        }

        Ok(
            BrightnessRequest {
                device: &self.device,
                model: &self.model,
                cmd: BrightnessInner {
                    name: "brightness".to_string(),
                    value: value,
                }
            }
        )
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
        write!(f, "({}, {})", self.model, self.name)
    }
}

#[derive(Debug)]
pub enum Command {
    Turn,
    Brightness,
    Color,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Turn => write!(f, "turn"),
            Command::Brightness => write!(f, "brightness"),
            Command::Color => write!(f, "color"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

}
