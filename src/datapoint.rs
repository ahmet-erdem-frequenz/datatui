use chrono::{DateTime, Local};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Datapoint {
    pub name: String,
    pub address: u16,
    pub value: Option<DataValue>,
    pub last_updated: Option<DateTime<Local>>,
    pub error: Option<String>,
    #[allow(dead_code)]
    pub description: Option<String>,
    pub bitfield_names: Option<HashMap<u8, String>>,
}

#[derive(Debug, Clone)]
pub enum DataValue {
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    F32(f32),
    Bitfield(u16),
}

impl DataValue {
    pub fn type_name(&self) -> &str {
        match self {
            DataValue::U16(_) => "u16",
            DataValue::I16(_) => "i16",
            DataValue::U32(_) => "u32",
            DataValue::I32(_) => "i32",
            DataValue::F32(_) => "f32",
            DataValue::Bitfield(_) => "bits",
        }
    }
}

impl fmt::Display for DataValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataValue::U16(v) => write!(f, "{}", v),
            DataValue::I16(v) => write!(f, "{}", v),
            DataValue::U32(v) => write!(f, "{}", v),
            DataValue::I32(v) => write!(f, "{}", v),
            DataValue::F32(v) => write!(f, "{:.2}", v),
            DataValue::Bitfield(v) => write!(f, "0x{:04X}", v),
        }
    }
}

impl Datapoint {
    pub fn new(name: String, address: u16, description: Option<String>) -> Self {
        Self {
            name,
            address,
            value: None,
            last_updated: None,
            error: None,
            description,
            bitfield_names: None,
        }
    }

    pub fn with_bitfields(
        name: String,
        address: u16,
        description: Option<String>,
        bitfield_names: HashMap<u8, String>,
    ) -> Self {
        Self {
            name,
            address,
            value: None,
            last_updated: None,
            error: None,
            description,
            bitfield_names: Some(bitfield_names),
        }
    }

    pub fn update_value(&mut self, value: DataValue) {
        self.value = Some(value);
        self.last_updated = Some(Local::now());
        self.error = None;
    }

    pub fn update_error(&mut self, error: String) {
        self.error = Some(error);
        self.last_updated = Some(Local::now());
    }

    pub fn get_bitfield_status(&self) -> Option<Vec<(u8, String, bool)>> {
        if let (Some(bitfield_names), Some(DataValue::Bitfield(value))) =
            (&self.bitfield_names, &self.value)
        {
            let mut bits = Vec::new();
            for (bit, name) in bitfield_names {
                let is_set = (*value & (1 << bit)) != 0;
                bits.push((*bit, name.clone(), is_set));
            }
            bits.sort_by_key(|(bit, _, _)| *bit);
            Some(bits)
        } else {
            None
        }
    }
}
