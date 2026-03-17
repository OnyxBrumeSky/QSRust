use serde::Deserialize;
use serde::Serialize;
use base64::{engine::general_purpose, Engine};


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


/// getting the 
impl ResultRoot {
    pub fn to_count(&self) -> Vec<u8>{
        let raw = &self.value
        .pub_results[0]
        .value
        .data
        .value
        .fields
        .c
        .value
        .array
        .value;

        general_purpose::STANDARD
        .decode(raw)
        .expect("decode failed")
    }
}
