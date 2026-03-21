use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Détails complets d'un job retourné par `GET /v1/jobs/{id}`.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobRoot {
    /// Nom du backend utilisé (ex: `"ibm_fez"`)
    #[serde(default)] pub backend: String,
    /// Date de création du job au format ISO8601
    #[serde(default)] pub created: String,
    /// Temps d'exécution estimé en secondes
    #[serde(rename = "estimated_running_time_seconds", default)] pub estimated_running_time_seconds: f64,
    /// Identifiant unique du job
    #[serde(default)] pub id: String,
    /// Coût du job en unités IBM
    #[serde(default)] pub cost: i64,
    /// Paramètres soumis avec le job
    #[serde(default)] pub params: Params,
    /// Programme IBM Quantum utilisé (ex: `"sampler"`)
    #[serde(default)] pub program: Program,
    /// État détaillé du job
    #[serde(default)] pub state: State,
    /// Statut global du job (ex: `"Completed"`)
    #[serde(default)] pub status: String,
    /// Identifiant de l'utilisateur ayant soumis le job
    #[serde(rename = "user_id", default)] pub user_id: String,
}

/// Paramètres soumis lors de la création du job.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Params {
    /// Options d'exécution (dynamical decoupling, twirling...)
    #[serde(default)] pub options: serde_json::Value,
    /// Liste des PUBs soumis — circuits + shots, format flexible
    #[serde(default)] pub pubs: Vec<serde_json::Value>,
    /// Indique si le job est compatible avec le SDK Qiskit
    #[serde(rename = "support_qiskit", default)] pub support_qiskit: bool,
    /// Version du schéma des paramètres
    #[serde(default)] pub version: i64,
}

/// Programme IBM Quantum associé au job.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Program {
    /// Identifiant du programme (ex: `"sampler"`, `"estimator"`)
    #[serde(default)] pub id: String,
}

/// État détaillé d'un job.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    /// Statut fin-grain (ex: `"Queued"`, `"Running"`, `"Completed"`, `"Failed"`)
    #[serde(default)] pub status: String,
}

/// Résultat complet retourné par `GET /v1/jobs/{id}/results`.
///
/// Contient un résultat par PUB soumis ainsi que les métadonnées globales d'exécution.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResultRoot {
    /// Résultats par circuit soumis (un par PUB)
    #[serde(default)] pub results: Vec<PubResult>,
    /// Métadonnées globales d'exécution (spans, version...)
    #[serde(default)] pub metadata: serde_json::Value,
}

/// Résultat d'un PUB (circuit) individuel.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PubResult {
    /// Données de mesure du circuit
    #[serde(default)] pub data: PubData,
    /// Métadonnées associées à ce circuit
    #[serde(default)] pub metadata: serde_json::Value,
}

/// Données de mesure d'un circuit.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PubData {
    /// Registre classique `c` — contient les mesures de chaque shot
    #[serde(default)] pub c: ClassicalRegister,
}

/// Registre classique contenant les mesures d'un circuit.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassicalRegister {
    /// Mesures individuelles en hexadécimal (ex: `["0x0", "0x3", "0x1"]`)
    #[serde(default)] pub samples: Vec<String>,
    /// Nombre de bits classiques du registre
    #[serde(default)] pub num_bits: u32,
}

impl ResultRoot {
    /// Convertit les samples hex du premier PUB en histogramme de counts.
    ///
    /// Chaque sample hexadécimal (ex: `"0x3"`) est converti en bitstring
    /// (ex: `"11"`) et les occurrences sont comptées.
    ///
    /// Le nombre de bits est lu depuis `num_bits` si disponible,
    /// sinon déduit depuis la valeur maximale des samples.
    ///
    /// # Erreurs
    /// Retourne une erreur si `results` ou `samples` est vide,
    /// ou si un sample n'est pas un hexadécimal valide.
    ///
    /// # Exemple
    /// ```rust
    /// // ["0x3", "0x0", "0x3"] → {"11": 2, "00": 1}
    /// let counts = result.to_counts()?;
    /// ```
    pub fn to_counts(&self) -> Result<HashMap<String, u32>, String> {
        if self.results.is_empty() {
            return Err("results est vide".to_string());
        }

        let register = &self.results[0].data.c;

        if register.samples.is_empty() {
            return Err("samples est vide — aucune mesure disponible".to_string());
        }

        let num_bits = if register.num_bits > 0 {
            register.num_bits as usize
        } else {
            let max = register.samples.iter()
                .filter_map(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
                .max()
                .unwrap_or(0);
            if max == 0 { 1 } else { (max as f64).log2().floor() as usize + 1 }
        };

        let mut counts: HashMap<String, u32> = HashMap::new();

        for sample in &register.samples {
            let hex = sample.trim_start_matches("0x");
            let val = u64::from_str_radix(hex, 16)
                .map_err(|e| format!("Sample hex invalide '{}' : {}", sample, e))?;
            let bitstring = format!("{:0>width$b}", val, width = num_bits);
            *counts.entry(bitstring).or_insert(0) += 1;
        }

        Ok(counts)
    }

    /// Convertit les samples de tous les PUBs en histogrammes de counts.
    ///
    /// Retourne un vecteur d'histogrammes, un par circuit soumis.
    /// Utile quand plusieurs circuits ont été soumis dans le même job.
    ///
    /// # Erreurs
    /// Retourne une erreur si `samples` est vide pour l'un des PUBs,
    /// ou si un sample n'est pas un hexadécimal valide.
    pub fn to_counts_all(&self) -> Result<Vec<HashMap<String, u32>>, String> {
        self.results.iter().enumerate().map(|(i, pub_result)| {
            let register = &pub_result.data.c;

            if register.samples.is_empty() {
                return Err(format!("PUB {} : samples vide", i));
            }

            let num_bits = if register.num_bits > 0 {
                register.num_bits as usize
            } else {
                let max = register.samples.iter()
                    .filter_map(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
                    .max().unwrap_or(0);
                if max == 0 { 1 } else { (max as f64).log2().floor() as usize + 1 }
            };

            let mut counts = HashMap::new();
            for sample in &register.samples {
                let hex = sample.trim_start_matches("0x");
                let val = u64::from_str_radix(hex, 16)
                    .map_err(|e| format!("PUB {} — sample invalide '{}' : {}", i, sample, e))?;
                let bitstring = format!("{:0>width$b}", val, width = num_bits);
                *counts.entry(bitstring).or_insert(0) += 1;
            }
            Ok(counts)
        }).collect()
    }
}

/// Réponse paginée de `GET /v1/jobs`.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobListRoot {
    /// Liste des jobs retournés
    #[serde(default)] pub jobs: Vec<Job>,
    /// Nombre total de jobs disponibles côté serveur
    #[serde(default)] pub count: i64,
    /// Nombre maximum de jobs retournés par page
    #[serde(default)] pub limit: i64,
    /// Décalage de pagination (index du premier job retourné)
    #[serde(default)] pub offset: i64,
}

/// Résumé d'un job dans une liste paginée.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    /// Nom du backend utilisé
    #[serde(default)] pub backend: String,
    /// Date de création au format ISO8601
    #[serde(default)] pub created: String,
    /// Temps d'exécution estimé en secondes
    #[serde(rename = "estimated_running_time_seconds", default)] pub estimated_running_time_seconds: f64,
    /// Identifiant unique du job
    #[serde(default)] pub id: String,
    /// Coût du job en unités IBM
    #[serde(default)] pub cost: i64,
    /// Programme IBM Quantum utilisé
    #[serde(default)] pub program: Program2,
    /// État détaillé du job
    #[serde(default)] pub state: State2,
    /// Statut global du job
    #[serde(default)] pub status: String,
    /// Identifiant de l'utilisateur
    #[serde(rename = "user_id", default)] pub user_id: String,
    /// Métriques de consommation de ressources
    #[serde(default)] pub usage: Usage,
}

/// Programme associé à un job dans une liste.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Program2 {
    /// Identifiant du programme (ex: `"sampler"`)
    #[serde(default)] pub id: String,
}

/// État d'un job dans une liste.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct State2 {
    /// Statut fin-grain (ex: `"Completed"`, `"Failed"`)
    #[serde(default)] pub status: String,
}

/// Métriques de consommation de ressources d'un job.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    /// Temps de calcul quantique consommé en secondes
    #[serde(rename = "quantum_seconds", default)] pub quantum_seconds: f64,
    /// Temps total consommé (quantique + overhead classique) en secondes
    #[serde(default)] pub seconds: f64,
}

/// Réponse immédiate de `POST /v1/jobs` après soumission d'un job.
#[derive(Debug, Deserialize)]
pub struct SubmitJobResponse {
    /// Identifiant unique du job soumis
    pub id: String,
    /// Statut initial du job (généralement `"Queued"`)
    pub status: Option<String>,
    /// Nom du backend sur lequel le job a été soumis
    pub backend: Option<String>,
    /// Date de création du job
    pub created: Option<String>,
}