use crate::api::service::Service;
use crate::api::job_struct::JobRoot;
use crate::api::job_struct::ResultRoot;
use crate::api::job_struct::JobListRoot;


impl Service {
    // pub async fn submit_job(&self, ){

    // }

    pub async  fn get_job_list(&self) -> Result<JobListRoot, reqwest::Error>{
        let response = self.get("https://quantum.cloud.ibm.com/api/v1/jobs?fields=id").await?;
        let response: JobListRoot = response.json().await?;
        Ok(response)
    }

    pub async fn get_job_result(&self, job_id : &str) -> Result<ResultRoot, reqwest::Error> {
        let url = format!("https://quantum.cloud.ibm.com/api/v1/jobs/{}/results",job_id);
        let response = self.get(&url).await?;
        let response: ResultRoot = response.json().await?;
        Ok(response)

    }

    pub async fn get_specific_job(&self, job_id : &str) -> Result<JobRoot, reqwest::Error> {
        let url = format!("https://quantum.cloud.ibm.com/api/v1/jobs/{}",job_id);
        let response = self.get(&url).await?;
        let response: JobRoot = response.json().await?;
        Ok(response)
    }



}