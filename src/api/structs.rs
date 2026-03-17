use serde::Deserialize;


pub enum Channel {
    IbmQuantumPlatform,
    IbmCloud,
    Local,
}



#[derive(Deserialize)]
pub struct Resource {
    pub crn: String,
}


#[derive(Deserialize)]
pub struct InstancesResponse {
    pub rows_count: u32,
    pub resources: Vec<Resource>,
}
