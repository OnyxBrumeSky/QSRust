use std::fmt::{Display};
use std::error::Error;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use reqwest::header::{ACCEPT, AUTHORIZATION};


pub enum Channel {
    IbmQuantumPlatform,
    IbmCloud,
    Local,
}


#[derive(Deserialize, Serialize)]
pub struct Device {
    pub name: String,
    pub qubits: u32,
    pub queue_length: u32,
    pub status: DeviceStatus,
    pub processor_type : Option<Processortype>
}

#[derive(Deserialize, Serialize)]
pub struct Processortype {
    pub family: String,
    pub revision: String,
}

#[derive(Deserialize, Serialize)]
pub struct DeviceStatus {
    pub name: String,
    pub reason: String,
}


#[derive(Deserialize)]
struct  IAM {
    access_token : String,
    refresh_token : String,
    expires_in : u64,
    expiration : u64,
}


#[derive(Deserialize)]
struct InstancesResponse {
    rows_count: u32,
    resources: Vec<Resource>,
}

#[derive(Deserialize)]
struct Resource {
    crn: String,
}

pub struct Service {
    channel : Channel,
    token: String,
    backends : Device,
    url : String,
    instance : String,
    region : String,
    http : reqwest::Client,
    iam : IAM,
}

pub struct  ServiceBuilder {
    token: Option<String>,
    channel: Option<Channel>,
    url: Option<String>,
    instance: Option<String>,
    region: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct BackendManager {
    pub devices: Vec<Device>,
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

impl Default for IAM {
    fn default() -> Self {
        IAM {
            access_token: String::new(),
            refresh_token: String::new(),
            expires_in: 0,
            expiration: 0,
        }
    }
}


impl ServiceBuilder {
    pub fn token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }

    pub fn channel(mut self, channel: Channel) -> Self {
        self.channel = Some(channel);
        self
    }

    pub fn url(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }

    pub fn region(mut self, region: String) -> Self {
        self.region = Some(region);
        self
    }

    pub async fn build(self) -> Result<Service,Box<dyn Error>> {

        let token = self.token.clone().unwrap_or_else(|| "no_token".to_string());
        let http = Client::builder().user_agent("quantum-rust-client/0.1 (reqwest)").build()?;
    
        let param = [("grant_type", "urn:ibm:params:oauth:grant-type:apikey"), ("apikey", &token)];

        let response = http
        .post("https://iam.cloud.ibm.com/identity/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&param)
        .send()
        .await?
        .json::<IAM>()
        .await?;


        let instance = http
        .get("https://resource-controller.cloud.ibm.com/v2/resource_instances")
        .bearer_auth(&response.access_token)
        .send()
        .await?
        .json::<InstancesResponse>()
        .await?;
        
        if instance.rows_count == 0 {
            return Err("No resource instance found for the provided API key.".into());
        }

        let ist = Service {
            token,
            channel: self.channel.unwrap_or(Channel::IbmQuantumPlatform),
            url: self.url.unwrap_or_else(|| "https://quantum-computing.ibm.com/api".to_string()),
            instance: instance.resources[0].crn.clone(),
            region: self.region.unwrap_or_else(|| "us-east".to_string()),
            backends: Device::default(),
            http,
            iam : response,
        };

        Ok(ist)
    }
}

#[derive(Deserialize)]
struct BackendsResponse {
    devices: Vec<Device>
}

impl Service {

    pub fn builder() -> ServiceBuilder {
        ServiceBuilder {
            token: None,
            channel: None,
            url: None,
            instance: None,
            region: None,
        }
    }



    pub async fn get_backends(&self) -> Result<BackendManager, reqwest::Error> {

        let response = self.http
            .get("https://quantum.cloud.ibm.com/api/v1/backends")
            .header(ACCEPT, "application/json")
            .header(AUTHORIZATION, format!("Bearer {}", &self.iam.access_token))
            .header("Service-CRN", format!("{}",&self.instance))
            .header("IBM-API-Version", "2026-02-15").send().await?.error_for_status()?;

        let backends: BackendsResponse = response.json().await?;

        Ok(BackendManager {
            devices : backends.devices,
         })
    }

    pub fn use_backend(&mut self, backend : Device) {
        self.backends = backend;
    }

}


impl Default for Service {
    fn default() -> Self {
        Service {
            channel: Channel::IbmQuantumPlatform,
            token: String::new(),
            backends: Device::default(),
            url: "https://quantum.cloud.ibm.com/api/v1".to_string(),
            instance: String::new(),
            region: "us-east".to_string(),
            http: Client::new(),
            iam: IAM::default(),
        }
    }
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



impl Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Backend: {}, Qubits: {}, Simulator: {}, Status: {}", self.name, self.qubits, self.processor_type.is_none(), self.status.name)
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