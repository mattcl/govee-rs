use std::borrow::Cow;

use derive_builder::Builder;
use gen_api_wrapper::{endpoint_prelude::Endpoint, params::QueryParams};
use http::Method;

use crate::models::{ControlCmd, ControlRequest};

/// An endpoint for getting the list of devices.
#[derive(Debug, Clone, Default)]
pub struct DevicesEndpoint;

impl Endpoint for DevicesEndpoint {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> std::borrow::Cow<'static, str> {
        "v1/devices".into()
    }
}

impl DevicesEndpoint {
    pub fn new() -> Self {
        Self
    }
}

/// An endpoint for controlling a particular device.
#[derive(Debug, Clone, Builder)]
pub struct DeviceControlEndpoint<'a> {
    #[builder(setter(into))]
    device: Cow<'a, str>,

    #[builder(setter(into))]
    model: Cow<'a, str>,

    control_cmd: ControlCmd,
}

impl<'a> Endpoint for DeviceControlEndpoint<'a> {
    fn method(&self) -> Method {
        Method::PUT
    }

    fn endpoint(&self) -> std::borrow::Cow<'static, str> {
        "v1/devices/control".into()
    }

    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, gen_api_wrapper::error::BodyError> {
        let control_body = ControlRequest {
            device: self.device.clone(),
            model: self.model.clone(),
            cmd: self.control_cmd,
        };

        Ok(Some((
            "application/json",
            serde_json::to_vec(&control_body)?,
        )))
    }
}

impl<'a> DeviceControlEndpoint<'a> {
    pub fn builder() -> DeviceControlEndpointBuilder<'a> {
        DeviceControlEndpointBuilder::default()
    }
}

/// An endpoint for getting the state of a particular device.
#[derive(Debug, Clone, Builder)]
pub struct DeviceStateEndpoint<'a> {
    #[builder(setter(into))]
    device: Cow<'a, str>,

    #[builder(setter(into))]
    model: Cow<'a, str>,
}

impl<'a> Endpoint for DeviceStateEndpoint<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> std::borrow::Cow<'static, str> {
        "v1/devices/state".into()
    }

    fn parameters(&self) -> QueryParams {
        let mut params = QueryParams::default();
        params.push("device", &self.device);
        params.push("model", &self.model);
        params
    }
}

impl<'a> DeviceStateEndpoint<'a> {
    pub fn builder() -> DeviceStateEndpointBuilder<'a> {
        DeviceStateEndpointBuilder::default()
    }
}
