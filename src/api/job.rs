use tokio::time::{sleep, Duration};
use crate::api::service::Service;
use crate::api::job_struct::{JobRoot, ResultRoot, JobListRoot, SubmitJobResponse};
use crate::api::job_builder::JobRequest;

/// Statut d'un job IBM Quantum.
///
/// Construit depuis la chaîne retournée par `state.status` dans la réponse API.
#[derive(Debug, PartialEq)]
pub enum JobStatus {
    /// Job en attente dans la file d'exécution
    Queued,
    /// Job en cours d'exécution sur le backend
    Running,
    /// Job terminé avec succès
    Completed,
    /// Job terminé en erreur
    Failed,
    /// Job annulé
    Cancelled,
    /// Statut non reconnu — contient la valeur brute reçue
    Unknown(String),
}

impl JobStatus {
    /// Construit un [`JobStatus`] depuis une chaîne insensible à la casse.
    ///
    /// Accepte `"cancelled"` et `"canceled"` (variante américaine).
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "queued"               => JobStatus::Queued,
            "running"              => JobStatus::Running,
            "completed"            => JobStatus::Completed,
            "failed"               => JobStatus::Failed,
            "cancelled" | "canceled" => JobStatus::Cancelled,
            other                  => JobStatus::Unknown(other.to_string()),
        }
    }

    /// Retourne `true` si le job est dans un état terminal (succès, échec ou annulation).
    pub fn is_terminal(&self) -> bool {
        matches!(self, JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled)
    }

    /// Retourne `true` uniquement si le job s'est terminé avec succès.
    pub fn is_success(&self) -> bool {
        matches!(self, JobStatus::Completed)
    }
}

impl Service {

    /// Soumet un job construit via [`SamplerJobBuilder`] ou [`EstimatorJobBuilder`].
    ///
    /// # Exemple
    /// ```rust
    /// let job = JobRequest::Sampler(
    ///     SamplerJobBuilder::new(service.backend_name())
    ///         .add_pub(SamplerPub::new(&qasm).shots(1024))
    /// );
    /// let response = service.submit_job(job).await?;
    /// println!("Job ID : {}", response.id);
    /// ```
    pub async fn submit_job(
        &self,
        job: JobRequest,
    ) -> Result<SubmitJobResponse, Box<dyn std::error::Error>> {
        let body = job.build().map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
        let response = self.post(
            "https://quantum.cloud.ibm.com/api/v1/jobs",
            &body,
        ).await?;
        let result: SubmitJobResponse = response.json().await?;
        Ok(result)
    }

    /// Retourne le [`JobStatus`] courant d'un job depuis `state.status`.
    pub async fn get_job_status(
        &self,
        job_id: &str,
    ) -> Result<JobStatus, Box<dyn std::error::Error>> {
        let job = self.get_specific_job(job_id).await?;
        Ok(JobStatus::from_str(&job.state.status))
    }

    /// Attend qu'un job atteigne un état terminal en pollant toutes les `interval_secs` secondes.
    ///
    /// Retourne `Ok(())` si le job se termine avec succès ([`JobStatus::Completed`]).
    /// Retourne une erreur si le job échoue ou est annulé.
    pub async fn wait_for_job(
        &self,
        job_id: &str,
        interval_secs: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let status = self.get_job_status(job_id).await?;

            if status.is_terminal() {
                return if status.is_success() {
                    Ok(())
                } else {
                    Err(format!("Job {} terminé avec statut {:?}", job_id, status).into())
                };
            }

            sleep(Duration::from_secs(interval_secs)).await;
        }
    }

    /// Soumet un job, attend sa complétion et retourne les résultats.
    ///
    /// Combine [`submit_job`], [`wait_for_job`] et [`get_job_result`] en un seul appel.
    ///
    /// [`submit_job`]: Service::submit_job
    /// [`wait_for_job`]: Service::wait_for_job
    /// [`get_job_result`]: Service::get_job_result
    ///
    /// # Exemple
    /// ```rust
    /// let results = service.run_and_collect(job, 5).await?;
    /// let counts = results.to_counts()?;
    /// println!("{:?}", counts); // {"00": 512, "11": 512}
    /// ```
    pub async fn run_and_collect(
        &self,
        job: JobRequest,
        poll_interval_secs: u64,
    ) -> Result<ResultRoot, Box<dyn std::error::Error>> {
        let submitted = self.submit_job(job).await?;
        self.wait_for_job(&submitted.id, poll_interval_secs).await?;
        let results = self.get_job_result(&submitted.id).await?;
        Ok(results)
    }

    /// Retourne la liste paginée des jobs de l'utilisateur.
    ///
    /// Appelle `GET /v1/jobs` avec les champs `id`, `status`, `state`, `backend` et `created`.
    pub async fn get_job_list(&self) -> Result<JobListRoot, reqwest::Error> {
        let response = self.get("https://quantum.cloud.ibm.com/api/v1/jobs?fields=id,status,state,backend,created").await?;
        let response: JobListRoot = response.json().await?;
        Ok(response)
    }

    /// Récupère les résultats d'un job terminé depuis `GET /v1/jobs/{id}/results`.
    ///
    /// ⚠️ Appelle [`wait_for_job`] avant si le job n'est pas encore [`JobStatus::Completed`] —
    /// l'API retourne une erreur 400 si le job est encore en cours.
    ///
    /// [`wait_for_job`]: Service::wait_for_job
    pub async fn get_job_result(&self, job_id: &str) -> Result<ResultRoot, Box<dyn std::error::Error>> {
        let url = format!("https://quantum.cloud.ibm.com/api/v1/jobs/{}/results", job_id);
        let response = self.get(&url).await?;
        let text = response.text().await?;
        let result: ResultRoot = serde_json::from_str(&text)?;
        Ok(result)
    }

    /// Récupère les détails complets d'un job depuis `GET /v1/jobs/{id}`.
    ///
    /// Retourne le [`JobRoot`] complet incluant statut, paramètres et métadonnées.
    pub async fn get_specific_job(&self, job_id: &str) -> Result<JobRoot, reqwest::Error> {
        let url = format!("https://quantum.cloud.ibm.com/api/v1/jobs/{}", job_id);
        let response = self.get(&url).await?;
        let response: JobRoot = response.json().await?;
        Ok(response)
    }
}