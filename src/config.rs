use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub datapoints: Vec<DatapointConfig>,
    #[serde(default = "default_scan_interval")]
    pub scan_interval_ms: u64,
}

fn default_scan_interval() -> u64 {
    1000
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub protocol: String,
    pub host: String,
    pub port: u16,
    #[serde(default = "default_unit_id")]
    pub unit_id: u8,
    #[serde(default = "default_endianness")]
    pub endianness: Endianness,
}

fn default_unit_id() -> u8 {
    1
}

fn default_endianness() -> Endianness {
    Endianness::Big
}

fn default_data_type() -> DataType {
    DataType::U16
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Endianness {
    Big,
    Little,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatapointConfig {
    pub name: String,
    pub address: u16,
    pub length: u16,
    #[serde(default = "default_data_type")]
    pub data_type: DataType,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub bitfields: Option<Vec<BitfieldConfig>>,
    #[serde(default = "default_register_type")]
    pub register_type: RegisterType,
}

fn default_register_type() -> RegisterType {
    RegisterType::Holding
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RegisterType {
    Holding,  // Function code 3
    Input,    // Function code 4
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BitfieldConfig {
    pub bit: u8,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    #[default]
    U16,
    I16,
    U32,
    I32,
    F32,
    Bitfield,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .context("Failed to read configuration file")?;
        let config: Config = serde_yaml::from_str(&content)
            .context("Failed to parse configuration file")?;
        Ok(config)
    }
}
