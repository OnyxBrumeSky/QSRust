use std::fmt::Display;
use serde::{Deserialize, Serialize};
use crate::api::device::Device;



#[derive(Deserialize, Serialize)]
pub struct BackendManager {
    pub devices: Vec<Device>,
}


#[derive(Deserialize)]
pub struct BackendsResponse {
    pub devices: Vec<Device>
}
impl BackendManager {

    pub fn list(&self) -> &Vec<Device> {
        &self.devices
    }

    pub fn simulators(&self) -> Vec<&Device> {
        self.devices
            .iter()
            .filter(|b| b.processor_type.is_none())
            .collect()
    }

    pub fn real(&self) -> Vec<&Device> {
        self.devices
            .iter()
            .filter(|b| b.processor_type.is_some())
            .collect()
    }

    pub fn least_busy(&self) -> Option<&Device> {
        self.devices
        .iter()
        .filter(|d| d.status.name == "online")
        .min_by_key(|d| d.queue_length)
        
    }

}


impl Display for BackendManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for backend in &self.devices {
            writeln!(f, "{}", backend)?;
        }
        Ok(())
    }
}