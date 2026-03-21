use crate::api::iam::IAM;
use crate::api::device::Device;
use crate::api::structs::Channel;
use crate::api::structs::InstancesResponse;
use crate::api::backend_manager::{BackendManager, BackendsResponse};
use std::error::Error;
use reqwest::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION};
use reqwest::Response;

/// Client principal pour interagir avec l'API IBM Quantum.
///
/// Encapsule l'authentification IAM, l'instance CRN, le backend actif
/// et le client HTTP. Construit via [`ServiceBuilder`].
pub struct Service {
    /// Canal d'accès IBM (plateforme quantique, cloud...)
    _channel: Channel,
    /// Clé API IBM Cloud utilisée pour l'authentification
    _token: String,
    /// Backend actif sélectionné via [`use_backend`](Service::use_backend)
    backends: Device,
    /// URL de base de l'API IBM Quantum
    _url: String,
    /// CRN de l'instance IBM Quantum Runtime (utilisé dans `Service-CRN`)
    instance: String,
    /// Région IBM Cloud (ex: `"us-east"`)
    _region: String,
    /// Client HTTP reqwest partagé entre toutes les requêtes
    http: Client,
    /// Token d'accès IAM courant
    iam: IAM,
}

/// Builder pour construire un [`Service`] authentifié.
///
/// Récupère automatiquement un token IAM depuis la clé API fournie
/// et résout le CRN de l'instance IBM Quantum Runtime.
///
/// # Exemple
/// ```rust
/// let service = Service::builder()
///     .token("votre_api_key".to_string())
///     .region("eu-de".to_string())
///     .build()
///     .await?;
/// ```
pub struct ServiceBuilder {
    /// Clé API IBM Cloud
    token: Option<String>,
    /// Canal d'accès IBM
    channel: Option<Channel>,
    /// URL de base personnalisée (optionnelle)
    url: Option<String>,
    /// Instance CRN personnalisée (optionnelle — détectée automatiquement sinon)
    _instance: Option<String>,
    /// Région IBM Cloud (défaut : `"us-east"`)
    region: Option<String>,
}

impl ServiceBuilder {
    /// Définit la clé API IBM Cloud à utiliser pour l'authentification.
    pub fn token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }

    /// Définit le canal d'accès IBM (plateforme quantique, cloud...).
    pub fn channel(mut self, channel: Channel) -> Self {
        self.channel = Some(channel);
        self
    }

    /// Définit une URL de base personnalisée pour l'API.
    pub fn url(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }

    /// Définit la région IBM Cloud (ex: `"eu-de"`, `"us-east"`).
    pub fn region(mut self, region: String) -> Self {
        self.region = Some(region);
        self
    }

    /// Construit le [`Service`] en s'authentifiant auprès de l'IAM IBM Cloud.
    ///
    /// Effectue deux appels réseau :
    /// 1. `POST https://iam.cloud.ibm.com/identity/token` — obtient un token Bearer
    /// 2. `GET https://resource-controller.cloud.ibm.com/v2/resource_instances` — résout le CRN
    ///
    /// Retourne une erreur si aucune instance IBM Quantum n'est trouvée pour la clé API.
    pub async fn build(self) -> Result<Service, Box<dyn Error>> {
        let token = self.token.clone().unwrap_or_else(|| "no_token".to_string());
        let http = Client::builder()
            .user_agent("quantum-rust-client/0.1 (reqwest)")
            .build()?;

        let param = [
            ("grant_type", "urn:ibm:params:oauth:grant-type:apikey"),
            ("apikey", &token),
        ];

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
            _token : token,
            _channel:  self.channel.unwrap_or(Channel::IbmQuantumPlatform),
            _url:      self.url.unwrap_or_else(|| "https://quantum-computing.ibm.com/api".to_string()),
            instance: instance.resources[0].crn.clone(),
            _region:   self.region.unwrap_or_else(|| "us-east".to_string()),
            backends: Device::default(),
            http,
            iam:      response,
        };

        Ok(ist)
    }
}

impl Service {
    /// Crée un [`ServiceBuilder`] pour construire un service authentifié.
    pub fn builder() -> ServiceBuilder {
        ServiceBuilder {
            token:    None,
            channel:  None,
            url:      None,
            _instance: None,
            region:   None,
        }
    }

    /// Effectue une requête `GET` authentifiée vers l'API IBM Quantum.
    ///
    /// Ajoute automatiquement les headers `Authorization`, `Service-CRN`
    /// et `IBM-API-Version` requis par l'API.
    pub async fn get(&self, url: &str) -> Result<Response, reqwest::Error> {
        self.http
            .get(url)
            .header(ACCEPT, "application/json")
            .header(AUTHORIZATION, format!("Bearer {}", &self.iam.access_token))
            .header("Service-CRN", format!("{}", &self.instance))
            .header("IBM-API-Version", "2026-02-15")
            .send()
            .await?
            .error_for_status()
    }

    /// Effectue une requête `POST` authentifiée avec un body JSON.
    ///
    /// Ajoute automatiquement les headers `Authorization`, `Service-CRN`,
    /// `Content-Type` et `IBM-API-Version`.
    pub async fn post(&self, url: &str, body: &serde_json::Value) -> Result<Response, reqwest::Error> {
        self.http
            .post(url)
            .header(reqwest::header::ACCEPT, "application/json")
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", &self.iam.access_token))
            .header("Service-CRN", &self.instance)
            .header("IBM-API-Version", "2026-02-15")
            .json(body)
            .send()
            .await?
            .error_for_status()
    }

    /// Récupère la liste des backends IBM Quantum disponibles.
    ///
    /// Appelle `GET /v1/backends` et retourne un [`BackendManager`]
    /// permettant de filtrer et sélectionner les backends.
    pub async fn get_backends(&self) -> Result<BackendManager, reqwest::Error> {
        let response = self.get("https://quantum.cloud.ibm.com/api/v1/backends").await?;
        let backends: BackendsResponse = response.json().await?;
        Ok(BackendManager { devices: backends.devices })
    }

    /// Définit le backend actif utilisé pour les jobs et la transpilation.
    pub fn use_backend(&mut self, backend: Device) {
        self.backends = backend;
    }

    /// Retourne le nom du backend actif.
    pub fn backend_name(&self) -> &str {
        &self.backends.name
    }
}

impl Default for Service {
    /// Retourne un [`Service`] vide non authentifié, avec les valeurs par défaut.
    fn default() -> Self {
        Service {
            _channel:  Channel::IbmQuantumPlatform,
            _token:    String::new(),
            backends: Device::default(),
            _url:      "https://quantum.cloud.ibm.com/api/v1".to_string(),
            instance: String::new(),
            _region:   "us-east".to_string(),
            http:     Client::new(),
            iam:      IAM::default(),
        }
    }
}