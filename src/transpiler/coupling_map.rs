use serde::Deserialize;
use std::collections::HashSet;
use crate::api::service::Service;

/// Porte native à 2 qubits supportée par le backend.
///
/// Détectée automatiquement depuis les `basis_gates` retournées par l'API IBM.
/// Utilisée par le générateur QASM pour choisir la bonne décomposition.
#[derive(Debug, Clone, PartialEq)]
pub enum TwoQubitGate {
    /// Porte CX directe — simulateurs et certains backends anciens
    CX,
    /// Porte CZ native — famille Heron (`ibm_fez`, `ibm_torino`...)
    CZ,
    /// Porte ECR native — famille Eagle (`ibm_brisbane`...)
    ECR,
}

/// Configuration brute d'un backend retournée par `GET /v1/backends/{name}/configuration`.
#[derive(Debug, Deserialize)]
pub struct BackendConfiguration {
    /// Nom du backend (ex: `"ibm_fez"`)
    pub backend_name: String,
    /// Nombre total de qubits physiques
    pub n_qubits: u32,
    /// Portes exécutables nativement (ex: `["cz", "rz", "sx", "x"]`)
    pub basis_gates: Vec<String>,
    /// Paires de qubits connectés — définit la topologie du backend
    pub coupling_map: Vec<[u32; 2]>,
}

/// Représente la topologie de connectivité d'un backend IBM Quantum.
///
/// Construit depuis la configuration du backend, il expose les méthodes
/// nécessaires au placement des qubits et au routing des circuits.
#[derive(Debug, Clone)]
pub struct CouplingMap {
    /// Nombre total de qubits physiques sur le backend
    pub n_qubits: u32,
    /// Portes nativement supportées
    pub basis_gates: Vec<String>,
    edges: Vec<(u32, u32)>,
    edge_set: HashSet<(u32, u32)>,
}

impl CouplingMap {
    /// Construit un [`CouplingMap`] depuis une [`BackendConfiguration`].
    pub fn from_config(config: BackendConfiguration) -> Self {
        let edges: Vec<(u32, u32)> = config.coupling_map
            .iter()
            .map(|pair| (pair[0], pair[1]))
            .collect();

        let edge_set: HashSet<(u32, u32)> = edges.iter().cloned().collect();

        Self {
            n_qubits: config.n_qubits,
            basis_gates: config.basis_gates,
            edges,
            edge_set,
        }
    }

    /// Détecte la porte native à 2 qubits depuis les `basis_gates`.
    ///
    /// Priorité : `cz` (Heron) > `ecr` (Eagle) > `cx` (fallback).
    pub fn two_qubit_gate(&self) -> TwoQubitGate {
        if self.basis_gates.iter().any(|g| g == "cz") {
            TwoQubitGate::CZ
        } else if self.basis_gates.iter().any(|g| g == "ecr") {
            TwoQubitGate::ECR
        } else {
            TwoQubitGate::CX
        }
    }

    /// Vérifie si une connexion directionnelle `control → target` existe.
    pub fn is_connected(&self, control: u32, target: u32) -> bool {
        self.edge_set.contains(&(control, target))
    }

    /// Vérifie si deux qubits sont voisins dans n'importe quel sens.
    pub fn are_neighbors(&self, a: u32, b: u32) -> bool {
        self.edge_set.contains(&(a, b)) || self.edge_set.contains(&(b, a))
    }

    /// Retourne tous les voisins sortants d'un qubit donné.
    pub fn neighbors(&self, qubit: u32) -> Vec<u32> {
        self.edges
            .iter()
            .filter(|(a, _)| *a == qubit)
            .map(|(_, b)| *b)
            .collect()
    }

    /// Trouve le chemin le plus court entre deux qubits via BFS.
    ///
    /// Retourne `None` si les qubits ne sont pas connectés.
    /// Utilisé par le router pour déterminer où insérer des SWAPs.
    pub fn shortest_path(&self, from: u32, to: u32) -> Option<Vec<u32>> {
        if from == to {
            return Some(vec![from]);
        }

        let mut visited = vec![false; self.n_qubits as usize];
        let mut queue = std::collections::VecDeque::new();
        let mut parent = vec![u32::MAX; self.n_qubits as usize];

        visited[from as usize] = true;
        queue.push_back(from);

        while let Some(current) = queue.pop_front() {
            for neighbor in self.neighbors(current) {
                if !visited[neighbor as usize] {
                    visited[neighbor as usize] = true;
                    parent[neighbor as usize] = current;
                    if neighbor == to {
                        let mut path = vec![to];
                        let mut node = to;
                        while node != from {
                            node = parent[node as usize];
                            path.push(node);
                        }
                        path.reverse();
                        return Some(path);
                    }
                    queue.push_back(neighbor);
                }
            }
        }
        None
    }

    /// Vérifie si une porte est nativement supportée par ce backend.
    pub fn is_native_gate(&self, gate: &str) -> bool {
        self.basis_gates.iter().any(|g| g == gate)
    }

    /// Retourne toutes les arêtes directionnelles du coupling map.
    pub fn edges(&self) -> &[(u32, u32)] {
        &self.edges
    }
}

impl Service {
    /// Récupère et construit le [`CouplingMap`] d'un backend depuis l'API IBM.
    ///
    /// Appelle `GET /v1/backends/{name}/configuration` et désérialise la réponse.
    ///
    /// # Exemple
    /// ```rust
    /// let coupling_map = service.get_coupling_map("ibm_fez").await?;
    /// println!("{:?}", coupling_map.two_qubit_gate()); // CZ
    /// ```
    pub async fn get_coupling_map(
        &self,
        backend_name: &str,
    ) -> Result<CouplingMap, Box<dyn std::error::Error>> {
        let url = format!(
            "https://quantum.cloud.ibm.com/api/v1/backends/{}/configuration",
            backend_name
        );

        let response = self.get(&url).await?;
        let text = response.text().await?;
        let config: BackendConfiguration = serde_json::from_str(&text)?;
        Ok(CouplingMap::from_config(config))
    }
}