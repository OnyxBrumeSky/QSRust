use serde::Deserialize;

/// Token d'accès IBM Cloud IAM retourné après authentification.
///
/// Obtenu via `POST https://iam.cloud.ibm.com/identity/token` avec une API key.
/// Le champ `access_token` est utilisé dans chaque requête vers l'API IBM Quantum
/// via le header `Authorization: Bearer <access_token>`.
#[derive(Deserialize)]
pub struct IAM {
    /// Token Bearer à inclure dans les headers des requêtes API
    pub access_token: String,
    /// Token permettant de renouveler l'accès sans se réauthentifier
    pub refresh_token: String,
    /// Durée de validité du token en secondes
    pub expires_in: u64,
    /// Timestamp Unix d'expiration du token
    pub expiration: u64,
}

impl Default for IAM {
    /// Retourne un [`IAM`] vide, utilisé avant la première authentification.
    fn default() -> Self {
        IAM {
            access_token:  String::new(),
            refresh_token: String::new(),
            expires_in:    0,
            expiration:    0,
        }
    }
}