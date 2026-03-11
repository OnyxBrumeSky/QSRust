use std::fmt::{format, Display};
use std::error::Error;
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
    backends : Backend,
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
        let http = Client::new();
    
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
        .get(" https://resource-controller.cloud.ibm.com/v2/resource_instances")
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
            backends: Backend::default(),
            http,
            iam : response,
        };

        Ok(ist)
    }
}

#[derive(Deserialize)]
struct BackendsResponse {
    backends: Vec<Backend>
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

        let url = format!(
            "{}/Backends",
            self.url
        );
    
        println!("Fetching backends from URL: {}", url);

        let response = self.http
            .get(url)
            //.bearer_auth(&self.iam.access_token)
            .header("accept", "application/json")
            .header("Authorization", format!("Bearer {}", &self.iam.access_token))
            .header("Service-CRN:", &self.instance)
            .header("IBM-API-Version:", "2026-02-15")
            .send()
            .await?;
    
        println!("Response status: {} with url : {}", response.status(), response.url());
        
        let backends: BackendsResponse = response.json().await?;

        Ok(BackendManager {
            backends: backends.backends
         })
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
            url: "https://quantum.cloud.ibm.com/api/v1".to_string(),
            instance: String::new(),
            region: "us-east".to_string(),
            http: Client::new(),
            iam: IAM::default(),
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



impl Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Backend: {}, Qubits: {}, Simulator: {}, Status: {}", self.name, self.num_qubits, self.simulator, self.status)
    }
}


impl Display for BackendManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for backend in &self.backends {
            writeln!(f, "{}", backend)?;
        }
        Ok(())
    }
}