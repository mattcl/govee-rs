use std::{borrow::Cow, collections::HashSet, ops::Deref, str::FromStr};

use hex_color::HexColor;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct BaseResponse<T>
where
    T: 'static,
{
    pub data: T,
}

pub type AnySuccessResponse = BaseResponse<Value>;

/// Control commands that can be issued against govee devices.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ControlCommand {
    /// Toggling power state.
    Turn,

    /// Adjusting brightness.
    Brightness,

    /// Adjusting color.
    Color,

    /// Adjusting color temperature.
    ColorTem,
}

/// A representation of a Govee device.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Device {
    pub model: String,
    pub device: String,
    #[serde(rename = "deviceName")]
    pub name: String,
    pub controllable: bool,
    pub retrievable: bool,
    #[serde(rename = "supportCmds")]
    pub supported_commands: HashSet<ControlCommand>,
}

impl Device {
    /// Check if this device supports the specified [ControlCommand].
    pub fn supports(&self, command: &ControlCommand) -> bool {
        self.supported_commands.contains(command)
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Devices {
    pub devices: Vec<Device>,
}

impl Deref for Devices {
    type Target = Vec<Device>;

    fn deref(&self) -> &Self::Target {
        &self.devices
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct DeviceState {
    pub device: String,
    pub model: String,
    pub properties: Vec<DeviceProperty>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PowerState {
    Off,
    On,
}

/// A RGB color.
///
/// # Examples
/// ```
/// use std::str::FromStr;
/// use govee_rs::Color;
///
/// assert_eq!(Color::default(), Color {r: 0, g: 0, b: 0});
///
/// // can be made from a tuple of `u8` values in (R, G, B)
/// let fromTuple = Color::from((10, 20, 30));
/// assert_eq!(fromTuple, Color {r: 10, g: 20, b: 30});
///
/// // can be parsed from a hex string
/// let parsed = Color::from_str("#0AFF06").unwrap();
/// assert_eq!(parsed, Color {r: 10, g: 255, b: 6});
///
/// let parsed = Color::parse("#12FF07").unwrap();
/// assert_eq!(parsed, Color {r: 18, g: 255, b: 7});
/// ```
#[derive(Debug, Clone, Copy, Eq, Default, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Parse a color from the given hex string.
    pub fn parse(s: &str) -> Result<Self, hex_color::ParseHexColorError> {
        s.parse()
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(value: (u8, u8, u8)) -> Self {
        Self {
            r: value.0,
            g: value.1,
            b: value.2,
        }
    }
}

impl From<HexColor> for Color {
    fn from(value: HexColor) -> Self {
        Self {
            r: value.r,
            g: value.g,
            b: value.b,
        }
    }
}

impl FromStr for Color {
    type Err = hex_color::ParseHexColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex_color = HexColor::parse(s)?;
        Ok(hex_color.into())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum DeviceProperty {
    Online {
        online: bool,
    },
    #[serde(rename_all = "camelCase")]
    PowerState {
        power_state: PowerState,
    },
    Brightness {
        brightness: u64,
    },
    Color {
        color: Color,
    },
    #[serde(rename_all = "camelCase")]
    ColorTem {
        color_tem: u64,
    },
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ControlRequest<'a> {
    pub device: Cow<'a, str>,
    pub model: Cow<'a, str>,
    pub cmd: ControlCmd,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "name", content = "value")]
pub enum ControlCmd {
    Turn(PowerState),
    Brightness(u64),
    Color(Color),
    ColorTem(u64),
}

#[cfg(test)]
mod tests {
    mod device_property {
        use super::super::*;

        #[test]
        fn serialization() {
            let prop = DeviceProperty::Online { online: false };
            let ser = serde_json::to_string(&prop).unwrap();
            assert_eq!(&ser, "{\"online\":false}");

            let prop = DeviceProperty::PowerState {
                power_state: PowerState::Off,
            };
            let ser = serde_json::to_string(&prop).unwrap();
            assert_eq!(&ser, "{\"powerState\":\"off\"}");

            let prop = DeviceProperty::Brightness { brightness: 22 };
            let ser = serde_json::to_string(&prop).unwrap();
            assert_eq!(&ser, "{\"brightness\":22}");

            let prop = DeviceProperty::Color {
                color: Color {
                    r: 1,
                    g: 10,
                    b: 100,
                },
            };
            let ser = serde_json::to_string(&prop).unwrap();
            assert_eq!(&ser, "{\"color\":{\"r\":1,\"g\":10,\"b\":100}}");
        }

        #[test]
        fn deserialization() {
            let input = r#"
            [
                {"online": false},
                {"powerState": "on"},
                {"brightness": 44},
                {"color": {"r": 2, "g": 20, "b": 200}}
            ]"#;

            let _props: Vec<DeviceProperty> = serde_json::from_str(input).unwrap();
        }
    }
}
