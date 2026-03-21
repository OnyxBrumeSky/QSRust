use serde::Deserialize;

/// Canal d'accès à IBM Quantum.
///
/// Détermine le point d'entrée et le mode d'authentification utilisés par le [`Service`].
///
/// [`Service`]: crate::api::service::Service
pub enum Channel {
    /// IBM Quantum Platform — accès via `quantum.cloud.ibm.com`
    IbmQuantumPlatform,
    /// IBM Cloud — accès via une instance de service IBM Cloud
    IbmCloud,
    /// Exécution locale (simulateur sans appel réseau)
    Local,
}

/// Ressource IBM Cloud retournée par le Resource Controller.
///
/// Contient le CRN (Cloud Resource Name) utilisé comme identifiant
/// de l'instance IBM Quantum Runtime dans le header `Service-CRN`.
#[derive(Deserialize)]
pub struct Resource {
    /// Cloud Resource Name de l'instance IBM Quantum Runtime
    pub crn: String,
}

/// Réponse de `GET /v2/resource_instances` du Resource Controller IBM Cloud.
///
/// Utilisée lors de la construction du [`Service`] pour résoudre automatiquement
/// le CRN de l'instance IBM Quantum associée à la clé API.
///
/// [`Service`]: crate::api::service::Service
#[derive(Deserialize)]
pub struct InstancesResponse {
    /// Nombre d'instances trouvées — si `0`, aucune instance n'est associée à la clé API
    pub rows_count: u32,
    /// Liste des instances retournées
    pub resources: Vec<Resource>,
}