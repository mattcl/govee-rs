pub mod client;
pub mod endpoints;
pub mod models;

pub use client::GoveeClient;
pub use models::Color;

pub const DEFAULT_API_URL: &str = "https://developer-api.govee.com";
