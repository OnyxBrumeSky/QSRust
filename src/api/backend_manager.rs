use std::fmt::Display;
use serde::{Deserialize, Serialize};
use crate::api::device::Device;

/// Gestionnaire de backends IBM Quantum.
///
/// Regroupe la liste des backends disponibles et expose des méthodes
/// pour les filtrer selon leur type ou leur disponibilité.
#[derive(Deserialize, Serialize)]
pub struct BackendManager {
    /// Liste complète des backends disponibles
    pub devices: Vec<Device>,
}

/// Réponse brute de l'API IBM pour la liste des backends.
///
/// Utilisée pour désérialiser `GET /v1/backends` avant de construire un [`BackendManager`].
#[derive(Deserialize)]
pub struct BackendsResponse {
    /// Backends retournés par l'API
    pub devices: Vec<Device>,
}

impl BackendManager {
    /// Retourne la liste complète des backends.
    pub fn list(&self) -> &Vec<Device> {
        &self.devices
    }

    /// Retourne uniquement les simulateurs.
    ///
    /// Un backend est considéré comme simulateur si son champ `processor_type` est `None`.
    pub fn simulators(&self) -> Vec<&Device> {
        self.devices
            .iter()
            .filter(|b| b.processor_type.is_none())
            .collect()
    }

    /// Retourne uniquement les backends quantiques réels.
    ///
    /// Un backend est considéré comme réel si son champ `processor_type` est `Some`.
    pub fn real(&self) -> Vec<&Device> {
        self.devices
            .iter()
            .filter(|b| b.processor_type.is_some())
            .collect()
    }

    /// Retourne le backend en ligne avec la file d'attente la plus courte.
    ///
    /// Filtre les backends dont le statut est `"online"`, puis sélectionne
    /// celui avec le `queue_length` minimal. Retourne `None` si aucun backend
    /// n'est disponible en ligne.
    pub fn least_busy(&self) -> Option<&Device> {
        self.devices
            .iter()
            .filter(|d| d.status.name == "online")
            .min_by_key(|d| d.queue_length)
    }
}

impl Display for BackendManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for backend in &self.devices {
            writeln!(f, "{}", backend)?;
        }
        Ok(())
    }
}