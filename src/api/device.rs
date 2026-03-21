use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Représente un backend IBM Quantum (réel ou simulateur).
#[derive(Deserialize, Serialize, Clone)]
pub struct Device {
    /// Nom du backend (ex: `"ibm_fez"`)
    pub name: String,
    /// Nombre de qubits disponibles
    pub qubits: u32,
    /// Nombre de jobs en attente dans la file d'exécution
    pub queue_length: u32,
    /// Statut courant du backend
    pub status: DeviceStatus,
    /// Type de processeur — `None` si le backend est un simulateur
    pub processor_type: Option<Processortype>,
}

/// Statut opérationnel d'un backend.
#[derive(Deserialize, Serialize, Clone)]
pub struct DeviceStatus {
    /// Nom du statut (ex: `"online"`, `"offline"`)
    pub name: String,
    /// Raison du statut si le backend est indisponible
    pub reason: String,
}

/// Type de processeur quantique d'un backend réel.
#[derive(Deserialize, Serialize, Clone)]
pub struct Processortype {
    /// Famille du processeur (ex: `"Heron"`, `"Eagle"`)
    pub family: String,
    /// Révision du processeur (ex: `"2"`)
    pub revision: String,
}

impl Default for Device {
    /// Retourne un backend par défaut avec 0 qubits et aucun processeur assigné.
    fn default() -> Self {
        Device {
            name: String::from("default_backend"),
            qubits: 0,
            queue_length: 0,
            status: DeviceStatus {
                name: String::new(),
                reason: String::new(),
            },
            processor_type: None,
        }
    }
}

impl Display for Device {
    /// Affiche les informations essentielles du backend sur une ligne.
    ///
    /// Format : `Backend: <nom>, Qubits: <n>, Simulator: <bool>, Status: <statut>`
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Backend: {}, Qubits: {}, Simulator: {}, Status: {}",
            self.name,
            self.qubits,
            self.processor_type.is_none(),
            self.status.name
        )
    }
}