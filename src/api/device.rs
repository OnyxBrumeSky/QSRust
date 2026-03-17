use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Deserialize, Serialize)]
pub struct Device {
    pub name: String,
    pub qubits: u32,
    pub queue_length: u32,
    pub status: DeviceStatus,
    pub processor_type : Option<Processortype>
}

#[derive(Deserialize, Serialize)]
pub struct DeviceStatus {
    pub name: String,
    pub reason: String,
}

#[derive(Deserialize, Serialize)]
pub struct Processortype {
    pub family: String,
    pub revision: String,
}


impl Default for Device {
    fn default() -> Self {
        Device {
            name: String::from("default_backend"),
            qubits: 0,
            queue_length: 0,
            status: DeviceStatus {
                name: String::new(),
                reason: String::new(),
            },
            processor_type: None,
        }
    }
}


impl Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Backend: {}, Qubits: {}, Simulator: {}, Status: {}", self.name, self.qubits, self.processor_type.is_none(), self.status.name)
    }
}