//! Point d'entrée de la bibliothèque de simulation et d'exécution de circuits quantiques.
//!
//! Ce crate regroupe les modules nécessaires pour créer, simuler, transpiler
//! et exécuter des circuits quantiques sur les backends IBM Quantum.
//!
//! # Modules
//!
//! - [`circuit`] — Structures et fonctionnalités pour créer et manipuler des circuits quantiques
//! - [`instruction`] — Instructions et opérations utilisées dans les circuits (portes, mesures...)
//! - [`simulator`] — Simulation classique du comportement des circuits quantiques
//! - [`transpiler`] — Transpilation des circuits vers le format natif des backends IBM
//! - [`api`] — Client REST pour l'API IBM Quantum (authentification, soumission de jobs, résultats)
//! - [`visualizer`] — Export et visualisation des résultats (terminal, JSON, HTML)

/// Structures et fonctionnalités pour créer et manipuler des circuits quantiques.
pub mod circuit;

/// Simulation classique du comportement des circuits quantiques.
pub mod simulator;

/// Instructions et opérations utilisées dans les circuits (portes unitaires, mesures...).
pub mod instruction;

/// Client REST pour l'API IBM Quantum.
///
/// Fournit l'authentification IAM, la soumission de jobs, le polling de statut
/// et la récupération des résultats.
pub mod api;

/// Transpilation des circuits vers le format natif des backends IBM.
///
/// Gère la décomposition des portes, le placement des qubits sur le coupling map
/// et le routing par insertion de SWAPs.
pub mod transpiler;

/// Export et visualisation des résultats de mesure.
///
/// Propose trois modes de sortie : affichage terminal, fichier JSON et fichier HTML interactif.
pub mod visualizer;