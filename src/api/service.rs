use crate::api::iam::IAM;
use crate::api::device::Device;
use crate::api::structs::Channel;
use crate::api::structs::InstancesResponse;
use crate::api::backend_manager::{BackendManager, BackendsResponse};
use std::error::Error;
use reqwest::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION};
use reqwest::Response;

pub struct Service {
    channel : Channel,
    token: String,
    backends : Device,
    url : String,
    instance : String,
    region : String,
    http : Client,
    iam : IAM,
}

pub struct  ServiceBuilder {
    token: Option<String>,
    channel: Option<Channel>,
    url: Option<String>,
    instance: Option<String>,
    region: Option<String>,
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


    pub async fn get(&self, url : &str) -> Result<Response, reqwest::Error>{
        self.http
        .get(url)
        .header(ACCEPT, "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", &self.iam.access_token))
        .header("Service-CRN", format!("{}",&self.instance))
        .header("IBM-API-Version", "2026-02-15")
        .send().await?.error_for_status()
    }


    pub async fn get_backends(&self) -> Result<BackendManager, reqwest::Error> {

        let response = self.get("https://quantum.cloud.ibm.com/api/v1/backends").await?;

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
