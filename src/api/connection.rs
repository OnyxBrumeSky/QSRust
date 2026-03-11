use reqwest::Client;
use serde::{Deserialize, Serialize};

pub enum Channel {
    IbmQuantumPlatform,
    IbmCloud,
    Local,
}


#[derive(Deserialize, Serialize)]
pub struct Backend {
    name: String,
    num_qubits: u32,
    simulator: bool,
    basis_gates: Vec<String>,
    coupling_map: Vec<(u32, u32)>,
    max_shots: u32,
    pending_jobs: Option<u32>,
    status: String,
}

pub struct Service {
    channel : Channel,
    token: String,
    backends : Backend,
    url : String,
    instance : String,
    region : String,
    http : reqwest::Client,

}

pub struct  ServiceBuilder {
    token: Option<String>,
    channel: Option<Channel>,
    url: Option<String>,
    instance: Option<String>,
    region: Option<String>,
}

pub struct BackendManager {
    backends: Vec<Backend>,
}

impl Default for Backend {
    fn default() -> Self {
        Backend {
            name: String::from("default_backend"),
            num_qubits: 0,
            simulator: false,
            basis_gates: Vec::new(),
            coupling_map: Vec::new(),
            max_shots: 0,
            pending_jobs: None,
            status: String::new(),
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

    pub fn instance(mut self, instance: String) -> Self {
        self.instance = Some(instance);
        self
    }

    pub fn region(mut self, region: String) -> Self {
        self.region = Some(region);
        self
    }

    pub fn build(self) -> Service {
        Service {
            token: self.token.unwrap_or_else(|| "no_token".to_string()),
            channel: self.channel.unwrap_or(Channel::IbmQuantumPlatform),
            url: self.url.unwrap_or_else(|| "https://quantum.cloud.ibm.com/api".to_string()),
            instance: self.instance.unwrap_or_default(),
            region: self.region.unwrap_or_else(|| "us-east".to_string()),
            backends: Backend::default(),
            http: Client::new(),
        }
    }
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

    pub async fn get_backends(&mut self) -> Result<BackendManager, reqwest::Error> {

        let url = format!("{}/backends", self.url);

        let response = self.http
            .get(url)
            .bearer_auth(&self.token)
            .send()
            .await?;

        let backends: Vec<Backend> = response.json().await?;

        Ok(BackendManager { backends })
    }

    pub fn use_backend(&mut self, backend : Backend) {
        self.backends = backend;
    }

}


impl Default for Service {
    fn default() -> Self {
        Service {
            channel: Channel::IbmQuantumPlatform,
            token: String::new(),
            backends: Backend::default(),
            url: "https://quantum.cloud.ibm.com/api".to_string(),
            instance: String::new(),
            region: "us-east".to_string(),
            http: Client::new(),
        }
    }
}



impl BackendManager {

    pub fn list(&self) -> &Vec<Backend> {
        &self.backends
    }

    pub fn simulators(&self) -> Vec<&Backend> {
        self.backends
            .iter()
            .filter(|b| b.simulator)
            .collect()
    }

    pub fn real(&self) -> Vec<&Backend> {
        self.backends
            .iter()
            .filter(|b| !b.simulator)
            .collect()
    }

    pub fn least_busy(&self) -> Option<&Backend> {
        self.backends
            .iter()
            .filter(|b| !b.simulator)
            .min_by_key(|b| b.pending_jobs.unwrap_or(u32::MAX))
    }

}