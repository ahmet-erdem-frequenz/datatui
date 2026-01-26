use crate::config::{Config, DataType, Endianness};
use crate::datapoint::{DataValue, Datapoint};
use anyhow::Result;
use tokio::time::Duration;
use tokio_modbus::prelude::*;
use log::{debug, info, warn, error};

pub struct Scanner {
    config: Config,
    datapoints: Vec<Datapoint>,
}

impl Scanner {
    pub fn new(config: Config) -> Self {
        let datapoints = config
            .datapoints
            .iter()
            .map(|dp| {
                if let Some(ref bitfields) = dp.bitfields {
                    let mut bitfield_map = std::collections::HashMap::new();
                    for bf in bitfields {
                        bitfield_map.insert(bf.bit, bf.name.clone());
                    }
                    Datapoint::with_bitfields(dp.name.clone(), dp.address, dp.description.clone(), bitfield_map)
                } else {
                    Datapoint::new(dp.name.clone(), dp.address, dp.description.clone())
                }
            })
            .collect();

        Self { config, datapoints }
    }

    pub async fn scan_once(&mut self) -> Result<()> {
        if self.config.server.protocol.to_lowercase() == "modbus" {
            self.scan_modbus().await?;
        } else {
            anyhow::bail!("Unsupported protocol: {}", self.config.server.protocol);
        }
        Ok(())
    }

    async fn scan_modbus(&mut self) -> Result<()> {
        // Handle IPv6 addresses by wrapping them in brackets
        let socket_addr = if self.config.server.host.contains(':') {
            // IPv6 address - needs brackets
            format!("[{}]:{}", self.config.server.host, self.config.server.port)
        } else {
            // IPv4 address or hostname
            format!("{}:{}", self.config.server.host, self.config.server.port)
        };
        
        info!("Connecting to Modbus server at {}", socket_addr);
        
        // Add timeout to connection attempt
        let connect_result = tokio::time::timeout(
            Duration::from_secs(5),
            tcp::connect_slave(socket_addr.parse()?, Slave(self.config.server.unit_id))
        ).await;
        
        let mut ctx = match connect_result {
            Ok(Ok(ctx)) => {
                debug!("Connected successfully");
                ctx
            },
            Ok(Err(e)) => {
                error!("Connection failed: {}", e);
                for i in 0..self.datapoints.len() {
                    self.datapoints[i].update_error(format!("Connection failed: {}", e));
                }
                anyhow::bail!("Failed to connect to Modbus server: {}", e);
            }
            Err(_) => {
                error!("Connection timeout");
                for i in 0..self.datapoints.len() {
                    self.datapoints[i].update_error("Connection timeout".to_string());
                }
                anyhow::bail!("Connection timeout");
            }
        };

        let endianness = self.config.server.endianness;

        for (i, dp_config) in self.config.datapoints.iter().enumerate() {
            debug!("Reading {} at address {} (length {})", 
                   dp_config.name, dp_config.address, dp_config.length);
            
            // Add timeout to read operations as well
            let read_result = tokio::time::timeout(
                Duration::from_secs(2),
                match dp_config.register_type {
                    crate::config::RegisterType::Holding => ctx.read_holding_registers(dp_config.address, dp_config.length),
                    crate::config::RegisterType::Input => ctx.read_input_registers(dp_config.address, dp_config.length),
                }
            ).await;
            
            match read_result {
                Ok(Ok(Ok(registers))) => {
                    debug!("Successfully read {} registers: {:?}", registers.len(), registers);
                    let value = match dp_config.data_type {
                        DataType::U16 => {
                            if !registers.is_empty() {
                                DataValue::U16(registers[0])
                            } else {
                                continue;
                            }
                        }
                        DataType::I16 => {
                            if !registers.is_empty() {
                                DataValue::I16(registers[0] as i16)
                            } else {
                                continue;
                            }
                        }
                        DataType::U32 => {
                            if registers.len() >= 2 {
                                let val = match endianness {
                                    Endianness::Big => ((registers[0] as u32) << 16) | (registers[1] as u32),
                                    Endianness::Little => ((registers[1] as u32) << 16) | (registers[0] as u32),
                                };
                                DataValue::U32(val)
                            } else {
                                continue;
                            }
                        }
                        DataType::I32 => {
                            if registers.len() >= 2 {
                                let val = match endianness {
                                    Endianness::Big => ((registers[0] as u32) << 16) | (registers[1] as u32),
                                    Endianness::Little => ((registers[1] as u32) << 16) | (registers[0] as u32),
                                };
                                DataValue::I32(val as i32)
                            } else {
                                continue;
                            }
                        }
                        DataType::F32 => {
                            if registers.len() >= 2 {
                                let bytes = match endianness {
                                    Endianness::Big => [
                                        (registers[0] >> 8) as u8,
                                        (registers[0] & 0xFF) as u8,
                                        (registers[1] >> 8) as u8,
                                        (registers[1] & 0xFF) as u8,
                                    ],
                                    Endianness::Little => [
                                        (registers[1] >> 8) as u8,
                                        (registers[1] & 0xFF) as u8,
                                        (registers[0] >> 8) as u8,
                                        (registers[0] & 0xFF) as u8,
                                    ],
                                };
                                DataValue::F32(f32::from_be_bytes(bytes))
                            } else {
                                continue;
                            }
                        }
                        DataType::Bitfield => {
                            if !registers.is_empty() {
                                DataValue::Bitfield(registers[0])
                            } else {
                                continue;
                            }
                        }
                    };
                    self.datapoints[i].update_value(value);
                }
                Ok(Ok(Err(e))) => {
                    warn!("Modbus exception for {}: {}", dp_config.name, e);
                    self.datapoints[i].update_error(format!("Modbus exception: {}", e));
                }
                Ok(Err(e)) => {
                    warn!("Read error for {}: {}", dp_config.name, e);
                    self.datapoints[i].update_error(format!("Read error: {}", e));
                }
                Err(_) => {
                    warn!("Read timeout for {}", dp_config.name);
                    self.datapoints[i].update_error("Read timeout".to_string());
                }
            }
        }

        Ok(())
    }

    pub fn get_datapoints(&self) -> &[Datapoint] {
        &self.datapoints
    }
}
