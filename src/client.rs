use async_trait::async_trait;
use bytes::Bytes;
use gen_api_wrapper::{
    client::{AsyncClient, RestClient},
    error::ApiError,
    query::AsyncQuery,
};
use http::{HeaderMap, HeaderValue, Response};
use reqwest::Client;
use thiserror::Error;
use url::Url;

use crate::{
    endpoints::{DeviceControlEndpoint, DevicesEndpoint, DeviceStateEndpoint},
    models::{AnySuccessResponse, BaseResponse, Color, ControlCmd, Device, Devices, PowerState, DeviceState},
};

#[derive(Debug, Error)]
pub enum GoveeError {
    #[error("failed to parse url: {}", source)]
    UrlParse {
        #[from]
        source: url::ParseError,
    },
    #[error("error setting auth header: {}", source)]
    AuthError {
        #[from]
        source: AuthError,
    },
    #[error("failed to talk to the govee api: {}", source)]
    Communication {
        #[from]
        source: reqwest::Error,
    },
    #[error("http error: {}", status)]
    Http { status: reqwest::StatusCode },
    #[error("could not parse {} data from json: {}", typename, source)]
    DataType {
        #[source]
        source: serde_json::Error,
        typename: &'static str,
    },
    #[error("api error: {}", source)]
    Api {
        #[from]
        source: ApiError<RestError>,
    },
}

#[derive(Debug, Error)]
pub enum RestError {
    #[error("error setting auth headers: {}", source)]
    AuthError {
        #[from]
        source: AuthError,
    },
    #[error("failed to talk to the govee api: {}", source)]
    Communication {
        #[from]
        source: reqwest::Error,
    },
    #[error("http error: {}", source)]
    Http {
        #[from]
        source: http::Error,
    },
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("header value error: {}", source)]
    HeaderValue {
        #[from]
        source: http::header::InvalidHeaderValue,
    },
}

#[derive(Clone)]
struct Auth {
    api_key: String,
}

impl Auth {
    pub fn set_header<'a>(
        &self,
        headers: &'a mut HeaderMap<HeaderValue>,
    ) -> Result<&'a mut HeaderMap<HeaderValue>, AuthError> {
        let mut header_value = HeaderValue::from_str(&self.api_key)?;
        header_value.set_sensitive(true);
        headers.insert("Govee-API-Key", header_value);
        Ok(headers)
    }
}

/// A client for interacting with the GoveeApi.
///
/// Can either be used directly or as an argument to the endpoint structs.
#[derive(Clone)]
pub struct GoveeClient {
    client: Client,
    api_url: Url,
    auth: Auth,
}

impl GoveeClient {
    /// Make a new [GoveeClient].
    ///
    /// This will fail if the provided `api_url` does not parse.
    pub fn new(api_url: &str, api_key: &str) -> Result<Self, GoveeError> {
        let api_url = Url::parse(api_url)?;
        let client = Client::builder().build()?;

        Ok(Self {
            client,
            api_url,
            auth: Auth {
                api_key: api_key.into(),
            },
        })
    }

    /// Gets the [Devices] associated with the account specified by the key.
    pub async fn devices(&self) -> Result<Devices, GoveeError> {
        let endpoint = DevicesEndpoint::new();
        let wrapper: BaseResponse<Devices> = endpoint.query_async(self).await?;
        Ok(wrapper.data)
    }

    /// Convenience method for getting [DeviceState] for a particular [Device].
    pub async fn state(&self, device: &Device) -> Result<DeviceState, GoveeError> {
        let endpoint = DeviceStateEndpoint::builder()
            .device(&device.device)
            .model(&device.model)
            .build()
            .expect("This should have been safe");
        let wrapper: BaseResponse<DeviceState> = endpoint.query_async(self).await?;
        Ok(wrapper.data)
    }

    /// Convenience method for setting the power state of a particular [Device].
    pub async fn turn(&self, device: &Device, state: PowerState) -> Result<(), GoveeError> {
        self.control(device, ControlCmd::Turn(state)).await
    }

    /// Convenience method for setting the brightness of a particular [Device].
    pub async fn brightness(&self, device: &Device, brightness: u64) -> Result<(), GoveeError> {
        self.control(device, ControlCmd::Brightness(brightness))
            .await
    }

    /// Convenience method for setting the color of a particular [Device].
    pub async fn color(&self, device: &Device, color: Color) -> Result<(), GoveeError> {
        self.control(device, ControlCmd::Color(color)).await
    }

    /// Convenience method for setting the color temp of a particular [Device].
    pub async fn color_temp(&self, device: &Device, color_temp: u64) -> Result<(), GoveeError> {
        self.control(device, ControlCmd::ColorTem(color_temp)).await
    }

    async fn control(&self, device: &Device, cmd: ControlCmd) -> Result<(), GoveeError> {
        let endpoint = DeviceControlEndpoint::builder()
            .device(&device.device)
            .model(&device.model)
            .control_cmd(cmd)
            .build()
            .expect("This should have been safe");

        let _: AnySuccessResponse = endpoint.query_async(self).await?;

        Ok(())
    }
}

impl RestClient for GoveeClient {
    type Error = RestError;

    fn rest_endpoint(&self, endpoint: &str) -> Result<Url, ApiError<Self::Error>> {
        Ok(self.api_url.join(endpoint)?)
    }
}

#[async_trait]
impl AsyncClient for GoveeClient {
    async fn rest_async(
        &self,
        mut request: http::request::Builder,
        body: Vec<u8>,
    ) -> Result<Response<Bytes>, ApiError<<Self as RestClient>::Error>> {
        use futures_util::TryFutureExt;
        let call = || async {
            self.auth.set_header(request.headers_mut().unwrap())?;
            let http_request = request.body(body)?;
            let request = http_request.try_into()?;
            let rsp = self.client.execute(request).await?;

            let mut http_rsp = Response::builder()
                .status(rsp.status())
                .version(rsp.version());
            let headers = http_rsp.headers_mut().unwrap();
            for (key, value) in rsp.headers() {
                headers.insert(key, value.clone());
            }
            Ok(http_rsp.body(rsp.bytes().await?)?)
        };
        call().map_err(ApiError::client).await
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::models::{ControlCommand, ControlRequest};

    use super::*;
    use mockito::Server;

    fn fake_device() -> Device {
        Device {
            device: "34:20:03:15:82:ae".to_string(),
            model: "H6089".to_string(),
            name: "fake device".to_string(),
            controllable: true,
            retrievable: true,
            supported_commands: HashSet::from_iter(
                [
                    ControlCommand::Turn,
                    ControlCommand::Brightness,
                    ControlCommand::Color,
                    ControlCommand::ColorTem,
                ]
                .into_iter(),
            ),
        }
    }

    #[tokio::test]
    async fn devices() {
        let mut server = Server::new_async().await;
        let fake_api_key = "foobarbaz";
        let client = GoveeClient::new(&server.url(), fake_api_key).unwrap();

        let fake_response = r#"
            {
                "data": {
                    "devices": [
                        {
                            "device": "99:A5:A4:C1:38:29:DA:7B",
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

        let devices_mock = server
            .mock("GET", "/v1/devices?")
            .match_header("Govee-API-Key", fake_api_key)
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(fake_response)
            .create_async()
            .await;

        let devices = client.devices().await.unwrap();

        assert_eq!(devices.len(), 3);

        let expected = Device {
            device: "99:A5:A4:C1:38:29:DA:7B".into(),
            model: "H6159".into(),
            name: "test light".into(),
            controllable: true,
            retrievable: true,
            supported_commands: HashSet::from_iter(
                [
                    ControlCommand::Turn,
                    ControlCommand::Brightness,
                    ControlCommand::Color,
                    ControlCommand::ColorTem,
                ]
                .into_iter(),
            ),
        };

        assert_eq!(devices[0], expected);

        devices_mock.assert_async().await;
    }

    #[tokio::test]
    async fn state() {
        let mut server = Server::new_async().await;
        let fake_api_key = "foobarbaz";
        let client = GoveeClient::new(&server.url(), fake_api_key).unwrap();

        let device = fake_device();

        let fake_response = r#"
            {
                "data": {
                    "device": "34:20:03:15:82:ae",
                    "model": "H6089",
                    "properties": [
                        {"online": false},
                        {"powerState": "off"},
                        {"brightness": 82},
                        {
                            "color": {"r": 11, "g": 22, "b": 33 }
                        }
                    ]
                },
                "message": "Success",
                "code": 201
            }"#;

        let control_mock = server
            .mock("GET", "/v1/devices/state")
            .match_header("Govee-API-Key", fake_api_key)
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("device".into(), device.device.clone()),
                mockito::Matcher::UrlEncoded("model".into(), device.model.clone()),
            ]))
            .with_status(200)
            .with_body(fake_response)
            .create_async()
            .await;

        client.state(&device).await.unwrap();

        control_mock.assert_async().await;
    }

    #[tokio::test]
    async fn turn() {
        let mut server = Server::new_async().await;
        let fake_api_key = "foobarbaz";
        let client = GoveeClient::new(&server.url(), fake_api_key).unwrap();

        let device = fake_device();

        let control_request = ControlRequest {
            device: device.device.clone().into(),
            model: device.model.clone().into(),
            cmd: ControlCmd::Turn(PowerState::On),
        };

        let fake_response = r#"
            {
                "data": {},
                "message": "Success",
                "code": 201
            }"#;

        let control_mock = server
            .mock("PUT", "/v1/devices/control?")
            .match_header("Govee-API-Key", fake_api_key)
            .match_header("Content-Type", "application/json")
            .match_body(mockito::Matcher::Json(
                serde_json::to_value(&control_request).unwrap(),
            ))
            .with_status(201)
            .with_body(fake_response)
            .create_async()
            .await;

        client.turn(&device, PowerState::On).await.unwrap();

        control_mock.assert_async().await;
    }

    #[tokio::test]
    async fn brightness() {
        let mut server = Server::new_async().await;
        let fake_api_key = "foobarbaz";
        let client = GoveeClient::new(&server.url(), fake_api_key).unwrap();

        let device = fake_device();

        let control_request = ControlRequest {
            device: device.device.clone().into(),
            model: device.model.clone().into(),
            cmd: ControlCmd::Brightness(25),
        };

        let fake_response = r#"
            {
                "data": {},
                "message": "Success",
                "code": 201
            }"#;

        let control_mock = server
            .mock("PUT", "/v1/devices/control?")
            .match_header("Govee-API-Key", fake_api_key)
            .match_header("Content-Type", "application/json")
            .match_body(mockito::Matcher::Json(
                serde_json::to_value(&control_request).unwrap(),
            ))
            .with_status(201)
            .with_body(fake_response)
            .create_async()
            .await;

        client.brightness(&device, 25).await.unwrap();

        control_mock.assert_async().await;
    }

    #[tokio::test]
    async fn color() {
        let mut server = Server::new_async().await;
        let fake_api_key = "foobarbaz";
        let client = GoveeClient::new(&server.url(), fake_api_key).unwrap();

        let device = fake_device();

        let color = Color {
            r: 0,
            g: 195,
            b: 255,
        };

        let control_request = ControlRequest {
            device: device.device.clone().into(),
            model: device.model.clone().into(),
            cmd: ControlCmd::Color(color),
        };

        let fake_response = r#"
            {
                "data": {},
                "message": "Success",
                "code": 201
            }"#;

        let control_mock = server
            .mock("PUT", "/v1/devices/control?")
            .match_header("Govee-API-Key", fake_api_key)
            .match_header("Content-Type", "application/json")
            .match_body(mockito::Matcher::Json(
                serde_json::to_value(&control_request).unwrap(),
            ))
            .with_status(201)
            .with_body(fake_response)
            .create_async()
            .await;

        client.color(&device, color).await.unwrap();

        control_mock.assert_async().await;
    }

    #[tokio::test]
    async fn color_temp() {
        let mut server = Server::new_async().await;
        let fake_api_key = "foobarbaz";
        let client = GoveeClient::new(&server.url(), fake_api_key).unwrap();

        let device = fake_device();

        let control_request = ControlRequest {
            device: device.device.clone().into(),
            model: device.model.clone().into(),
            cmd: ControlCmd::ColorTem(1000),
        };

        let fake_response = r#"
            {
                "data": {},
                "message": "Success",
                "code": 201
            }"#;

        let control_mock = server
            .mock("PUT", "/v1/devices/control?")
            .match_header("Govee-API-Key", fake_api_key)
            .match_header("Content-Type", "application/json")
            .match_body(mockito::Matcher::Json(
                serde_json::to_value(&control_request).unwrap(),
            ))
            .with_status(201)
            .with_body(fake_response)
            .create_async()
            .await;

        client.color_temp(&device, 1000).await.unwrap();

        control_mock.assert_async().await;
    }
}
