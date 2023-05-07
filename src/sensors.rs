use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::{error::Result, AppState};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Sensor {
    #[serde(rename = "_id")]
    pub id: String,
    pub small_id: u8,
    thing_id: String,
    #[serde(rename = "type")]
    pub type_id: SensorType,
    last_update: u64,
    name: String,
    frequency: u32,
    can_id: u32,
    can_offset: u32,
    unit: Option<String>,
    upper_bound: Option<f64>,
    lower_bound: Option<f64>,
    upper_warning: Option<f64>,
    lower_warning: Option<f64>,
    upper_danger: Option<f64>,
    lower_danger: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SensorType {
    #[serde(rename = "?")]
    Bool, // ?
    #[serde(rename = "c")]
    Char, // c
    #[serde(rename = "B")]
    UnsignedByte, // B
    #[serde(rename = "h")]
    Short, // h
    #[serde(rename = "H")]
    UnsignedShort, // H
    #[serde(rename = "i")]
    Int, // i
    #[serde(rename = "I")]
    UnsignedInt, // I
    #[serde(rename = "f")]
    Float, // f
    #[serde(rename = "q")]
    LongLong, // q
    #[serde(rename = "Q")]
    UnsignedLongLong, // Q
    #[serde(rename = "d")]
    Double, // d
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum SensorValue {
    Bool(bool),
    Char(char),
    UnsignedByte(u8),
    Short(i16),
    UnsignedShort(u16),
    Int(i32),
    UnsignedInt(u32),
    Float(f32),
    LongLong(i64),
    UnsignedLongLong(u64),
    Double(f64),
}

#[derive(Serialize, Deserialize)]
struct SensorResponse {
    data: Vec<Sensor>,
    message: String,
}

impl Sensor {
    pub fn size(&self) -> usize {
        match self.type_id {
            SensorType::Bool | SensorType::Char | SensorType::UnsignedByte => 1,
            SensorType::Short | SensorType::UnsignedShort => 2,
            SensorType::Int | SensorType::UnsignedInt | SensorType::Float => 4,
            SensorType::LongLong | SensorType::UnsignedLongLong | SensorType::Double => 8,
        }
    }
}

pub async fn fetch_sensors(thing_id: &str, app: AppState, api_key: &str) -> Result<Vec<Sensor>> {
    let url = format!(
        "{}/api/database/sensors/thing/{thing_id}",
        app.config.gateway_url
    );

    let res = app
        .http
        .get(&url)
        .header("apiKey", api_key)
        .send()
        .await
        .context("failed to fetch sensors from database")?
        .json::<SensorResponse>()
        .await
        .context("failed to parse sensor response from database")?;

    Ok(res.data)
}
