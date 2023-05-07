use std::collections::HashMap;

use byteorder::{ByteOrder, LittleEndian};

use crate::sensors::{Sensor, SensorType, SensorValue};

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("Invalid message")]
    InvalidMessage,
    #[error("Invalid sensor ID")]
    InvalidSensorId,
    #[error("Invalid sensor value")]
    InvalidSensorValue,
}

pub fn parse_message(
    buf: &[u8],
    sensors: &[Sensor],
) -> Result<(u32, HashMap<String, SensorValue>), ParseError> {
    // If the message is less than 6 bytes, it must be invalid
    if buf.len() < 6 {
        return Err(ParseError::InvalidMessage);
    }

    // Data is in the format:
    // Sensor Count (1 byte) + Timestamp (4 bytes)
    // + Sensor IDs (Variable; Number of sensors)
    // + Sensor Data (Variable; Number of sensors and the byte width of values)
    let sensor_count = buf[0] as usize;
    let timestamp = u32::from_le_bytes([buf[1], buf[2], buf[3], buf[4]]);
    let small_ids = &buf[5..5 + sensor_count];

    let mut sensor_data = HashMap::new();
    sensor_data.insert("ts".to_string(), SensorValue::UnsignedInt(timestamp));

    let mut cursor = 5 + sensor_count;
    for small_id in small_ids.iter() {
        let sensor = if let Some(sensor) = sensors.iter().find(|s| s.small_id == *small_id) {
            sensor
        } else {
            return Err(ParseError::InvalidSensorId);
        };

        let size = sensor.size();
        let data = &buf[cursor..cursor + size];
        cursor += size;
        let value = parse_data(data, sensor)?;

        sensor_data.insert(small_id.to_string(), value);
    }

    Ok((timestamp, sensor_data))
}

pub fn parse_data(buf: &[u8], sensor: &Sensor) -> Result<SensorValue, ParseError> {
    if buf.len() != sensor.size() {
        return Err(ParseError::InvalidSensorValue);
    }

    // Conversion will panic if `buf` is not the correct length,
    // but that will be caught by the above check
    let value = match sensor.type_id {
        SensorType::Bool => SensorValue::Bool(buf[0] != 0),
        SensorType::Char => SensorValue::Char(buf[0] as char),
        SensorType::UnsignedByte => SensorValue::UnsignedByte(buf[0]),
        SensorType::Short => SensorValue::Short(LittleEndian::read_i16(buf)),
        SensorType::UnsignedShort => SensorValue::UnsignedShort(LittleEndian::read_u16(buf)),
        SensorType::Int => SensorValue::Int(LittleEndian::read_i32(buf)),
        SensorType::UnsignedInt => SensorValue::UnsignedInt(LittleEndian::read_u32(buf)),
        SensorType::Float => SensorValue::Float(LittleEndian::read_f32(buf)),
        SensorType::LongLong => SensorValue::LongLong(LittleEndian::read_i64(buf)),
        SensorType::UnsignedLongLong => SensorValue::UnsignedLongLong(LittleEndian::read_u64(buf)),
        SensorType::Double => SensorValue::Double(LittleEndian::read_f64(buf)),
    };
    Ok(value)
}
