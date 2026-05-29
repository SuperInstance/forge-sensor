//! Forge Sensor — Decompose sensor readings into tiles for Plato agents
//!
//! Sensor data (temperature, pressure, GPS, sonar, etc.) gets decomposed
//! into timestamped tiles that Plato agents can read, filter, and transform.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A sensor reading tile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorTile {
    pub id: Uuid,
    pub sensor_type: SensorType,
    pub value: f64,
    pub unit: String,
    pub timestamp_ms: u64,
    pub location: Option<(f64, f64)>, // lat, lon
    pub meta: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SensorType {
    Temperature,
    Pressure,
    Humidity,
    Gps,
    Sonar,
    Radar,
    WindSpeed,
    WindDirection,
    EngineRpm,
    FuelLevel,
    Depth,
    Custom(String),
}

/// Decompose raw sensor readings into tiles
pub struct SensorDecomposer;

impl SensorDecomposer {
    pub fn new() -> Self { Self }

    /// Parse a line of sensor data: "TYPE:value:unit@timestamp"
    pub fn parse_line(&self, line: &str) -> Result<SensorTile, String> {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 3 {
            return Err(format!("Invalid sensor line: {}", line));
        }
        let sensor_type = match parts[0].to_lowercase().as_str() {
            "temp" | "temperature" => SensorType::Temperature,
            "pressure" => SensorType::Pressure,
            "humidity" => SensorType::Humidity,
            "gps" => SensorType::Gps,
            "sonar" => SensorType::Sonar,
            "radar" => SensorType::Radar,
            "wind_speed" => SensorType::WindSpeed,
            "wind_dir" => SensorType::WindDirection,
            "rpm" => SensorType::EngineRpm,
            "fuel" => SensorType::FuelLevel,
            "depth" => SensorType::Depth,
            other => SensorType::Custom(other.to_string()),
        };
        let value: f64 = parts[1].parse().map_err(|_| format!("Invalid value: {}", parts[1]))?;
        let rest = parts[2..].join(":");
        let (unit, timestamp_ms) = if let Some(at_pos) = rest.find('@') {
            (rest[..at_pos].to_string(), rest[at_pos+1..].parse().unwrap_or(0))
        } else {
            (rest, 0)
        };
        Ok(SensorTile {
            id: Uuid::new_v4(),
            sensor_type,
            value,
            unit,
            timestamp_ms,
            location: None,
            meta: HashMap::new(),
        })
    }

    /// Parse multiple lines of sensor data
    pub fn parse_lines(&self, input: &str) -> Vec<SensorTile> {
        input.lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| self.parse_line(l).ok())
            .collect()
    }

    /// Filter tiles by sensor type
    pub fn filter_by_type(&self, tiles: &[SensorTile], sensor_type: &SensorType) -> Vec<SensorTile> {
        tiles.iter().filter(|t| &t.sensor_type == sensor_type).cloned().collect()
    }

    /// Filter tiles by time range
    pub fn filter_by_time(&self, tiles: &[SensorTile], start_ms: u64, end_ms: u64) -> Vec<SensorTile> {
        tiles.iter().filter(|t| t.timestamp_ms >= start_ms && t.timestamp_ms <= end_ms).cloned().collect()
    }

    /// Get the latest reading for each sensor type
    pub fn latest_by_type<'a>(&self, tiles: &'a [SensorTile]) -> HashMap<String, &'a SensorTile> {
        let mut map: HashMap<String, &'a SensorTile> = HashMap::new();
        for tile in tiles {
            let key = format!("{:?}", tile.sensor_type);
            match map.get(&key) {
                Some(existing) if existing.timestamp_ms > tile.timestamp_ms => {}
                _ => { map.insert(key, tile); }
            }
        }
        map
    }

    /// Compute statistics for a set of tiles
    pub fn stats(&self, tiles: &[SensorTile]) -> SensorStats {
        if tiles.is_empty() {
            return SensorStats { count: 0, min: 0.0, max: 0.0, mean: 0.0, std_dev: 0.0 };
        }
        let values: Vec<f64> = tiles.iter().map(|t| t.value).collect();
        let count = values.len();
        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let mean = values.iter().sum::<f64>() / count as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / count as f64;
        SensorStats { count, min, max, mean, std_dev: variance.sqrt() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorStats {
    pub count: usize,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub std_dev: f64,
}

impl Default for SensorDecomposer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_temperature() {
        let d = SensorDecomposer::new();
        let tile = d.parse_line("temperature:22.5:celsius@1700000000").unwrap();
        assert_eq!(tile.sensor_type, SensorType::Temperature);
        assert!((tile.value - 22.5).abs() < 0.001);
        assert_eq!(tile.unit, "celsius");
        assert_eq!(tile.timestamp_ms, 1700000000);
    }

    #[test]
    fn test_parse_sonar() {
        let d = SensorDecomposer::new();
        let tile = d.parse_line("sonar:45.2:fathoms@1700001000").unwrap();
        assert_eq!(tile.sensor_type, SensorType::Sonar);
        assert!((tile.value - 45.2).abs() < 0.001);
    }

    #[test]
    fn test_parse_rpm() {
        let d = SensorDecomposer::new();
        let tile = d.parse_line("rpm:1650:revolutions_per_minute@1700002000").unwrap();
        assert_eq!(tile.sensor_type, SensorType::EngineRpm);
    }

    #[test]
    fn test_parse_custom_sensor() {
        let d = SensorDecomposer::new();
        let tile = d.parse_line("salinity:35.2:psu@1700003000").unwrap();
        assert_eq!(tile.sensor_type, SensorType::Custom("salinity".to_string()));
    }

    #[test]
    fn test_parse_multiple_lines() {
        let input = "temperature:22.5:c@100\nsonar:45.2:fm@200\nrpm:1650:rpm@300\n";
        let tiles = SensorDecomposer::new().parse_lines(input);
        assert_eq!(tiles.len(), 3);
    }

    #[test]
    fn test_filter_by_type() {
        let d = SensorDecomposer::new();
        let tiles = d.parse_lines("temp:22:c@100\nsonar:45:fm@200\ntemp:23:c@300");
        let filtered = d.filter_by_type(&tiles, &SensorType::Temperature);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_by_time() {
        let d = SensorDecomposer::new();
        let tiles = d.parse_lines("temp:22:c@100\nsonar:45:fm@200\ntemp:23:c@300");
        let filtered = d.filter_by_time(&tiles, 150, 250);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].sensor_type, SensorType::Sonar);
    }

    #[test]
    fn test_latest_by_type() {
        let d = SensorDecomposer::new();
        let tiles = d.parse_lines("temp:22:c@100\ntemp:24:c@200\ntemp:23:c@150");
        let latest = d.latest_by_type(&tiles);
        assert_eq!(latest.len(), 1);
        assert!((latest.values().next().unwrap().value - 24.0).abs() < 0.001);
    }

    #[test]
    fn test_stats() {
        let d = SensorDecomposer::new();
        let tiles = d.parse_lines("temp:20:c@100\ntemp:22:c@200\ntemp:24:c@300");
        let stats = d.stats(&tiles);
        assert_eq!(stats.count, 3);
        assert!((stats.min - 20.0).abs() < 0.001);
        assert!((stats.max - 24.0).abs() < 0.001);
        assert!((stats.mean - 22.0).abs() < 0.001);
    }

    #[test]
    fn test_stats_empty() {
        let stats = SensorDecomposer::new().stats(&[]);
        assert_eq!(stats.count, 0);
    }

    #[test]
    fn test_invalid_line() {
        let d = SensorDecomposer::new();
        assert!(d.parse_line("invalid").is_err());
        assert!(d.parse_line("").is_err());
    }

    #[test]
    fn test_tile_serialization() {
        let d = SensorDecomposer::new();
        let tile = d.parse_line("depth:100:fm@1000").unwrap();
        let json = serde_json::to_string(&tile).unwrap();
        let back: SensorTile = serde_json::from_str(&json).unwrap();
        assert_eq!(tile.id, back.id);
        assert!((tile.value - back.value).abs() < 0.001);
    }
}
