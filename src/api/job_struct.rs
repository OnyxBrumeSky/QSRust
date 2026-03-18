use serde::Deserialize;
use serde::Serialize;
use base64::{engine::general_purpose, Engine};
use flate2::read::ZlibDecoder;
use std::io::Read;
use std::collections::HashMap;


/// Structure représentant un job retourné par l'API IBM Quantum.
///
/// Cette structure modélise un job quantique soumis à la plateforme,
/// incluant ses métadonnées, ses paramètres d'exécution et son état.
///
/// ⚠️ Remarque :
/// Cette structure a été initialement générée à l'aide d'un parseur JSON → Rust en ligne,
/// puis ajustée manuellement pour correspondre aux conventions Rust et à l'utilisation de Serde.
///
/// L'API IBM utilise parfois un format imbriqué avec des champs spéciaux (`__type__`, `__value__`),
/// qui sont conservés ici pour assurer la compatibilité.
///
/// Champs principaux :
/// - `id` : identifiant unique du job
/// - `backend` : backend quantique utilisé
/// - `status` : statut global du job (ex: "completed", "running")
/// - `state` : état détaillé du job
/// - `params` : paramètres d'exécution (circuits, options, etc.)
/// - `created` : date de création du job
///
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobRoot {
    pub backend: String,
    pub created: String,

    /// Temps d'exécution estimé (en secondes)
    #[serde(rename = "estimated_running_time_seconds")]
    pub estimated_running_time_seconds: f64,

    /// Identifiant unique du job
    pub id: String,

    /// Coût associé au job (si applicable)
    pub cost: i64,

    /// Paramètres d'exécution
    pub params: Params,

    /// Programme utilisé pour exécuter le job (ex: "circuit-runner")
    pub program: Program,

    /// État détaillé du job
    pub state: State,

    /// Statut global (ex: "queued", "running", "completed")
    pub status: String,

    /// Identifiant de l'utilisateur ayant soumis le job
    #[serde(rename = "user_id")]
    pub user_id: String,
}

/// Paramètres utilisés lors de la soumission du job.
///
/// Contient les options d'exécution et les données des circuits.
/// Le champ `pubs` est une représentation interne brute des circuits
/// et nécessite souvent un parsing supplémentaire pour être exploité.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Params {
    pub options: Options,

    /// Données brutes des circuits soumis (format interne IBM)
    pub pubs: Vec<(Pub, Pub2, i64)>,

    /// Indique si le job est compatible avec Qiskit
    #[serde(rename = "support_qiskit")]
    pub support_qiskit: bool,

    /// Version du schéma des paramètres
    pub version: i64,
}

/// Options d'exécution (actuellement vide, réservé pour usage futur)
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {}

/// Wrapper interne utilisé par l'API IBM (`__type__`, `__value__`).
///
/// Ces champs font partie d'un système de sérialisation générique.
/// Ils ne sont pas directement exploitables sans décodage supplémentaire.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pub {
    #[serde(rename = "__type__")]
    pub type_field: String,

    #[serde(rename = "__value__")]
    pub value: String,
}

/// Variante de `Pub`, utilisée pour représenter d'autres données internes.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pub2 {
    #[serde(rename = "__type__")]
    pub type_field: String,

    #[serde(rename = "__value__")]
    pub value: String,
}

/// Métadonnées du programme associé au job.
///
/// Contient généralement l'identifiant du programme exécuté.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Program {
    pub id: String,
}

/// État détaillé du job.
///
/// Fournit une information plus fine que le champ `status` global.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub status: String,
}



/// Structure représentant le résultat d’un job retourné par l’API IBM Quantum.
///
/// Cette structure contient les données brutes issues de l’exécution d’un circuit
/// quantique, ainsi que des métadonnées d’exécution.
///
/// ⚠️ Remarque importante :
/// Cette structure a été générée initialement via un parseur JSON → Rust en ligne,
/// puis adaptée manuellement. L’API IBM utilise un format de sérialisation imbriqué
/// avec des champs spéciaux (`__type__`, `__value__`) qui rendent la structure
/// relativement complexe.
///
/// Les résultats des mesures sont encodés en base64 dans :
/// `value.pub_results[*].value.data.value.fields.c.value.array.value`
///
/// Il est nécessaire de :
/// 1. Décoder cette chaîne base64
/// 2. Utiliser `num_bits` pour reconstruire les bits mesurés
///
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultRoot {
    #[serde(rename = "__type__")]
    pub type_field: String,

    #[serde(rename = "__value__")]
    pub value: Value,
}

/// Contenu principal du résultat.
///
/// Contient :
/// - `pub_results` : résultats des circuits exécutés
/// - `metadata` : informations globales d’exécution
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Value {
    #[serde(rename = "pub_results")]
    pub pub_results: Vec<PubResult>,

    pub metadata: Metadata2,
}

/// Résultat d’un circuit individuel.
///
/// Chaque entrée correspond à un circuit soumis.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PubResult {
    #[serde(rename = "__type__")]
    pub type_field: String,

    #[serde(rename = "__value__")]
    pub value: Value2,
}

/// Contient les données mesurées et les métadonnées associées.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Value2 {
    pub data: Data,
    pub metadata: Metadata,
}

/// Données brutes du résultat.
///
/// Utilise le format interne IBM avec `__type__` / `__value__`.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    #[serde(rename = "__type__")]
    pub type_field: String,

    #[serde(rename = "__value__")]
    pub value: Value3,
}

/// Structure décrivant les champs des résultats.
///
/// - `field_names` : noms des registres (ex: "c")
/// - `field_types` : types associés
/// - `fields` : données réelles des mesures
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Value3 {
    #[serde(rename = "field_names")]
    pub field_names: Vec<String>,

    #[serde(rename = "field_types")]
    pub field_types: Vec<String>,

    pub shape: Vec<Value>,

    pub fields: Fields,
}

/// Contient les registres de sortie.
///
/// Généralement, `c` correspond au registre classique.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fields {
    pub c: C,
}

/// Données du registre classique.
///
/// Contient les mesures encodées en base64.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct C {
    #[serde(rename = "__type__")]
    pub type_field: String,

    #[serde(rename = "__value__")]
    pub value: Value4,
}

/// Contenu réel des mesures.
///
/// - `array.value` : données encodées en base64
/// - `num_bits` : nombre de bits par mesure
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Value4 {
    pub array: Array,

    #[serde(rename = "num_bits")]
    pub num_bits: i64,
}

/// Tableau contenant les données encodées.
///
/// Le champ `value` contient une chaîne base64 représentant les résultats.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Array {
    #[serde(rename = "__type__")]
    pub type_field: String,

    #[serde(rename = "__value__")]
    pub value: String,
}

/// Métadonnées associées à un circuit.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    #[serde(rename = "circuit_metadata")]
    pub circuit_metadata: CircuitMetadata,
}

/// Métadonnées du circuit (souvent vide)
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CircuitMetadata {}

/// Métadonnées globales d’exécution.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata2 {
    pub execution: Execution,

    pub version: i64,
}

/// Informations d’exécution du job.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Execution {
    #[serde(rename = "execution_spans")]
    pub execution_spans: ExecutionSpans,
}

/// Intervalles d’exécution.
///
/// Permet de connaître les périodes de traitement du job.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionSpans {
    #[serde(rename = "__type__")]
    pub type_field: String,

    #[serde(rename = "__value__")]
    pub value: Value5,
}

/// Liste des spans d’exécution.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Value5 {
    pub spans: Vec<Span>,
}

/// Représente une période d’exécution.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Span {
    #[serde(rename = "__type__")]
    pub type_field: String,

    #[serde(rename = "__value__")]
    pub value: Value6,
}

/// Détails d’un span d’exécution.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Value6 {
    pub start: Start,
    pub stop: Stop,

    #[serde(rename = "data_slices")]
    pub data_slices: DataSlices,
}

/// Timestamp de début d’exécution.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Start {
    #[serde(rename = "__type__")]
    pub type_field: String,

    #[serde(rename = "__value__")]
    pub value: String,
}

/// Timestamp de fin d’exécution.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stop {
    #[serde(rename = "__type__")]
    pub type_field: String,

    #[serde(rename = "__value__")]
    pub value: String,
}

/// Découpage interne des données.
///
/// Structure spécifique à l’API IBM, rarement utilisée directement.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSlices {
    #[serde(rename = "0")]
    pub n0: (Vec<i64>, i64, i64, i64, i64),
}


impl ResultRoot {
    /// Convertit le résultat brut retourné par l’API IBM Quantum en un histogramme
    /// de counts (`bitstring` → nombre d’occurrences).
    ///
    /// ⚠️ Pipeline interne :
    /// Les résultats IBM ne sont pas directement exploitables. Ils sont encodés sous
    /// plusieurs couches successives :
    ///
    /// 1. Base64
    /// 2. Compression zlib
    /// 3. Format binaire NumPy (`.npy`)
    /// 4. Tableau d’entiers représentant les états mesurés
    ///
    /// Cette fonction effectue automatiquement toutes ces étapes pour produire un
    /// résultat utilisable sous forme de `HashMap<String, u32>`.
    ///
    /// Fonctionnement détaillé :
    /// - Récupère la chaîne base64 contenant les résultats
    /// - Décode en bytes
    /// - Décompresse via zlib
    /// - Parse le header `.npy` pour extraire les métadonnées (notamment le nombre de shots)
    /// - Lit les valeurs brutes (entiers)
    /// - Déduit automatiquement le nombre de qubits
    /// - Convertit chaque valeur en bitstring (ex: 3 → "11")
    /// - Compte les occurrences
    ///
    /// Format des données :
    /// - `|u1` : entiers non signés sur 8 bits
    /// - `shape = (shots, 1)` : nombre de mesures
    ///
    /// Exemple de sortie :
    /// ```text
    /// {
    ///     "00": 512,
    ///     "11": 512
    /// }
    /// ```
    ///
    /// ⚠️ Limitations :
    /// - Suppose un seul registre classique (`c`)
    /// - Suppose des données en `u8` (format standard IBM actuel)
    ///
    /// # Panics
    /// Cette fonction panic si :
    /// - le décodage base64 échoue
    /// - la décompression zlib échoue
    /// - le format `.npy` est invalide
    pub fn to_counts(&self) -> HashMap<String, u32> {

        let raw = &self.value
            .pub_results[0]
            .value
            .data
            .value
            .fields
            .c
            .value;
    
        let compressed = general_purpose::STANDARD
            .decode(&raw.array.value)
            .expect("base64 decode failed");
    
        let mut decoder = ZlibDecoder::new(&compressed[..]);
        let mut npy = Vec::new();
        decoder
            .read_to_end(&mut npy)
            .expect("zlib decode failed");
    
        assert!(&npy[0..6] == b"\x93NUMPY", "invalid npy file");
    
        let major = npy[6];
        let _minor = npy[7];
    
        let (header_len, header_start) = if major == 1 {
            let len = u16::from_le_bytes([npy[8], npy[9]]) as usize;
            (len, 10)
        } else {
            let len = u32::from_le_bytes([npy[8], npy[9], npy[10], npy[11]]) as usize;
            (len, 12)
        };
    
        let header_end = header_start + header_len;
    
        let header = std::str::from_utf8(&npy[header_start..header_end])
            .expect("invalid header utf8");
    
        let num_shots = header
            .split("shape")
            .nth(1)
            .and_then(|s| s.split('(').nth(1))
            .and_then(|s| s.split(')').next())
            .and_then(|s| s.split(',').next())
            .and_then(|s| s.trim().parse::<usize>().ok())
            .unwrap_or(0);
    
        let raw_data = &npy[header_end..];
    
        let max_val = raw_data.iter().copied().max().unwrap_or(0);
        let num_bits = if max_val == 0 {
            1
        } else {
            ((max_val as f64).log2().floor() as usize) + 1
        };
    
        let mut counts = HashMap::new();
    
        for &val in raw_data.iter().take(num_shots) {
            let bitstring = format!("{:0width$b}", val, width = num_bits);
            *counts.entry(bitstring).or_insert(0) += 1;
        }
    
        counts
    }
}


/// Représente la réponse paginée de l'API IBM Quantum pour la liste des jobs.
///
/// Cette structure est utilisée pour parser le JSON retourné par l'endpoint
/// `/jobs?fields=id`.
///
/// ⚠️ Par défaut, l'API retourne beaucoup de données pour chaque job.
/// Pour optimiser les performances et réduire la taille des réponses,
/// il est fortement recommandé d'utiliser le paramètre `fields=id`
/// afin de ne récupérer que les identifiants des jobs.
///
/// Cette structure contient :
/// - `jobs` : liste des jobs retournés
/// - `count` : nombre total de jobs disponibles côté serveur
/// - `limit` : nombre de jobs retournés dans cette réponse
/// - `offset` : position de départ (pagination)
///
/// Elle est typiquement utilisée pour récupérer efficacement les `job_id`
/// sans télécharger toutes les métadonnées associées.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobListRoot {
    pub jobs: Vec<Job>,
    pub count: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Représente un job individuel retourné par l'API IBM Quantum.
///
/// Cette structure correspond à un job complet avec ses métadonnées.
///
/// Champs importants :
/// - `id` : identifiant unique du job (clé principale)
/// - `backend` : backend utilisé (quantique ou simulateur)
/// - `status` : statut global du job
/// - `state.status` : statut détaillé (queued, running, completed, etc.)
/// - `created` : date de création du job
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub bss: Bss,
    pub backend: String,
    pub created: String,
    #[serde(rename = "estimated_running_time_seconds")]
    pub estimated_running_time_seconds: f64,
    pub id: String,
    pub cost: i64,
    pub program: Program2,
    pub state: State2,
    pub status: String,
    #[serde(rename = "user_id")]
    pub user_id: String,
    pub usage: Usage,
}

/// Informations liées au temps d'exécution backend (BSS).
///
/// Ce champ représente le temps consommé côté backend
/// en secondes pour ce job.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bss {
    pub seconds: i64,
}

/// Représente le programme IBM Quantum utilisé pour exécuter le job.
///
/// Le champ `id` correspond généralement à un runtime program
/// (ex: sampler, estimator, etc.).
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Program2 {
    pub id: String,
}

/// Représente l'état détaillé du job.
///
/// Permet d'obtenir un statut plus précis que le champ `status`
/// principal.
///
/// Exemples de valeurs :
/// - `queued`
/// - `running`
/// - `completed`
/// - `failed`
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct State2 {
    pub status: String,
}

/// Informations d'utilisation des ressources pour un job.
///
/// Contient des métriques sur le temps d'exécution réel,
/// notamment sur les ressources quantiques utilisées.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    /// Temps d'utilisation du processeur quantique (en secondes).
    #[serde(rename = "quantum_seconds")]
    pub quantum_seconds: i64,

    /// Temps total d'exécution (incluant overhead classique).
    pub seconds: i64,
}
